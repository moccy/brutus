// src/lib.rs

mod strategy;
pub use strategy::*;

use std::fs::File;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;

/// Reads the file via memory mapping and returns a Vec<String> with a lossy conversion.
pub fn read_lines_lossy(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = String::from_utf8_lossy(&mmap);
    let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    Ok(lines)
}

/// Process a diagonal (wavefront) layer using CPU parallelism.
/// The provided `login_attempt` strategy is called for each username/password pair.
pub fn process_layer_cpu(
    layer: &[(usize, usize)],
    usernames: &[&str],
    passwords: &[&str],
    pb: &ProgressBar,
    found: &AtomicBool,
    success_pair: &Mutex<Option<(usize, usize)>>,
    login_attempt: &dyn LoginStrategy,
) {
    layer.par_iter().for_each(|&(x, y)| {
        if found.load(Ordering::Relaxed) {
            return;
        }
        pb.inc(1);
        if login_attempt.attempt(usernames[x], passwords[y]) {
            found.store(true, Ordering::Relaxed);
            let mut lock = success_pair.lock().unwrap();
            *lock = Some((x, y));
        }
    });
}

/// Diagonal brute-force algorithm that avoids building an n×m visited matrix.
/// Iterates over diagonals (wavefronts) where d = i + j.
pub fn diagonal_bruteforce(
    usernames: &[&str],
    passwords: &[&str],
    threads: usize,
    login_attempt: &dyn LoginStrategy,
) -> io::Result<()> {
    let n = usernames.len();
    let m = passwords.len();

    if n == 0 || m == 0 {
        eprintln!("No usernames or no passwords provided. Aborting.");
        return Ok(());
    }

    let total_attempts = (n as u64) * (m as u64);
    let pb = ProgressBar::new(total_attempts);
    pb.set_style(
        ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );

    let found = AtomicBool::new(false);
    let success_pair = Mutex::new(None);

    for d in 0..=(n + m - 2) {
        if found.load(Ordering::Relaxed) {
            break;
        }

        let mut layer = Vec::new();
        let start = if d >= m { d - m + 1 } else { 0 };
        let end = if d < n { d } else { n - 1 };
        for i in start..=end {
            let j = d - i;
            if j < m {
                layer.push((i, j));
            }
        }

        if threads > 0 {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| {
                process_layer_cpu(
                    &layer,
                    usernames,
                    passwords,
                    &pb,
                    &found,
                    &success_pair,
                    login_attempt,
                )
            });
        } else {
            process_layer_cpu(
                &layer,
                usernames,
                passwords,
                &pb,
                &found,
                &success_pair,
                login_attempt,
            );
        }
    }

    pb.finish_and_clear();
    let lock = success_pair.lock().unwrap();
    if let Some((x, y)) = *lock {
        println!("SUCCESS FOUND! Username: '{}', Password: '{}'", usernames[x], passwords[y]);
    } else {
        println!("No success found after exhausting the search space.");
    }
    Ok(())
}

/// Runs the brute-force search.  
/// If `target_url` is provided, the `%user%` and `%pass%` tokens in the URL or body are replaced.
/// If `target_body` is provided, a POST request is made; otherwise, a GET request is made.
/// If neither is provided, a dummy strategy is used.
pub fn run_bruteforce(
    usernames_file: &str,
    passwords_file: &str,
    threads: usize,
    target_url: Option<&str>,
    target_body: Option<&str>,
) -> io::Result<()> {
    let usernames_vec = read_lines_lossy(usernames_file)?;
    let passwords_vec = read_lines_lossy(passwords_file)?;

    let usernames: Vec<&str> = usernames_vec.iter().map(|s| s.as_str()).collect();
    let passwords: Vec<&str> = passwords_vec.iter().map(|s| s.as_str()).collect();

    println!("Starting diagonal brute-force with UTF-8 normalization...");
    println!("Loaded {} username(s) from '{}'", usernames.len(), usernames_file);
    println!("Loaded {} password(s) from '{}'", passwords.len(), passwords_file);

    if threads == 0 {
        println!("Using Rayon default thread count (# of CPU cores).");
    } else {
        println!("Using {} thread(s).", threads);
    }

    let strategy: Box<dyn LoginStrategy> = if let Some(url) = target_url {
        if let Some(body) = target_body {
            Box::new(PostStrategy::new(url, body))
        } else {
            Box::new(GetStrategy::new(url))
        }
    } else {
        Box::new(DummyStrategy::new())
    };

    diagonal_bruteforce(&usernames, &passwords, threads, &*strategy)
}
