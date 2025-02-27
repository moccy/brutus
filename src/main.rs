use std::collections::VecDeque;

// Simulated function to check username/password
fn attempt_login(username: &str, password: &str) -> bool {
    println!("Trying: {} / {}", username, password);
    // Replace this with actual login logic (return true if successful)
    false
}

// Expanding wavefront generation
fn wavefront_bruteforce(usernames: &[&str], passwords: &[&str]) {
    let size = usernames.len().max(passwords.len());
    let mut queue = VecDeque::new();

    // Start with the first element (0,0)
    queue.push_back((0, 0));

    let mut step = 1;

    while let Some((x, y)) = queue.pop_front() {
        let username = usernames.get(x);
        let password = passwords.get(y);

        if let (Some(u), Some(p)) = (username, password) {
            print!("({},{}) ", x, y);
            if attempt_login(u, p) {
                println!("Success! Username: {}, Password: {}", u, p);
                return; // Stop on success
            }
        }

        // Generate next wavefront
        for i in 0..=step {
            let j = step - i;
            if i < size && j < size {
                queue.push_back((i, j));
            }
        }

        step += 1; // Expand the wavefront step
    }
}

fn main() {
    let usernames = vec!["admin", "user", "test"];
    let passwords = vec!["1234", "password", "admin", "letmein"];

    println!("Starting wavefront brute-force attack...");
    println!("Input usernames: {:?}", usernames);
    println!("Input passwords: {:?}", passwords);
    wavefront_bruteforce(&usernames, &passwords);
}