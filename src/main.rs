use brutus::run_bruteforce;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of threads to use in parallel (0 uses the Rayon default, which is the number of CPU cores)
    #[arg(short, long, default_value_t = 0)]
    threads: usize,

    /// Path to the file containing usernames
    #[arg(long)]
    usernames_file: String,

    /// Path to the file containing passwords
    #[arg(long)]
    passwords_file: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    run_bruteforce(&args.usernames_file, &args.passwords_file, args.threads)
}
