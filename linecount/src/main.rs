use anyhow::Result;
use clap::Parser;
use linecount::{
    EstimateOptions, SMALL_FILE_THRESHOLD, count_lines_estimate, count_lines_exact,
    count_lines_exact_reader,
};
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::io::{self};
use std::path::PathBuf;

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

    /// Chunk size (e.g. 64KB, 1MB)
    #[arg(long, default_value = "64KB", value_parser = parse_bytes)]
    chunk_size: usize,

    /// Number of chunks to read per sample
    #[arg(long, default_value_t = 500)]
    sample_length: usize,

    /// Number of samples to take
    #[arg(long, default_value_t = 5)]
    samples: usize,

    /// Optional seed for random number generation (for reproducibility)
    #[arg(long)]
    seed: Option<u64>,
}

fn parse_bytes(src: &str) -> Result<usize, String> {
    let lowercase = src.to_ascii_lowercase();
    let num_part: String = lowercase.chars().filter(|c| c.is_ascii_digit()).collect();
    let suffix = lowercase.trim_start_matches(&num_part);
    let base: usize = num_part.parse().map_err(|_| "Invalid number")?;

    let multiplier = match suffix {
        "" => 1,
        "kb" => 1024,
        "mb" => 1024 * 1024,
        _ => return Err("Invalid size suffix (use KB, MB)".into()),
    };

    Ok(base * multiplier)
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
