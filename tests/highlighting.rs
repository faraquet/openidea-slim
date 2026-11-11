use openidea_plugin::syntax::{Highlight as H, Token};
use openidea_slim::highlights;

fn has(text: &str, needle: &str, kind: H) {
    let tokens = highlights(text);
    assert!(
        tokens
            .iter()
            .any(|t| &text[t.range.clone()] == needle && t.kind == kind),
        "missing {needle:?} as {kind:?}: {tokens:?}"
    );
    valid(text, &tokens);
}

fn valid(text: &str, tokens: &[Token]) {
    for t in tokens {
        assert!(t.range.start < t.range.end && t.range.end <= text.len());
        assert!(text.is_char_boundary(t.range.start) && text.is_char_boundary(t.range.end));
    }
    assert!(
        tokens
            .windows(2)
            .all(|w| w[0].range.end <= w[1].range.start)
    );
}

#[test]
fn tags_shorthand_and_attributes() {
    let s = "main#content.panel data-controller=\"booking\" hidden=true\n  h1 = t(\"title\")\n";
    for (text, kind) in [
        ("main", H::Tag),
        ("#content", H::Attribute),
        (".panel", H::Attribute),
        ("data-controller", H::Attribute),
        ("\"booking\"", H::String),
        ("true", H::Atom),
        ("t", H::FunctionCall),
    ] {
        has(s, text, kind);
    }
}

#[test]
fn ruby_control_flow_and_helpers() {
    let s = "- if @booking.paid? && Price::MAX > 12.50\n  = link_to \"Details\", booking_path(@booking), class: :button\n";
    for (text, kind) in [
        ("if", H::Keyword),
        ("@booking", H::Variable),
        ("paid?", H::FunctionCall),
        ("Price", H::Constant),
        ("MAX", H::Constant),
        ("12.50", H::Number),
        (":button", H::Atom),
    ] {
        has(s, text, kind);
    }
}

#[test]
fn nested_interpolation_and_unicode() {
    let s = "p أهلاً #{user.name} — #{format(\"#{price}\", {unit: \"€\"})}\n";
    has(s, "name", H::FunctionCall);
    has(s, "price", H::Variable);
    has(s, "\"€\"", H::String);
    has(s, "unit", H::Attribute);
}

#[test]
fn comments_cover_indented_children_and_stop_at_siblings() {
    let s = "/! a comment\n  = dangerous_looking_code\n  p text\np Visible\n";
    has(s, "= dangerous_looking_code", H::Comment);
    has(s, "p text", H::Comment);
    has(s, "p", H::Tag);
    has(s, "Visible", H::String);
}

#[test]
fn ruby_filter_and_other_filters_do_not_create_slim_tags() {
    let s = "ruby:\n  price = Money.new(10)\n  if price\n    render price\np After\njavascript:\n  const result = #{value};\np Done\n";
    has(s, "Money", H::Constant);
    has(s, "if", H::Keyword);
    has(s, "value", H::Variable);
    assert!(
        !highlights(s)
            .iter()
            .any(|t| t.kind == H::Tag && &s[t.range.clone()] == "const")
    );
}

#[test]
fn multiline_helper_call_and_hash() {
    let s = "= turbo_frame_tag :booking,\n    data: { controller: \"booking\",\n            action: :refresh } do\n  p Content\n";
    has(s, "data", H::Attribute);
    has(s, "controller", H::Attribute);
    has(s, ":refresh", H::Atom);
    has(s, "do", H::Keyword);
    has(s, "p", H::Tag);
}

#[test]
fn multiline_wrapped_attributes_and_boolean_attribute() {
    let s = "input(\n  disabled\n  data-controller=\"booking\"\n  value=(price || 0)\n)\np Next\n";
    has(s, "disabled", H::Attribute);
    has(s, "data-controller", H::Attribute);
    has(s, "price", H::Variable);
    has(s, "p", H::Tag);
}

#[test]
fn text_blocks_and_escaped_interpolation() {
    let s = "| Text \\#{literal}\n  More #{user}\np Following\n";
    has(s, "user", H::Variable);
    assert!(
        !highlights(s)
            .iter()
            .any(|t| t.kind == H::Variable && &s[t.range.clone()] == "literal")
    );
    has(s, "Following", H::String);
}

#[test]
fn block_expansion_and_whitespace_markers() {
    let s = "li: a.link< href=url =< label\nspan> Hello\n";
    has(s, "li", H::Tag);
    has(s, "a", H::Tag);
    has(s, "href", H::Attribute);
    has(s, "label", H::Variable);
}

#[test]
fn percent_literals_regex_and_division() {
    let s = "- names = %w[one two]\n- pattern = /a{2}\\d+/i\n- amount = total / 2\n";
    has(s, "%w[one two]", H::String);
    has(s, "\\d", H::Escape);
    has(s, "/", H::Operator);
    has(s, "2", H::Number);
}

#[test]
fn crlf_offsets_are_valid() {
    let s = "h1 أهلاً\r\n  = t(\"title\")\r\n";
    has(s, "t", H::FunctionCall);
}

#[test]
fn incomplete_documents_and_every_utf8_prefix() {
    for text in [
        "a(href=\"x\") Hello #{user.name}",
        "p #{format(\"#{price}\")}",
        "ruby:\n  foo = [\"é\", {key: :value}]",
        "div(data={enabled: true})",
        "= %Q(مرحبا #{name})",
    ] {
        for end in (0..=text.len()).filter(|&n| text.is_char_boundary(n)) {
            let prefix = &text[..end];
            valid(prefix, &highlights(prefix));
        }
    }
}

#[test]
fn deterministic_random_input_keeps_valid_ranges() {
    let alphabet = [
        'a', 'é', 'ا', '🙂', '\n', '\r', ' ', '\t', '#', '{', '}', '(', ')', '[', ']', '\'', '"',
        '\\', '=', '-', '.', ':', '/', '*', '%', '0',
    ];
    let mut state = 12345u32;
    for _ in 0..2000 {
        let mut s = String::new();
        for _ in 0..120 {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            s.push(alphabet[(state as usize) % alphabet.len()]);
        }
        valid(&s, &highlights(&s));
    }
}

#[test]
fn multiline_unwrapped_attribute_values() {
    let s = "div data={controller: \"booking\",\n          action: :refresh}\n  p Content\n";
    has(s, "action", H::Attribute);
    has(s, ":refresh", H::Atom);
    has(s, "p", H::Tag);
}

#[test]
fn deeply_nested_untrusted_input_is_bounded() {
    let s = format!("{}p text", "div: ".repeat(10000));
    valid(&s, &highlights(&s));
    let s = format!("p {}x{}", "#{\"".repeat(1000), "\"}".repeat(1000));
    valid(&s, &highlights(&s));
}
