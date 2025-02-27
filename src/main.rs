use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use clap::Parser;
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};

/// CLI arguments for wavefront BFS brute-forcing with UTF-8 normalization.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of threads to use in parallel (0 => use Rayon default = # of CPU cores)
    #[arg(short, long, default_value_t = 0)]
    threads: usize,

    /// Path to a file containing one username per line (any byte encoding)
    #[arg(long)]
    usernames_file: String,

    /// Path to a file containing one password per line (any byte encoding)
    #[arg(long)]
    passwords_file: String,
}

/// In a real environment, you'd make a network request here.
/// We'll just print and return `false` for demo.
fn attempt_login(username: &str, password: &str) -> bool {
    // Log: We also show the attempt here, with (username, password).
    // println!("Trying: {}:{}", username, password);

    false
}


/// Reads a file line by line and normalizes each line to valid UTF-8 using
/// `String::from_utf8_lossy()`. If the file contains invalid UTF-8 byte sequences,
/// they will be replaced with the Unicode replacement character (�).
///
/// - Each line is trimmed of the trailing newline.
/// - The result is guaranteed to be valid UTF-8 for each line.
fn read_lines_normalized(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    let mut lines = Vec::new();
    let mut buf = Vec::new();
    let mut handle = reader;

    loop {
        buf.clear();
        let bytes_read = handle.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            // EOF reached
            break;
        }
        // Remove trailing newline if present
        if let Some(&b'\n') = buf.last() {
            buf.pop();
            // If you also want to remove '\r' in Windows line-endings:
            if let Some(&b'\r') = buf.last() {
                buf.pop();
            }
        }
        // Convert raw bytes to a UTF-8 String (lossy)
        let line_utf8 = String::from_utf8_lossy(&buf).to_string();
        lines.push(line_utf8);
    }

    Ok(lines)
}

/// Wavefront BFS from (0,0) outward (usernames x passwords).
/// Each BFS "layer" is processed in parallel.
/// We stop on the first successful `attempt_login`.
/// 
/// Now includes an Indicatif progress bar for attempts and logs `(x, y)`.
fn wavefront_bruteforce(usernames: &[&str], passwords: &[&str], threads: usize) {
    let n = usernames.len();
    let m = passwords.len();

    if n == 0 || m == 0 {
        eprintln!("No usernames or no passwords provided. Aborting BFS.");
        return;
    }

    // Mark visited to avoid repeating pairs
    let mut visited = vec![vec![false; m]; n];
    visited[0][0] = true;

    // We'll store positions for BFS in a queue
    let mut queue = VecDeque::new();
    queue.push_back((0, 0));

    // Atomic flag for success
    let found = AtomicBool::new(false);

    // Shared storage for the successful (x, y)
    let success_pair = Mutex::new(None);

    // Create a custom Rayon thread pool if `threads > 0`,
    // else fall back to the global default pool
    let maybe_pool = if threads > 0 {
        Some(
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .expect("Failed to build thread pool"),
        )
    } else {
        None
    };

    // ---------------------------
    // Setup an Indicatif progress bar
    // The total number of possible attempts is n * m (each username + password pair).
    // BFS will eventually cover all pairs if no success is found first.
    let total_attempts = (n as u64) * (m as u64);
    let pb = ProgressBar::new(total_attempts);

    // You can customize the style as you wish:
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    // BFS wavefront
    while !queue.is_empty() && !found.load(Ordering::Relaxed) {
        // Extract this BFS "layer"
        let layer_size = queue.len();
        let mut layer = Vec::with_capacity(layer_size);
        for _ in 0..layer_size {
            if let Some(pos) = queue.pop_front() {
                layer.push(pos);
            }
        }

        // We'll process the layer in parallel
        let process_layer = || {
            layer.par_iter().for_each(|&(x, y)| {
                // If success was already found, no need to attempt more
                if found.load(Ordering::Relaxed) {
                    return;
                }

                // Log (x, y), then do the attempt
                // (You can remove this line if you find it too verbose.)
                //println!("Trying ({}, {}): {}:{}", x, y, usernames[x], passwords[y]);

                // Increment progress bar by 1 attempt
                pb.inc(1);

                // Actually attempt the login
                if attempt_login(usernames[x], passwords[y]) {
                    found.store(true, Ordering::Relaxed);
                    let mut lock = success_pair.lock().unwrap();
                    *lock = Some((x, y));
                }
            });
        };

        // Run either on custom or global pool
        if let Some(ref pool) = maybe_pool {
            pool.install(process_layer);
        } else {
            process_layer();
        }

        // If found a success, we stop
        if found.load(Ordering::Relaxed) {
            break;
        }

        // Enqueue neighbors for next BFS layer
        for &(x, y) in &layer {
            let candidates = [(x, y + 1), (x + 1, y), (x + 1, y + 1)];
            for &(nx, ny) in &candidates {
                if nx < n && ny < m && !visited[nx][ny] {
                    visited[nx][ny] = true;
                    queue.push_back((nx, ny));
                }
            }
        }
    }

    // Finish the progress bar
    pb.finish_and_clear();

    // Print final result
    let lock = success_pair.lock().unwrap();
    if let Some((x, y)) = *lock {
        println!(
            "SUCCESS FOUND! Username: '{}', Password: '{}'",
            usernames[x], passwords[y]
        );
    } else {
        println!("No success found after exhausting the wavefront BFS.");
    }
}

fn main() -> io::Result<()> {
    // Parse CLI arguments
    let args = Args::parse();

    // Read + normalize each line of the username file
    let username_lines = read_lines_normalized(&args.usernames_file)?;
    // Read + normalize each line of the password file
    let password_lines = read_lines_normalized(&args.passwords_file)?;

    // Convert them to &str for BFS
    let usernames: Vec<&str> = username_lines.iter().map(|s| s.as_str()).collect();
    let passwords: Vec<&str> = password_lines.iter().map(|s| s.as_str()).collect();

    println!("Starting wavefront BFS brute-force with UTF-8 normalization...");
    println!(
        "Loaded {} username(s) from '{}'",
        usernames.len(),
        args.usernames_file
    );
    println!(
        "Loaded {} password(s) from '{}'",
        passwords.len(),
        args.passwords_file
    );

    if args.threads == 0 {
        println!("Using Rayon default thread count (# of CPU cores).");
    } else {
        println!("Using {} thread(s).", args.threads);
    }

    wavefront_bruteforce(&usernames, &passwords, args.threads);

    Ok(())
}
