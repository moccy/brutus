# Brutus

**Brutus** is an experimental password brute-forcing tool written in Rust. It uses a diagonal (wavefront) search strategy combined with parallel processing to efficiently explore username and password combinations. This tool supports multiple HTTP-based login strategies (GET, JSON POST, and Form POST) via a flexible strategy pattern. 

Note that this is an early version (v0.1.0) and is still under active development.

## Features

- **Diagonal Brute-Force Algorithm:**  
  Traverses the username–password space diagonally to balance the search order.

- **Parallel Processing:**  
  Uses [rayon](https://crates.io/crates/rayon) to run login attempts concurrently. The number of threads is configurable.

- **Multiple Login Strategies:**  
  - **GET Strategy:** Performs URL token replacement.
  - **JSON POST Strategy:** Sends a JSON body with token replacement.
  - **Form POST Strategy:** Sends form data with token replacement.
  - **Dummy Strategy:** Always fails (useful for testing).

- **Progress Reporting:**  
  Uses [indicatif](https://crates.io/crates/indicatif) to display a progress bar during execution.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable version recommended)
- Cargo (comes with Rust)

## Installation

1. **Clone the repository:**

```sh
git clone https://github.com/yourusername/brutus.git
cd brutus
```

2. Build the project:
```sh
cargo build --release
```

## Usage

Run the brute-forcer from the command line. Here’s an example:

```sh
cargo run --release -- \
  --threads 4 \
  --usernames_file usernames.txt \
  --passwords_file passwords.txt \
  --url "http://example.com/login?user=%user%&pass=%pass%" \
  --body '{"username": "%user%", "password": "%pass%"}' \
  --format json
```


### Command Line Arguments

`--threads` (`-t`): Number of threads to use. Pass `0` to use the Rayon default (number of CPU cores).

`--usernames_file`: Path to a file containing a list of usernames (one per line).

`--passwords_file`: Path to a file containing a list of passwords (one per line).

`--url`: Target URL with `%user%` and `%pass%` tokens for login attempts.

`--body`: Request body template with `%user%` and `%pass%` tokens (optional).

`--format`: Format for the request body (`json` or `form`). Defaults to `json` if omitted.

## Project Structure

```
brutus/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── README.md
└── src
    ├── core.rs         # Contains the core diagonal brute-force logic.
    ├── lib.rs          # Library entry point that ties together file I/O, progress, and processing.
    ├── main.rs         # CLI entry point.
    ├── ui.rs           # UI utilities (e.g., progress bar creation).
    └── strategy
         ├── mod.rs     # Exports the LoginStrategy trait and concrete strategies.
         ├── get.rs     # Implementation of the GET login strategy.
         ├── json.rs    # Implementation of the JSON POST login strategy.
         ├── form.rs    # Implementation of the Form POST login strategy.
         └── dummy.rs   # Implementation of the Dummy login strategy.
```

## Extending the Project

The design follows a modular architecture with clear separation of concerns. To extend or modify the tool:

- *Adding a New Strategy:*
    Create a new file in the src/strategy/ folder (e.g., new_strategy.rs), implement the LoginStrategy trait, and re-export it in mod.rs.

- *Modifying the Core Logic:*
    Changes to the diagonal search algorithm or parallel processing can be made in src/core.rs without affecting UI or strategy modules.

- *Updating UI Components:*
    UI-specific code (like progress bars) is isolated in src/ui.rs.
 
## Ethical Considerations

*WARNING:* Use this tool only in environments where you have explicit authorization to perform brute-force testing. Unauthorized access to systems is illegal.

## License

Dual-licensed under [Apache 2.0](./LICENSE-APACHE) or [MIT](./LICENSE-MIT).