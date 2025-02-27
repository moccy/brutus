use brutus::run_bruteforce;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of threads to use in parallel (0 uses the Rayon default)
    #[arg(short, long, default_value_t = 0)]
    threads: usize,

    /// Path to the file containing usernames
    #[arg(long)]
    usernames_file: String,

    /// Path to the file containing passwords
    #[arg(long)]
    passwords_file: String,

    /// URL with %user% and %pass% tokens for login attempts (optional)
    #[arg(long)]
    url: Option<String>,

    /// Request body template with %user% and %pass% tokens (optional)
    #[arg(long)]
    body: Option<String>,

    /// Format for the request body: "json" or "form" (optional, defaults to "json")
    #[arg(long)]
    format: Option<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    run_bruteforce(
        &args.usernames_file,
        &args.passwords_file,
        args.threads,
        args.url.as_deref(),
        args.body.as_deref(),
        args.format.as_deref(),
    )
}
