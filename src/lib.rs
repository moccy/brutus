// src/lib.rs

//! Library entry point for the brute-force tool.

pub mod core;
pub mod ui;
pub mod strategy;

use anyhow::Result;
use log::info;
use strategy::{DummyStrategy, FormStrategy, GetStrategy, JsonStrategy, LoginStrategy};
use std::io;
use url::Url;
use crate::core::diagonal_bruteforce_core;
use crate::ui::create_progress_bar;

/// Reads a file and returns its lines as a vector of strings with lossy UTF-8 conversion.
pub fn read_lines_lossy(filename: &str) -> io::Result<Vec<String>> {
    use memmap2::Mmap;
    use std::fs::File;
    let file = File::open(filename)?;
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(String::from_utf8_lossy(&mmap).lines().map(|s| s.to_string()).collect())
}

/// Runs the brute-force search. This function is responsible for handling UI aspects
/// (such as file I/O, progress bar updates, and logging) and then calling the core processing.
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
    info!("Loaded {} username(s) from '{}'", usernames.len(), usernames_file);
    info!("Loaded {} password(s) from '{}'", passwords.len(), passwords_file);
    if threads == 0 {
        info!("Using Rayon default thread count (# of CPU cores).");
    } else {
        info!("Using {} thread(s).", threads);
    }

    // Validate URL and select the appropriate login strategy.
    let strategy: Box<dyn LoginStrategy> = if let Some(url) = target_url {
        if Url::parse(url).is_err() {
            anyhow::bail!("Invalid URL provided: {}", url);
        }
        if let Some(body) = target_body {
            match target_format {
                Some("form") => Box::new(FormStrategy::new(url, body)),
                _ => Box::new(JsonStrategy::new(url, body)),
            }
        } else {
            Box::new(GetStrategy::new(url))
        }
    } else {
        Box::new(DummyStrategy::new())
    };

    let total_attempts = (usernames.len() as u64) * (passwords.len() as u64);
    let pb = create_progress_bar(total_attempts);
    let progress_update = || {
        pb.inc(1);
    };

    let result = diagonal_bruteforce_core(&usernames, &passwords, threads, &*strategy, &progress_update);
    pb.finish_and_clear();

    if let Some((x, y)) = result {
        info!("SUCCESS FOUND! Username: '{}', Password: '{}'", usernames[x], passwords[y]);
    } else {
        info!("No success found after exhausting the search space.");
    }
    Ok(())
}
