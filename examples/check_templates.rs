use std::{fs, path::Path, time::Instant};

fn check(path: &Path, count: &mut usize, bytes: &mut usize) {
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            check(&entry.unwrap().path(), count, bytes);
        }
    } else if path.extension().is_some_and(|ext| ext == "slim") {
        let text = fs::read_to_string(path).unwrap();
        let tokens = openidea_slim::highlights(&text);
        assert!(
            tokens
                .windows(2)
                .all(|w| w[0].range.end <= w[1].range.start),
            "{}",
            path.display()
        );
        for t in &tokens {
            assert!(t.range.start < t.range.end && t.range.end <= text.len());
            assert!(text.is_char_boundary(t.range.start) && text.is_char_boundary(t.range.end));
        }
        *count += 1;
        *bytes += text.len();
    }
}

fn main() {
    let start = Instant::now();
    let mut count = 0;
    let mut bytes = 0;
    for path in std::env::args().skip(1) {
        check(Path::new(&path), &mut count, &mut bytes);
    }
    println!(
        "Validated {count} Slim templates ({bytes} bytes) in {:?}",
        start.elapsed()
    );
    assert!(count > 0, "pass one or more template directories");
}
