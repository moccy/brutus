use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use clap::Parser;
use rayon::prelude::*;

/// A simple CLI for wavefront BFS-based brute-forcing.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of threads to use in parallel (default = number of CPU cores)
    #[arg(short, long, default_value_t = 0)]
    threads: usize,

    /// Comma-separated list of usernames
    #[arg(long, default_value = "admin,user,test")]
    usernames: String,

    /// Comma-separated list of passwords
    #[arg(long, default_value = "1234,password,admin,letmein")]
    passwords: String,
}

/// In a real environment, you'd make an actual login/network request here.
fn attempt_login(username: &str, password: &str) -> bool {
    // For demo: just printing and always returning false.
    println!("Trying: {}:{}", username, password);
    false
}

/// Performs a wavefront BFS from (0,0) outward over a grid of
/// size (num_usernames x num_passwords). Each BFS "layer" is processed
/// in parallel. We stop (short-circuit) as soon as any attempt succeeds.
fn wavefront_bruteforce(usernames: &[&str], passwords: &[&str], threads: usize) {
    let n = usernames.len();
    let m = passwords.len();

    // Mark visited to avoid repeating pairs
    let mut visited = vec![vec![false; m]; n];
    visited[0][0] = true;

    // We'll store positions for BFS in a queue
    let mut queue = VecDeque::new();
    queue.push_back((0, 0));

    // This AtomicBool indicates if we've found a valid login
    let found = AtomicBool::new(false);

    // We'll store the successful (x, y) in a Mutex so multiple threads
    // can write it, but only the first success is relevant.
    let success_pair = Mutex::new(None);

    // Setup a custom Rayon thread pool if threads != 0,
    // otherwise just use the global default pool.
    let maybe_pool = if threads > 0 {
        Some(rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .expect("Failed to build thread pool"))
    } else {
        None
    };

    // BFS wavefront
    while !queue.is_empty() && !found.load(Ordering::Relaxed) {
        // Extract the entire current BFS "layer" from the queue
        let layer_size = queue.len();
        let mut layer = Vec::with_capacity(layer_size);
        for _ in 0..layer_size {
            if let Some(pos) = queue.pop_front() {
                layer.push(pos);
            }
        }

        // Process this BFS layer in parallel
        let process_layer = || {
            layer.par_iter().for_each(|&(x, y)| {
                // If someone else already found a success, skip more attempts
                if found.load(Ordering::Relaxed) {
                    return;
                }
                // Attempt login
                if attempt_login(usernames[x], passwords[y]) {
                    found.store(true, Ordering::Relaxed);
                    // Record the success
                    let mut lock = success_pair.lock().unwrap();
                    *lock = Some((x, y));
                }
            });
        };

        // If we built a custom pool, run inside it; otherwise just run globally
        if let Some(ref pool) = maybe_pool {
            pool.install(process_layer);
        } else {
            process_layer();
        }

        // If found a success, we can stop BFS
        if found.load(Ordering::Relaxed) {
            break;
        }

        //
        // Gather neighbors for the *next* BFS layer
        //
        for &(x, y) in &layer {
            // The "wavefront" neighbors: down, right, diagonal
            let candidates = [(x, y + 1), (x + 1, y), (x + 1, y + 1)];
            for &(nx, ny) in &candidates {
                if nx < n && ny < m && !visited[nx][ny] {
                    visited[nx][ny] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    // Report final result
    let guard = success_pair.lock().unwrap(); // store the lock in a local variable
    if let Some((x, y)) = *guard {
        println!(
            "SUCCESS FOUND! Username: {}, Password: {}",
            usernames[x], passwords[y]
        );
    } else {
        println!("No success found after exhausting the wavefront BFS.");
    }
}

fn main() {
    // Parse CLI arguments
    let args = Args::parse();

    // Split the comma-separated usernames/passwords
    let usernames: Vec<_> = args.usernames.split(',').collect();
    let passwords: Vec<_> = args.passwords.split(',').collect();

    println!("Starting wavefront BFS brute-force...");
    println!("Usernames: {:?}", usernames);
    println!("Passwords: {:?}", passwords);
    if args.threads == 0 {
        println!("Using default Rayon thread count (usually # of CPU cores).");
    } else {
        println!("Using {} thread(s).", args.threads);
    }

    wavefront_bruteforce(&usernames, &passwords, args.threads);
}
