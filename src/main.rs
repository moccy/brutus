//! Main executable entry point.

use anyhow::Result;
use brutus::run_bruteforce;
use clap::Parser;
use env_logger::Env;

/// Command-line arguments.
#[derive(Parser, Debug)]
#[command(author, version, about = "A high-performance password brute-forcer", long_about = None)]
struct Args {
    /// Number of threads to use in parallel (0 uses the Rayon default)
    #[arg(short, long, default_value_t = 0)]
    threads: usize,

    /// Path to the file containing usernames
    #[arg(long, help = "Path to the usernames file")]
    usernames_file: String,

    /// Path to the file containing passwords
    #[arg(long, help = "Path to the passwords file")]
    passwords_file: String,

    /// URL with %user% and %pass% tokens for login attempts (optional)
    #[arg(long, help = "Target URL with tokens %user% and %pass%")]
    url: Option<String>,

    /// Request body template with %user% and %pass% tokens (optional)
    #[arg(
        long,
        help = "Body template for POST requests with tokens %user% and %pass%"
    )]
    body: Option<String>,

    /// Format for the request body: "json" or "form" (optional, defaults to json)
    #[arg(long, help = "Format for the request body: json or form")]
    format: Option<String>,
}

fn main() -> Result<()> {
    // Initialize structured logging with env_logger.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let args = Args::parse();
    run_bruteforce(
        &args.usernames_file,
        &args.passwords_file,
        args.threads,
        args.url.as_deref(),
        args.body.as_deref(),
        args.format.as_deref(),
    )?;
    Ok(())
}
