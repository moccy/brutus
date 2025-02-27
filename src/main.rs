use std::collections::VecDeque;

// Simulated function to check username/password
fn attempt_login(username: &str, password: &str) -> bool {
    println!("{}:{}", username, password);
    false // Replace with actual login logic
}

// BFS-style wavefront brute-force attack
fn wavefront_bruteforce(usernames: &[&str], passwords: &[&str]) {
    let size = usernames.len().max(passwords.len());
    let mut queue = VecDeque::new();
    
    // Start at (0,0)
    queue.push_back((0, 0));

    // Track visited positions
    let mut visited = vec![vec![false; size]; size];
    visited[0][0] = true;

    while let Some((x, y)) = queue.pop_front() {
        let username = usernames.get(x);
        let password = passwords.get(y);

        if let (Some(u), Some(p)) = (username, password) {
            print!("Trying ({},{}): ", x, y);
            if attempt_login(u, p) {
                println!("Success! Username: {}, Password: {}", u, p);
                return; // Stop on success
            }
        }

        // Expand to next layer (wavefront)
        let neighbors = [
            (x + 1, y), // Right
            (x, y + 1), // Down
            (x + 1, y + 1), // Diagonal
        ];

        for &(nx, ny) in &neighbors {
            if nx < size && ny < size && !visited[nx][ny] {
                queue.push_back((nx, ny));
                visited[nx][ny] = true;
            }
        }
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
