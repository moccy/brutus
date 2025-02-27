// src/core.rs

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use rayon::prelude::*;

use crate::strategy::LoginStrategy;

/// Builds a diagonal layer (or wavefront) for the given index `d`.
fn build_layer(d: usize, n: usize, m: usize) -> Vec<(usize, usize)> {
    let start = if d >= m { d - m + 1 } else { 0 };
    let end = if d < n { d } else { n - 1 };
    (start..=end)
        .filter_map(|i| {
            let j = d - i;
            if j < m { Some((i, j)) } else { None }
        })
        .collect()
}

/// The core brute-force function. It iterates over the search space in diagonal layers,
/// calls the provided `progress_update` callback on every attempt, and returns the first
/// successful (username, password) index pair if found.
pub fn diagonal_bruteforce_core(
    usernames: &[&str],
    passwords: &[&str],
    threads: usize,
    login_attempt: &dyn LoginStrategy,
    progress_update: &(dyn Fn() + Sync),
) -> Option<(usize, usize)> {
    let n = usernames.len();
    let m = passwords.len();

    if n == 0 || m == 0 {
        return None;
    }

    let found = AtomicBool::new(false);
    let result = Mutex::new(None);

    let maybe_pool = if threads > 0 {
        Some(
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap(),
        )
    } else {
        None
    };

    for d in 0..=(n + m - 2) {
        if found.load(Ordering::Relaxed) {
            break;
        }
        let layer = build_layer(d, n, m);
        if let Some(ref pool) = maybe_pool {
            pool.install(|| {
                layer.par_iter().for_each(|&(x, y)| {
                    if found.load(Ordering::Relaxed) {
                        return;
                    }
                    progress_update();
                    if login_attempt.attempt(usernames[x], passwords[y]) {
                        found.store(true, Ordering::Relaxed);
                        let mut lock = result.lock().unwrap();
                        *lock = Some((x, y));
                    }
                });
            });
        } else {
            for &(x, y) in &layer {
                progress_update();
                if login_attempt.attempt(usernames[x], passwords[y]) {
                    found.store(true, Ordering::Relaxed);
                    let mut lock = result.lock().unwrap();
                    *lock = Some((x, y));
                    break;
                }
            }
        }
    }

    let lock = result.lock().unwrap();
    *lock
}
