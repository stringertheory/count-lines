use anyhow::Result;
use clap::Parser;
use linecount::{EstimateOptions, count_lines_estimate, count_lines_exact};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "linecount",
    version,
    about = "Count the number of lines in a file efficiently"
)]
struct Args {
    /// Path to the file
    file: PathBuf,

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
    let file_size = std::fs::metadata(&args.file)?.len();

    let mode = if args.exact {
        "exact"
    } else if args.estimate {
        "estimate"
    } else if file_size < 1_000_000_000 {
        "exact"
    } else {
        "estimate"
    };

    let count = match mode {
        "exact" => count_lines_exact(&args.file)?,
        "estimate" => {
            let opts = EstimateOptions {
                chunk_size: args.chunk_size,
                sample_length: args.sample_length,
                num_samples: args.samples,
            };
            count_lines_estimate(&args.file, opts)?
        }
        _ => unreachable!(),
    };

    println!("{count}");

    Ok(())
}
