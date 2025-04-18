use count_lines::{EstimateOptions, count_lines_estimate, count_lines_exact};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs::{File, create_dir_all};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

fn get_fixture_path() -> PathBuf {
    let path = Path::new("bench_data/lines_1g.txt");
    if !path.exists() {
        println!("Generating fixture: {}", path.display());
        create_dir_all(path.parent().unwrap()).unwrap();

        let file = File::create(path).expect("Failed to create fixture file");
        let mut writer = BufWriter::new(file);

        let mut rng = StdRng::seed_from_u64(17);

        for _ in 0..1_000_000_000 {
            let len = rng.gen_range(1..100);
            let line: String = (0..len)
                .map(|_| rng.gen_range(b'a'..=b'z') as char)
                .collect();
            writeln!(writer, "{}", line).unwrap();
        }
        writer.flush().unwrap();
    }
    path.to_path_buf()
}

fn benchmark_line_counts(c: &mut Criterion) {
    let path = get_fixture_path();

    let mut exact_group = c.benchmark_group("line_counts_exact");
    exact_group.sample_size(10);
    exact_group.bench_function("count_lines_exact (1G lines)", |b| {
        b.iter(|| {
            let result = count_lines_exact(black_box(&path)).unwrap();
            black_box(result);
        });
    });
    exact_group.finish();

    c.bench_function("count_lines_estimate (1G lines)", |b| {
        let opts = EstimateOptions {
            chunk_size: 1 << 16,
            sample_length: 500,
            num_samples: 5,
            rng: StdRng::from_entropy(),
        };
        b.iter(|| {
            let opts = opts.clone();
            let result = count_lines_estimate(black_box(&path), opts).unwrap();
            black_box(result);
        });
    });
}

criterion_group!(benches, benchmark_line_counts);
criterion_main!(benches);
