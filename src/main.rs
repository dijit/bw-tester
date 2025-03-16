use std::time::Instant;
use std::thread;
use std::sync::Arc;
use std::env;

const DEFAULT_BUFFER_SIZE_MB: usize = 8 * 1024;

fn run_test(buffer_size: usize, thread_count: usize) {
    println!("Testing memory bandwidth:");
    println!("  Buffer Size: {} MB", buffer_size);
    println!("  Threads: {}", thread_count);

    let total_bytes = buffer_size * 1024 * 1024;
    let chunk_size = total_bytes / thread_count;

    let source_buffer = Arc::new(vec![0u8; total_bytes]);
    let destination_buffer = Arc::new(std::sync::Mutex::new(vec![0u8; total_bytes]));

    let start_time = Instant::now();
    let mut handles = vec![];

    for i in 0..thread_count {
        let src_start = i * chunk_size;
        let src_end = if i == thread_count - 1 { total_bytes } else { (i + 1) * chunk_size };
        let dst_start = src_start;
        let dst_end = src_end;

        let source_buffer_clone = source_buffer.clone();
        let dst_buffer_arc = destination_buffer.clone();

        let handle = thread::spawn(move || {
            let src_slice = &source_buffer_clone[src_start..src_end];
            let mut dst_buffer_guard = dst_buffer_arc.lock().unwrap();
            let dst_slice = &mut dst_buffer_guard[dst_start..dst_end];
            dst_slice.copy_from_slice(src_slice);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed_time = start_time.elapsed();
    let elapsed_seconds = elapsed_time.as_secs_f64();

    if elapsed_seconds > 0.0 {
        let bandwidth_gbps = (total_bytes as f64 / elapsed_seconds) / (1024.0 * 1024.0 * 1024.0);
        println!("  Time taken: {:.4} seconds", elapsed_seconds);
        println!("  Bandwidth: {:.2} GB/s", bandwidth_gbps);
    } else {
        println!("  Error: Elapsed time is zero.");
    }
    println!();
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let buffer_size_mb = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_BUFFER_SIZE_MB);
    let thread_count = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(num_cpus::get());

    println!("BlandWidth (Simple - Optimized)");
    println!("===============================\n");

    run_test(buffer_size_mb, thread_count);
}