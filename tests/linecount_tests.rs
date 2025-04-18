use linecount::{EstimateOptions, count_lines_estimate, count_lines_exact};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::io::{BufWriter, Write};
use tempfile::NamedTempFile;

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
        chunk_size: 1 << 16,
        sample_length: 500,
        num_samples: 5,
        rng: StdRng::from_entropy(),
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

#[test]
fn estimate_mean_within_four_stddevs_of_exact() {
    const NUM_LINES: usize = 100_000;
    const NUM_TRIALS: usize = 20;

    let mut file = NamedTempFile::new().expect("failed to create temp file");
    {
        let mut writer = BufWriter::new(&mut file);
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..NUM_LINES {
            let len = rng.gen_range(1..=10);
            let line: String = (0..len).map(|_| 'x').collect();
            writeln!(writer, "{}", line).unwrap();
        }
        writer.flush().unwrap();
    }

    let path = file.path();
    let exact = count_lines_exact(path).expect("failed to count lines exactly");

    let mut estimates = Vec::with_capacity(NUM_TRIALS);
    for _ in 0..NUM_TRIALS {
        let opts = EstimateOptions {
            chunk_size: 4 * 1024,
            sample_length: 2,
            num_samples: 3,
            rng: StdRng::from_entropy(),
        };
        let estimate =
            count_lines_estimate(path, opts).expect("failed to estimate number of lines");
        estimates.push(estimate as f64);
    }

    let mean: f64 = estimates.iter().sum::<f64>() / NUM_TRIALS as f64;
    let stddev: f64 =
        (estimates.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / NUM_TRIALS as f64).sqrt()
            / (NUM_TRIALS as f64).sqrt();
    let exact_f64 = exact as f64;
    let lower = mean - 4.0 * stddev;
    let upper = mean + 4.0 * stddev;

    eprintln!(
        "\n→ estimate mean: {:.0}, stddev: {:.2}, exact: {}, range: [{:.0}, {:.0}]",
        mean, stddev, exact, lower, upper
    );

    assert!(
        (exact_f64 >= lower) && (exact_f64 <= upper),
        "Estimate mean {} ± 4σ ({:.2}) does not include exact value {}",
        mean,
        stddev * 4.0,
        exact
    );
}

#[test]
fn test_estimate_is_deterministic_with_seed() {
    let content = (0..10_000)
        .map(|i| format!("line {}\n", i))
        .collect::<String>();
    let file = write_temp_file(&content);

    let seed = 42;
    let opts1 = EstimateOptions {
        chunk_size: 1 << 14,
        sample_length: 10,
        num_samples: 3,
        rng: StdRng::seed_from_u64(seed),
    };
    let opts2 = EstimateOptions {
        chunk_size: 1 << 14,
        sample_length: 10,
        num_samples: 3,
        rng: StdRng::seed_from_u64(seed),
    };

    let est1 = count_lines_estimate(file.path(), opts1).unwrap();
    let est2 = count_lines_estimate(file.path(), opts2).unwrap();

    assert_eq!(est1, est2, "Estimates with same seed should match");
}
