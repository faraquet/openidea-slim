use openidea_plugin::syntax::{Highlight, SyntaxProvider, Token};
use openidea_plugin::{Error, Invocation, Plugin, register_plugin};

#[derive(Default)]
pub struct Slim;

impl Plugin for Slim {
    fn run_command(
        &mut self,
        id: &str,
        _: &Invocation,
        _: Option<&str>,
    ) -> Result<Option<String>, Error> {
        Err(Error::NotFound(id.to_owned()))
    }
}

impl SyntaxProvider for Slim {
    fn highlights(&mut self, _: &str, text: &str) -> Vec<Token> {
        highlights(text)
    }
}

register_plugin!(Slim, syntax);

#[derive(Clone, Copy, PartialEq)]
enum Stop {
    Line,
    Space,
    Brace,
    End,
}

struct Lexer<'a> {
    text: &'a str,
    bytes: &'a [u8],
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn token(&mut self, start: usize, end: usize, kind: Highlight) {
        if end > start {
            self.tokens.push(Token {
                range: start..end,
                kind,
            });
        }
    }

    fn next(&self, i: usize) -> usize {
        i + self.text[i..].chars().next().map_or(0, char::len_utf8)
    }

    fn line_end(&self, i: usize) -> usize {
        self.text[i..]
            .find('\n')
            .map_or(self.bytes.len(), |n| i + n)
    }

    fn spaces(&self, mut i: usize, end: usize) -> usize {
        while i < end && matches!(self.bytes[i], b' ' | b'\t' | b'\r') {
            i += 1;
        }
        i
    }

    fn word(&self, mut i: usize, end: usize) -> usize {
        while i < end {
            let c = self.text[i..].chars().next().unwrap();
            if !(c.is_alphanumeric() || c == '_') {
                break;
            }
            i += c.len_utf8();
        }
        i
    }

    fn interpolation(&mut self, i: usize, end: usize, depth: usize) -> usize {
        self.token(i, i + 2, Highlight::Punctuation);
        let j = self.ruby(i + 2, end, Stop::Brace, depth + 1);
        if j < end && self.bytes[j] == b'}' {
            self.token(j, j + 1, Highlight::Punctuation);
            j + 1
        } else {
            j
        }
    }

    fn text(&mut self, mut i: usize, end: usize, kind: Highlight, depth: usize) {
        let mut start = i;
        while i < end {
            if self.bytes[i] == b'\\' {
                i += 1;
                if i < end {
                    i = self.next(i);
                }
            } else if self.bytes[i..end].starts_with(b"#{") && depth < 64 {
                self.token(start, i, kind);
                i = self.interpolation(i, end, depth);
                start = i;
            } else {
                i = self.next(i);
            }
        }
        self.token(start, end, kind);
    }

    fn string(
        &mut self,
        start: usize,
        open: usize,
        end: usize,
        interpolate: bool,
        kind: Highlight,
        depth: usize,
    ) -> usize {
        let close = match self.bytes[open] {
            b'(' => b')',
            b'[' => b']',
            b'{' => b'}',
            b'<' => b'>',
            c => c,
        };
        let paired = close != self.bytes[open];
        let mut nesting = 0;
        let mut chunk = start;
        let mut i = open + 1;
        while i < end {
            if self.bytes[i] == b'\\' {
                self.token(chunk, i, kind);
                let escape = i;
                i += 1;
                if i < end {
                    i = self.next(i);
                }
                self.token(escape, i, Highlight::Escape);
                chunk = i;
            } else if interpolate && self.bytes[i..end].starts_with(b"#{") && depth < 64 {
                self.token(chunk, i, kind);
                i = self.interpolation(i, end, depth);
                chunk = i;
            } else if paired && self.bytes[i] == self.bytes[open] {
                nesting += 1;
                i += 1;
            } else if self.bytes[i] == close {
                i += 1;
                if nesting == 0 {
                    self.token(chunk, i, kind);
                    return i;
                }
                nesting -= 1;
            } else {
                i = self.next(i);
            }
        }
        self.token(chunk, i, kind);
        i
    }

    fn ruby(&mut self, mut i: usize, end: usize, stop: Stop, depth: usize) -> usize {
        if depth >= 64 {
            let end = self.line_end(i).min(end);
            self.token(i, end, Highlight::Variable);
            return end;
        }
        let mut brackets = Vec::new();
        let mut continuation = false;
        let mut expect_value = true;
        while i < end {
            let b = self.bytes[i];
            if brackets.is_empty() {
                if stop == Stop::Brace && b == b'}' {
                    break;
                }
                if stop == Stop::Space && (b.is_ascii_whitespace() || b")]}".contains(&b)) {
                    break;
                }
                if stop == Stop::Line && b == b'\n' && !continuation {
                    break;
                }
            }
            if b.is_ascii_whitespace() {
                i += 1;
                continue;
            }
            let start = i;
            let kind;
            if b == b'#' {
                i = self.line_end(i).min(end);
                kind = Highlight::Comment;
            } else if matches!(b, b'\'' | b'"' | b'`') {
                i = self.string(i, i, end, b != b'\'', Highlight::String, depth);
                continuation = false;
                expect_value = false;
                continue;
            } else if b == b'%'
                && i + 1 < end
                && (expect_value || b"qQwWiIrxs".contains(&self.bytes[i + 1]))
            {
                let mut open = i + 1;
                let mut modifier = b'Q';
                if b"qQwWiIrxs".contains(&self.bytes[open]) {
                    modifier = self.bytes[open];
                    open += 1;
                }
                if open < end && self.bytes[open].is_ascii_punctuation() && self.bytes[open] != b'_'
                {
                    let kind = if modifier == b'r' {
                        Highlight::Regexp
                    } else {
                        Highlight::String
                    };
                    i = self.string(i, open, end, !b"qwis".contains(&modifier), kind, depth);
                    expect_value = false;
                    continuation = false;
                    continue;
                }
                i += 1;
                kind = Highlight::Operator;
            } else if b == b'/' && expect_value && self.bytes.get(i + 1) != Some(&b' ') {
                i = self.string(i, i, end, true, Highlight::Regexp, depth);
                let flags = i;
                while i < end && b"imxounes".contains(&self.bytes[i]) {
                    i += 1;
                }
                self.token(flags, i, Highlight::Regexp);
                expect_value = false;
                continuation = false;
                continue;
            } else if b.is_ascii_digit() {
                i += 1;
                while i < end
                    && (self.bytes[i].is_ascii_alphanumeric()
                        || self.bytes[i] == b'_'
                        || (self.bytes[i] == b'.'
                            && self.bytes.get(i + 1).is_some_and(u8::is_ascii_digit)))
                {
                    i += 1;
                }
                kind = Highlight::Number;
                expect_value = false;
                continuation = false;
            } else if matches!(b, b'@' | b'$') {
                i += 1;
                if i < end && self.bytes[i] == b'@' {
                    i += 1;
                }
                i = self.word(i, end);
                kind = Highlight::Variable;
                expect_value = false;
                continuation = false;
            } else if b == b':'
                && self
                    .bytes
                    .get(i + 1)
                    .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_')
                && (i == 0 || self.bytes[i - 1] != b':')
            {
                i = self.word(i + 1, end);
                kind = Highlight::Atom;
                expect_value = false;
                continuation = false;
            } else if self.text[i..]
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
            {
                i = self.word(i, end);
                if i < end
                    && matches!(self.bytes[i], b'?' | b'!')
                    && self.bytes.get(i + 1) != Some(&b'=')
                {
                    i += 1;
                }
                let word = &self.text[start..i];
                kind = if matches!(word, "true" | "false" | "nil") {
                    Highlight::Atom
                } else if matches!(
                    word,
                    "if" | "unless"
                        | "else"
                        | "elsif"
                        | "end"
                        | "do"
                        | "then"
                        | "case"
                        | "when"
                        | "in"
                        | "while"
                        | "until"
                        | "for"
                        | "begin"
                        | "rescue"
                        | "ensure"
                        | "def"
                        | "class"
                        | "module"
                        | "return"
                        | "yield"
                        | "break"
                        | "next"
                        | "redo"
                        | "retry"
                        | "super"
                        | "and"
                        | "or"
                        | "not"
                        | "alias"
                        | "defined?"
                        | "undef"
                ) {
                    Highlight::Keyword
                } else if word == "self" {
                    Highlight::SelfVariable
                } else if i < end && self.bytes[i] == b':' && self.bytes.get(i + 1) != Some(&b':') {
                    Highlight::Attribute
                } else if word.chars().next().unwrap().is_uppercase() {
                    Highlight::Constant
                } else if self.bytes.get(i) == Some(&b'(')
                    || (start > 0 && self.bytes[start - 1] == b'.')
                {
                    Highlight::FunctionCall
                } else {
                    Highlight::Variable
                };
                expect_value = kind == Highlight::Keyword;
                continuation = false;
            } else {
                i = self.next(i);
                kind = if b"()[]{}.,;".contains(&b) {
                    Highlight::Punctuation
                } else {
                    Highlight::Operator
                };
                match b {
                    b'(' => brackets.push(b')'),
                    b'[' => brackets.push(b']'),
                    b'{' => brackets.push(b'}'),
                    b')' | b']' | b'}' => {
                        if brackets.last() == Some(&b) {
                            brackets.pop();
                        }
                    }
                    _ => {}
                }
                continuation = matches!(b, b',' | b'\\');
                expect_value = !matches!(b, b')' | b']' | b'}' | b'.');
            }
            self.token(start, i, kind);
        }
        i
    }

    fn attributes(&mut self, mut i: usize, end: usize, close: Option<u8>) -> usize {
        loop {
            i = self.spaces(i, end);
            if close.is_some() {
                while i < end && self.bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
            }
            if i >= end {
                return i;
            }
            if close == Some(self.bytes[i]) {
                self.token(i, i + 1, Highlight::Punctuation);
                return i + 1;
            }
            if self.bytes[i] == b'*' {
                self.token(i, i + 1, Highlight::Operator);
                i = self.ruby(i + 1, end, Stop::Space, 0);
                continue;
            }
            let start = i;
            while i < end
                && (self.bytes[i].is_ascii_alphanumeric() || b"_-:@.".contains(&self.bytes[i]))
            {
                i += 1;
            }
            let equal = self.spaces(i, end);
            if i > start && equal < end && self.bytes[equal] == b'=' {
                self.token(start, i, Highlight::Attribute);
                let mut value = equal + 1;
                if self.bytes.get(value) == Some(&b'=') {
                    value += 1;
                }
                self.token(equal, value, Highlight::Operator);
                value = self.spaces(value, end);
                i = self.ruby(value, self.bytes.len(), Stop::Space, 0);
                if i == value && i < end && close != Some(self.bytes[i]) {
                    return i;
                }
            } else if close.is_some() && i > start {
                self.token(start, i, Highlight::Attribute);
            } else {
                return start;
            }
        }
    }

    fn tag(&mut self, mut i: usize, end: usize, depth: usize) -> usize {
        if depth >= 64 {
            self.text(i, end, Highlight::String, 0);
            return end;
        }
        let start = i;
        while i < end
            && (self.bytes[i].is_ascii_alphanumeric()
                || b"_-".contains(&self.bytes[i])
                || (self.bytes[i] == b':'
                    && self.bytes.get(i + 1).is_some_and(u8::is_ascii_alphabetic)))
        {
            i += 1;
        }
        self.token(start, i, Highlight::Tag);
        while i < end
            && matches!(self.bytes[i], b'.' | b'#')
            && !self.bytes[i..end].starts_with(b"#{")
        {
            let start = i;
            i += 1;
            while i < end
                && (self.bytes[i].is_ascii_alphanumeric() || b"_-".contains(&self.bytes[i]))
            {
                i += 1;
            }
            self.token(start, i, Highlight::Attribute);
        }
        loop {
            i = self.spaces(i, end);
            if i >= end {
                return i;
            }
            if matches!(self.bytes[i], b'<' | b'>') {
                self.token(i, i + 1, Highlight::Operator);
                i += 1;
            } else if matches!(self.bytes[i], b'(' | b'[' | b'{') {
                let close = match self.bytes[i] {
                    b'(' => b')',
                    b'[' => b']',
                    _ => b'}',
                };
                self.token(i, i + 1, Highlight::Punctuation);
                i = self.attributes(i + 1, self.bytes.len(), Some(close));
                return self.tag_tail(i, depth);
            } else {
                let after = self.attributes(i, end, None);
                return self.tag_tail(after, depth);
            }
        }
    }

    fn tag_tail(&mut self, i: usize, depth: usize) -> usize {
        let end = self.line_end(i);
        let i = self.spaces(i, end);
        if i >= end {
            return end;
        }
        if self.bytes[i] == b'=' {
            let mut j = i + 1;
            while j < end && b"=<>".contains(&self.bytes[j]) {
                j += 1;
            }
            self.token(i, j, Highlight::Operator);
            return self.ruby(j, self.bytes.len(), Stop::Line, 0);
        }
        if self.bytes[i] == b':' {
            self.token(i, i + 1, Highlight::Punctuation);
            return self.tag(self.spaces(i + 1, end), end, depth + 1);
        }
        if self.bytes[i] == b'/' && self.text[i + 1..end].trim().is_empty() {
            self.token(i, i + 1, Highlight::Punctuation);
        } else {
            self.text(i, end, Highlight::String, 0);
        }
        end
    }
}

