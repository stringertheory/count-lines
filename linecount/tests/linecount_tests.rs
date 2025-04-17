use std::io::Write;
use tempfile::NamedTempFile;

use linecount::{EstimateOptions, count_lines_estimate, count_lines_exact};

fn write_temp_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    write!(file, "{}", content).unwrap();
    file
}

#[test]
fn test_empty_file() {
    let file = write_temp_file("");
    assert_eq!(count_lines_exact(file.path()).unwrap(), 0);
}

#[test]
fn test_one_line_no_newline() {
    let file = write_temp_file("Hello, world!");
    assert_eq!(count_lines_exact(file.path()).unwrap(), 0); // no '\n'
}

#[test]
fn test_one_line_with_newline() {
    let file = write_temp_file("Hello, world!\n");
    assert_eq!(count_lines_exact(file.path()).unwrap(), 1);
}

#[test]
fn test_multiple_lines() {
    let content = "line1\nline2\nline3\n";
    let file = write_temp_file(content);
    assert_eq!(count_lines_exact(file.path()).unwrap(), 3);
}

#[test]
fn test_multiple_lines_no_trailing_newline() {
    let content = "line1\nline2\nline3";
    let file = write_temp_file(content);
    assert_eq!(count_lines_exact(file.path()).unwrap(), 2); // no newline after last line
}

#[test]
fn test_estimation_on_small_file_matches_exact() {
    let content = (0..10_000)
        .map(|i| format!("line {}\n", i))
        .collect::<String>();
    let file = write_temp_file(&content);

    let exact = count_lines_exact(file.path()).unwrap();

    let opts = EstimateOptions {
        chunk_size: 1 << 16, // 64KB
        sample_length: 500,
        num_samples: 5,
    };

    let estimated = count_lines_estimate(file.path(), opts).unwrap();

    let diff = (exact as i64 - estimated as i64).abs();
    let allowed_error = (exact as f64 * 0.01).ceil() as i64; // 1% tolerance

    assert!(
        diff <= allowed_error,
        "estimated {} vs exact {}, diff {} exceeds allowed error {}",
        estimated,
        exact,
        diff,
        allowed_error
    );
}
