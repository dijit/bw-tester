use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Instant};

static GLOBAL_SUM: AtomicUsize = AtomicUsize::new(0);
static DEFAULT_MEMORY_SIZE: usize = 8 * 1024 * 1024 * 1024; // 8 GiB

#[derive(Debug, Copy, Clone)]
enum PrefetchMemory {
    Disabled,
    Enabled,
}

fn sum(prefetch: PrefetchMemory, data: &[u8], start: usize, len: usize, skip: usize) -> u64 {
    let mut sum: u64 = 0;
    let data_len = data.len();
    for i in (start..len).step_by(skip) {
        sum += data[i] as u64;
        if matches!(prefetch, PrefetchMemory::Enabled) && i + 4096 < data_len {
            unsafe { std::ptr::read_volatile(&data[i + 4096]); }
        }
    }
    GLOBAL_SUM.fetch_add(sum as usize, Ordering::Relaxed);
    sum
}

fn estimate_bandwidth(prefetch: PrefetchMemory, thread_count: usize, data: Arc<Vec<u8>>, data_volume: usize) -> f64 {
    let segment_length = data_volume / thread_count;
    let cache_line = 64;
    let mut handles = Vec::with_capacity(thread_count);

    let start = Instant::now();
    if thread_count == 1 {
        sum(prefetch, &data, 0, segment_length, cache_line);
    } else {
        for i in 0..thread_count {
            let data_clone = Arc::clone(&data);
            handles.push(thread::spawn(move || {
                sum(prefetch, &data_clone, i * segment_length, (i + 1) * segment_length, cache_line);
            }));
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }
    let elapsed = start.elapsed();
    (data_volume as f64 * 1e9) / elapsed.as_nanos() as f64
}

fn main() {
    let data_volume = DEFAULT_MEMORY_SIZE;
    let data = vec![1u8; data_volume];
    let data = Arc::new(data);
    let n = 3;

    print!("# threads\t b/w (raw)\tb/w (prefetched)\n");
    for threads in 1..=num_cpus::get() {
        let mut bw = estimate_bandwidth(PrefetchMemory::Disabled, threads, Arc::clone(&data), data_volume);
        for _ in 0..n {
            let cbw = estimate_bandwidth(PrefetchMemory::Disabled, threads, Arc::clone(&data), data_volume);
            if cbw > bw {
                bw = cbw;
            }
        }
        print!("{}\t\t {:.1}GiB/s ", threads, bw / 1e9);

        let mut bw = estimate_bandwidth(PrefetchMemory::Enabled, threads, Arc::clone(&data), data_volume);
        for _ in 0..n {
            let cbw = estimate_bandwidth(PrefetchMemory::Enabled, threads, Arc::clone(&data), data_volume);
            if cbw > bw {
                bw = cbw;
            }
        }
        println!("\t{:.1}GiB/s", bw / 1e9);
    }
}