pub fn highlights(text: &str) -> Vec<Token> {
    let mut l = Lexer {
        text,
        bytes: text.as_bytes(),
        tokens: Vec::new(),
    };
    let mut i = 0;
    let mut block: Option<(usize, Highlight)> = None;
    while i < text.len() {
        let end = l.line_end(i);
        let start = l.spaces(i, end);
        let indent = start - i;
        if start == end {
            i = (end + 1).min(text.len());
            continue;
        }
        if let Some((parent, kind)) = block {
            if indent > parent {
                if kind == Highlight::Comment {
                    l.token(start, end, kind);
                } else {
                    l.text(start, end, kind, 0);
                }
                i = (end + 1).min(text.len());
                continue;
            }
            block = None;
        }
        let line = &text[start..end];
        let mut consumed = end;
        match l.bytes[start] {
            b'/' => {
                l.token(start, end, Highlight::Comment);
                block = Some((indent, Highlight::Comment));
            }
            b'|' | b'\'' => {
                l.token(start, start + 1, Highlight::Operator);
                l.text(start + 1, end, Highlight::String, 0);
                block = Some((indent, Highlight::String));
            }
            b'=' | b'-' => {
                let mut ruby = start + 1;
                while ruby < end && b"=<>".contains(&l.bytes[ruby]) {
                    ruby += 1;
                }
                l.token(start, ruby, Highlight::Operator);
                consumed = l.ruby(ruby, text.len(), Stop::Line, 0);
            }
            _ if line == "doctype" || line.starts_with("doctype ") => {
                l.token(start, end, Highlight::Keyword)
            }
            _ if line.trim_end().ends_with(':')
                && line
                    .trim_end_matches(|c: char| c == ':' || c.is_whitespace())
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_') =>
            {
                l.token(start, end, Highlight::Keyword);
                if line.trim() == "ruby:" {
                    let mut finish = (end + 1).min(text.len());
                    while finish < text.len() {
                        let e = l.line_end(finish);
                        let s = l.spaces(finish, e);
                        if s < e && s - finish <= indent {
                            break;
                        }
                        finish = (e + 1).min(text.len());
                    }
                    l.ruby((end + 1).min(text.len()), finish, Stop::End, 0);
                    i = finish;
                    continue;
                } else {
                    block = Some((indent, Highlight::String));
                }
            }
            b'<' => l.text(start, end, Highlight::String, 0),
            _ => consumed = l.tag(start, end, 0),
        }
        i = (consumed + 1).min(text.len());
    }
    l.tokens
}
