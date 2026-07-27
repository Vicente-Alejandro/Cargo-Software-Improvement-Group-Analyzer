use clap::Parser;

/// cargo-sig - Software Improvement Group Analyzer
/// Zero-configuration tool to measure maintainability, duplication, and churn vs. coverage.
#[derive(Parser, Debug)]
#[command(name = "cargo", bin_name = "cargo")]
pub enum CargoCli {
    Sig(SigArgs),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct SigArgs {
    /// Quality gate: Fail if the final rating is below this threshold (1-5)
    #[arg(long, default_value = "0")]
    pub fail_below: u8,

    /// Output format (terminal, json, html)
    #[arg(long, default_value = "terminal")]
    pub format: String,
}
