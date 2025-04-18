# count-lines

`lc` is a command-line tool to count lines in files or standard
input, with optional estimation for large files.

## Features
- Exact and estimated line counting
- Automatically chooses method based on file size
- Fast performance with chunked reading and sampling
- Supports seeding RNG for reproducible estimates

## Installation

```sh
cargo install count-lines
```

## Usage
```sh
lc [OPTIONS] [FILE]
```

If `FILE` is omitted or set to `-`, the program reads from standard input.

### Options
- `--exact` - Force exact line count
- `--estimate` - Force estimated line count
- `--chunk-size <SIZE>` - Size of chunks to read (default: 64KB). Supports suffixes like `KB`, `MB`.
- `--sample-length <N>` - Number of chunks to read per sample (default: 500)
- `--samples <N>` - Number of samples to take (default: 5)
- `--seed <N>` - Seed for reproducible estimation

### Example
```sh
lc bigfile.txt          # Automatically uses estimate if file is large
lc --exact data.csv     # Force exact count
lc --estimate --seed 17 huge.log
```

## How It Works

### Exact Method
The file is read in chunks, counting newline characters (`\n`) using the `bytecount` crate.

### Estimate Method
1. Randomly samples `samples` number of positions in the file.
2. Reads `sample_length` chunks of `chunk_size` bytes from each position.
3. Counts newlines in all sampled bytes.
4. Computes the average bytes per line and extrapolates to the entire file size.

### Auto Mode
- If no method is specified:
  - Uses exact for small files (<2GB)
  - Uses estimation for larger files

### Reproducibility
Using `--seed` ensures the same samples are selected, leading to consistent results.

## Development
Run tests with:
```
cargo test
```
Benchmark performance with:
```
cargo bench
```
