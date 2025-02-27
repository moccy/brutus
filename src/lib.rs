//! Core brute-force functionality.
//!
//! This module provides functions for reading input files, running the diagonal
//! brute-force search, and integrating a login strategy.

mod strategy;
pub use strategy::*;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use url::Url;

/// Reads a file using memory mapping and returns a vector of lines with lossy UTF-8 conversion.
///
/// # Errors
/// Returns an error if the file cannot be opened or mapped.
pub fn read_lines_lossy(filename: &str) -> io::Result<Vec<String>> {
    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let content = String::from_utf8_lossy(&mmap);
    let lines = content.lines().map(|line| line.to_string()).collect();
    Ok(lines)
}

/// Processes a single diagonal (wavefront) layer in parallel, using the provided login strategy.
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

/// Runs the diagonal brute-force search without building a huge visited matrix.
///
/// # Errors
/// Returns an error if any IO operation fails.
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
        ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
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
        info!(
            "SUCCESS FOUND! Username: '{}', Password: '{}'",
            usernames[x], passwords[y]
        );
    } else {
        info!("No success found after exhausting the search space.");
    }
    Ok(())
}

/// Runs the brute-force search.
///
/// If a target URL is provided, the URL is validated. If a body template is provided,
/// then the format option is used to select the appropriate POST strategy ("json" or "form").
/// If no target URL is provided, a dummy strategy is used.
///
/// # Errors
/// Returns an error if input files cannot be read or the URL is invalid.
pub fn run_bruteforce(
    usernames_file: &str,
    passwords_file: &str,
    threads: usize,
    target_url: Option<&str>,
    target_body: Option<&str>,
    target_format: Option<&str>,
) -> Result<()> {
    let usernames_vec = read_lines_lossy(usernames_file)?;
    let passwords_vec = read_lines_lossy(passwords_file)?;

    let usernames: Vec<&str> = usernames_vec.iter().map(|s| s.as_str()).collect();
    let passwords: Vec<&str> = passwords_vec.iter().map(|s| s.as_str()).collect();

    info!("Starting diagonal brute-force with UTF-8 normalization...");
    info!(
        "Loaded {} username(s) from '{}'",
        usernames.len(),
        usernames_file
    );
    info!(
        "Loaded {} password(s) from '{}'",
        passwords.len(),
        passwords_file
    );
    if threads == 0 {
        info!("Using Rayon default thread count (# of CPU cores).");
    } else {
        info!("Using {} thread(s).", threads);
    }

    // Validate URL if provided.
    let strategy: Box<dyn LoginStrategy> = if let Some(url) = target_url {
        if Url::parse(url).is_err() {
            anyhow::bail!("Invalid URL provided: {}", url);
        }
        if let Some(body) = target_body {
            match target_format {
                Some("form") => Box::new(FormStrategy::new(url, body)),
                Some("json") | _ => Box::new(JsonStrategy::new(url, body)),
            }
        } else {
            Box::new(GetStrategy::new(url))
        }
    } else {
        Box::new(DummyStrategy::new())
    };

    diagonal_bruteforce(&usernames, &passwords, threads, &*strategy)?;
    Ok(())
}
