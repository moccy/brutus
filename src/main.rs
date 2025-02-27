use std::collections::VecDeque;
use rayon::prelude::*;

// Simulated function to check username/password
fn attempt_login(username: &str, password: &str) -> bool {
    println!("Trying: {} / {}", username, password);
    // Replace this with actual login logic (return true if successful)
    false
}

// Generate the next wave of coordinates in a table
fn generate_wavefront(size: usize, step: usize) -> Vec<(usize, usize)> {
    let mut wave = Vec::new();
    for i in 0..=step {
        let j = step - i;
        if i < size && j < size {
            wave.push((i, j));
        }
    }
    wave
}

fn wavefront_bruteforce(usernames: &[&str], passwords: &[&str]) {
    let size = usernames.len().max(passwords.len());
    let mut queue = VecDeque::new();
    queue.push_back((0, 0));

    while let Some((x, y)) = queue.pop_front() {
        let username = usernames.get(x);
        let password = passwords.get(y);

        if let (Some(u), Some(p)) = (username, password) {
            if attempt_login(u, p) {
                println!("Success! Username: {}, Password: {}", u, p);
                return; // Stop on success
            }
        }

        // Add the next wavefront layer
        let next_wave = generate_wavefront(size, queue.len() + 1);
        for coord in next_wave {
            if !queue.contains(&coord) {
                queue.push_back(coord);
            }
        }
    }
}

fn main() {
    let usernames = vec!["admin", "user", "test"];
    let passwords = vec!["1234", "password", "admin", "letmein"];

    println!("Starting wavefront brute-force attack...");
    wavefront_bruteforce(&usernames, &passwords);
}