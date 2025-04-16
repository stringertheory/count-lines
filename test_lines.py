import time
import os
import itertools
import subprocess
import random

def simple(filename):
    with open(filename) as f:
        for i, _ in enumerate(f):
            pass
    return i + 1

def buf_count_newlines_gen(fname):
    def _make_gen(reader):
        while True:
            b = reader(2 ** 16)
            if not b: break
            yield b

    with open(fname, "rb") as f:
        count = sum(buf.count(b"\n") for buf in _make_gen(f.raw.read))
    return count

def iter_chunks(stream, start, chunk_size, num_chunks):
    stream.seek(start)
    for chunk_number in range(num_chunks):
        chunk = stream.read(chunk_size)
        remaining = chunk_size - len(chunk)
        if remaining:
            print('snabick mcjabick', chunk_number, remaining)
            stream.seek(0)
            chunk += stream.read(remaining)
        yield chunk

def iter_all_chunks(read_stream, chunk_size=2**16):
    while True:
        b = read_stream.read(chunk_size)
        if not b:
            break
        yield b

def count_newlines_sample(fname):

    stat = os.stat(fname)
    print(stat)
    
    n_samples = 5
    chunk_size = 2**16
    sample_length = 500
    
    total_bytes = os.path.getsize(fname)
    print(total_bytes, total_bytes / (1024 * 1024))
    n_bytes_read = n_samples * chunk_size * sample_length
    print(n_bytes_read / (1024 * 1024))

    if n_bytes_read > total_bytes:
        return count_newlines(fname)
    
    with open(fname, "rb", buffering=0) as stream:
        n_newlines = 0
        for i in range(n_samples):
            start = random.randint(0, total_bytes - 1)
            n_newlines += sum(chunk.count(b"\n") for chunk in iter_chunks(stream, start, chunk_size, sample_length))

    bytes_per_line = n_bytes_read / n_newlines
    answer = int(round(total_bytes / bytes_per_line))
        
    return answer

def count_newlines(fname):

    with open(fname, "rb", buffering=0) as stream:
        count = sum(chunk.count(b"\n") for chunk in iter_all_chunks(stream))

    return count

def wc_l(fname):
    return int(subprocess.check_output(["wc", "-l", fname]).split()[0])

def make_file(filename, n_lines, line_size, final_newline=True):
    with open(filename, 'w') as outfile:
        for i in range(n_lines - 1):
            outfile.write(f"{random.randint(0, line_size) * 'x'}\n")
        outfile.write(f"{random.randint(0, line_size) * 'x'}")
        if final_newline:
            outfile.write("\n")

# make_file('small-no.txt', 4200, 100, final_newline=False)

for filename in ['kk-no.txt', 'big-no.txt', 'small-no.txt']:

    for f in [wc_l, count_newlines, count_newlines_sample]:
        i = time.perf_counter()
        print(filename, f.__name__, f(filename))
        print(time.perf_counter() - i)

# for i in range(100):
#     print(count_newlines_sample('big-no.txt'))
