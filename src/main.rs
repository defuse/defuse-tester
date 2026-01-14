//! defuse-tester - Black-box integration test suite for defuse.ca
//!
//! Point this at any URL (production defuse.ca or local dev server)
//! to verify functionality from an external perspective.

use clap::Parser;

#[derive(Parser)]
#[command(name = "defuse-tester")]
#[command(about = "Black-box integration test suite for defuse.ca")]
struct Args {
    /// Base URL to test (e.g., https://defuse.ca or http://localhost:3000)
    #[arg(short, long, default_value = "http://localhost:3000")]
    url: String,

    /// Run only tests matching this filter
    #[arg(short, long)]
    filter: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("defuse-tester");
    println!("Target: {}", args.url);
    println!();

    // TODO: Run test suite

    Ok(())
}
