use anyhow::Result;
use rand::Rng;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

pub const SMALL_FILE_THRESHOLD: u64 = 2 * 1024 * 1024 * 1024;

const CHUNK_SIZE: usize = 1 << 16; // 64KB

#[derive(Clone)]
pub struct EstimateOptions<R: Rng> {
    pub chunk_size: usize,
    pub sample_length: usize,
    pub num_samples: usize,
    pub rng: R,
}

fn count_lines_from_reader<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buffer = [0u8; CHUNK_SIZE];
    let mut count = 0;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        count += bytecount::count(&buffer[..bytes_read], b'\n') as u64;
    }

    Ok(count)
}

pub fn count_lines_exact(path: &Path) -> Result<u64> {
    let mut file = File::open(path)?;
    count_lines_from_reader(&mut file)
}

pub fn count_lines_exact_reader<R: Read>(reader: &mut R) -> Result<u64> {
    count_lines_from_reader(reader)
}

pub fn count_lines_estimate<R: Rng>(path: &Path, opts: EstimateOptions<R>) -> Result<u64> {
    let EstimateOptions {
        chunk_size,
        sample_length,
        num_samples,
        mut rng,
    } = opts;

    let total_bytes = std::fs::metadata(path)?.len();
    let n_bytes_read = (chunk_size * sample_length * num_samples) as u64;

    if n_bytes_read > total_bytes {
        return count_lines_exact(path);
    }

    let mut file = File::open(path)?;
    let mut newline_count = 0;

    for _ in 0..num_samples {
        let start_pos = rng.gen_range(0..(total_bytes - (chunk_size * sample_length) as u64));
        file.seek(SeekFrom::Start(start_pos))?;

        let mut buffer = vec![0u8; chunk_size];
        for _ in 0..sample_length {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            newline_count += bytecount::count(&buffer[..bytes_read], b'\n') as u64;
        }
    }

    // Estimate the average number of bytes per line from the sampled data.
    // Then extrapolate to estimate total lines in the file.
    let bytes_per_line = n_bytes_read as f64 / newline_count as f64;
    let estimated = (total_bytes as f64 / bytes_per_line).round() as u64;

    Ok(estimated)
}
