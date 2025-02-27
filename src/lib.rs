use std::fs::File;
use std::io::{self};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;

/// In a real environment, you'd perform a network request or heavy hash calculation here.
/// For demo purposes, this always returns false.
pub fn attempt_login(username: &str, password: &str) -> bool {
    false
}

/// Reads the file via memory mapping and returns a Vec<String> with a lossy conversion.
/// This handles both ASCII and non‑UTF‑8 files.
pub fn read_lines_lossy(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file)? };
    // Use from_utf8_lossy: if the file is valid ASCII (or UTF‑8) it borrows, otherwise it allocates.
    let content = String::from_utf8_lossy(&mmap);
    let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    Ok(lines)
}

/// Process a wavefront layer using CPU parallelism.
pub fn process_layer_cpu(
    layer: &[(usize, usize)],
    usernames: &[&str],
    passwords: &[&str],
    pb: &ProgressBar,
    found: &AtomicBool,
    success_pair: &Mutex<Option<(usize, usize)>>,
) {
    layer.par_iter().for_each(|&(x, y)| {
        if found.load(Ordering::Relaxed) {
            return;
        }
        pb.inc(1);
        if attempt_login(usernames[x], passwords[y]) {
            found.store(true, Ordering::Relaxed);
            let mut lock = success_pair.lock().unwrap();
            *lock = Some((x, y));
        }
    });
}

/// Diagonal brute-force algorithm that avoids building an n×m visited matrix.
/// It iterates over diagonals (wavefronts) where d = i + j.
pub fn diagonal_bruteforce(
    usernames: &[&str],
    passwords: &[&str],
    threads: usize,
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

    // Instead of building a huge visited matrix, iterate over diagonals.
    for d in 0..=(n + m - 2) {
        if found.load(Ordering::Relaxed) {
            break;
        }

        // Generate all (i, j) pairs such that i + j = d.
        let mut layer = Vec::new();
        let start = if d >= m { d - m + 1 } else { 0 };
        let end = if d < n { d } else { n - 1 };
        for i in start..=end {
            let j = d - i;
            if j < m {
                layer.push((i, j));
            }
        }

        // Process the current diagonal layer in parallel.
        if threads > 0 {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            pool.install(|| process_layer_cpu(&layer, usernames, passwords, &pb, &found, &success_pair));
        } else {
            process_layer_cpu(&layer, usernames, passwords, &pb, &found, &success_pair);
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

/// The main entry point for the library.
/// It reads the input files, prints load information, and kicks off the brute-force search.
pub fn run_bruteforce(usernames_file: &str, passwords_file: &str, threads: usize) -> io::Result<()> {
    let usernames_vec = read_lines_lossy(usernames_file)?;
    let passwords_vec = read_lines_lossy(passwords_file)?;

    // Convert to slices for random access in the search.
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

    diagonal_bruteforce(&usernames, &passwords, threads)
}
