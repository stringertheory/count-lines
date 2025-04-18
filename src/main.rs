use anyhow::Result;
use byte_unit::Byte;
use clap::Parser;
use linecount::{
    EstimateOptions, SMALL_FILE_THRESHOLD, count_lines_estimate, count_lines_exact,
    count_lines_exact_reader,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::io::{self};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser, Debug)]
#[command(
    name = "linecount",
    version,
    about = "Count the number of lines in a file or stdin efficiently"
)]
struct Args {
    /// File to read (use '-' or omit to read from stdin)
    file: Option<PathBuf>,

    /// Force exact line counting
    #[arg(long, conflicts_with = "estimate")]
    exact: bool,

    /// Force estimated line counting
    #[arg(long, conflicts_with = "exact")]
    estimate: bool,

    /// Chunk size (e.g. 64kB, 1MB, 2GB)
    #[arg(long, default_value = "64kB", value_parser = parse_bytes)]
    chunk_size: usize,

    /// Number of chunks to read per sample
    #[arg(long, default_value_t = 500)]
    sample_length: usize,

    /// Number of samples to take
    #[arg(long, default_value_t = 5)]
    samples: usize,

    /// Optional seed for random number generation (for reproducibility)
    #[arg(
        long,
        help = "Seed for RNG (used in --estimate mode for reproducible results)"
    )]
    seed: Option<u64>,
}

fn parse_bytes(src: &str) -> Result<usize, String> {
    Byte::from_str(src)
        .map_err(|e| format!("Invalid chunk size: {}", e))
        .map(|b| b.as_u128() as usize)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let use_stdin =
        args.file.is_none() || args.file.as_deref() == Some(PathBuf::from("-").as_path());

    if use_stdin {
        if args.estimate {
            eprintln!(
                "Warning: wc --estimate is not supported for stdin. Falling back to exact count."
            );
        }

        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let count = count_lines_exact_reader(&mut handle)?;
        println!("{count}");
        return Ok(());
    }

    let path = args.file.unwrap();
    let file_size = std::fs::metadata(&path)?.len();

    let mode = if args.exact {
        "exact"
    } else if args.estimate {
        "estimate"
    } else if file_size < SMALL_FILE_THRESHOLD {
        "exact"
    } else {
        "estimate"
    };

    // Create the random number generator (RNG)
    let rng = if let Some(seed) = args.seed {
        // Use the provided seed for reproducibility
        StdRng::seed_from_u64(seed)
    } else {
        // Use a time-based RNG if no seed is provided
        StdRng::from_entropy()
    };

    let count = match mode {
        "exact" => count_lines_exact(&path)?,
        "estimate" => {
            let opts = EstimateOptions {
                chunk_size: args.chunk_size,
                sample_length: args.sample_length,
                num_samples: args.samples,
                rng,
            };
            count_lines_estimate(&path, opts)?
        }
        _ => unreachable!(),
    };

    println!("{count}");
    Ok(())
}
