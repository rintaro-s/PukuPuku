use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CpuBenchmarkResult {
    pub threads: usize,
    pub duration: Duration,
    pub total_iterations: u64,
    pub iterations_per_second: f64,
}

#[derive(Debug, Clone)]
pub struct IoBenchmarkResult {
    pub file_size_bytes: u64,
    pub block_size_bytes: usize,
    pub write_duration: Duration,
    pub read_duration: Duration,
    pub write_mebibytes_per_second: f64,
    pub read_mebibytes_per_second: f64,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub cpu: CpuBenchmarkResult,
    pub io: IoBenchmarkResult,
}

fn mebibytes_per_second(bytes: u64, duration: Duration) -> f64 {
    let secs = duration.as_secs_f64();
    if secs <= 0.0 {
        return 0.0;
    }

    (bytes as f64 / (1024.0 * 1024.0)) / secs
}

pub fn run_cpu_benchmark(duration: Duration, threads: usize) -> CpuBenchmarkResult {
    let threads = threads.max(1);
    let deadline = Instant::now() + duration;

    let mut handles = Vec::with_capacity(threads);

    for thread_index in 0..threads {
        let deadline = deadline;
        handles.push(std::thread::spawn(move || {
            let mut iterations: u64 = 0;

            // Simple integer-heavy loop; avoids allocations and keeps it deterministic.
            let mut x: u64 = 0x9E3779B97F4A7C15u64 ^ (thread_index as u64);
            while Instant::now() < deadline {
                // xorshift64*
                x ^= x >> 12;
                x ^= x << 25;
                x ^= x >> 27;
                x = x.wrapping_mul(0x2545F4914F6CDD1Du64);
                std::hint::black_box(x);
                iterations = iterations.wrapping_add(1);
            }

            iterations
        }));
    }

    let total_iterations: u64 = handles
        .into_iter()
        .filter_map(|h| h.join().ok())
        .sum();

    CpuBenchmarkResult {
        threads,
        duration,
        total_iterations,
        iterations_per_second: (total_iterations as f64) / duration.as_secs_f64().max(1e-9),
    }
}

fn default_benchmark_file_path(dir: &Path) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);

    dir.join(format!("pukupuku-io-bench-{nanos}.bin"))
}

pub fn run_io_benchmark(
    dir: &Path,
    file_size_bytes: u64,
    block_size_bytes: usize,
) -> std::io::Result<IoBenchmarkResult> {
    let block_size_bytes = block_size_bytes.max(4 * 1024);
    let file_size_bytes = file_size_bytes.max(block_size_bytes as u64);

    std::fs::create_dir_all(dir)?;

    let path = default_benchmark_file_path(dir);

    let buffer = vec![0xA5u8; block_size_bytes];

    // Write benchmark
    let write_start = Instant::now();
    {
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&path)?;
        let mut writer = BufWriter::with_capacity(block_size_bytes * 2, file);

        let mut remaining = file_size_bytes;
        while remaining > 0 {
            let to_write = (remaining as usize).min(block_size_bytes);
            writer.write_all(&buffer[..to_write])?;
            remaining -= to_write as u64;
        }

        writer.flush()?;
        // Try to make this less "pure cache" without being too expensive.
        writer.get_ref().sync_data()?;
    }
    let write_duration = write_start.elapsed();

    // Read benchmark
    let read_start = Instant::now();
    {
        let file = File::open(&path)?;
        let mut reader = BufReader::with_capacity(block_size_bytes * 2, file);
        let mut scratch = vec![0u8; block_size_bytes];

        let mut remaining = file_size_bytes;
        while remaining > 0 {
            let to_read = (remaining as usize).min(block_size_bytes);
            reader.read_exact(&mut scratch[..to_read])?;
            std::hint::black_box(&scratch[..to_read]);
            remaining -= to_read as u64;
        }
    }
    let read_duration = read_start.elapsed();

    let result = IoBenchmarkResult {
        file_size_bytes,
        block_size_bytes,
        write_duration,
        read_duration,
        write_mebibytes_per_second: mebibytes_per_second(file_size_bytes, write_duration),
        read_mebibytes_per_second: mebibytes_per_second(file_size_bytes, read_duration),
        path: path.clone(),
    };

    // Best-effort cleanup.
    let _ = std::fs::remove_file(&path);

    Ok(result)
}

pub fn run_default_benchmarks(io_dir: &Path) -> std::io::Result<BenchmarkResult> {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let cpu = run_cpu_benchmark(Duration::from_secs(1), threads);
    let io = run_io_benchmark(io_dir, 64 * 1024 * 1024, 256 * 1024)?;

    Ok(BenchmarkResult { cpu, io })
}

pub fn run_cpu_only_benchmark() -> CpuBenchmarkResult {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    run_cpu_benchmark(Duration::from_secs(1), threads)
}
