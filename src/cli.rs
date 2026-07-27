/// cargo-sig - Software Improvement Group Analyzer
/// Zero-configuration tool to measure maintainability, duplication, and churn vs. coverage.

#[derive(Debug)]
pub struct SigArgs {
    pub fail_below: u8,
    pub format: String,
}

impl SigArgs {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Self {
        let mut fail_below = 0;
        let mut format = "terminal".to_string();

        while let Some(arg) = args.next() {
            if arg == "--fail-below" {
                if let Some(val) = args.next() {
                    fail_below = val.parse().unwrap_or(0);
                }
            } else if arg == "--format" {
                if let Some(val) = args.next() {
                    format = val;
                }
            } else if arg == "--help" || arg == "-h" {
                Self::print_help();
                std::process::exit(0);
            }
        }
        
        Self { fail_below, format }
    }

    fn print_help() {
        println!("cargo-sig - Software Improvement Group Analyzer");
        println!("Zero-configuration tool to measure maintainability, duplication, and churn vs. coverage.");
        println!("\nUsage: cargo sig [OPTIONS]");
        println!("\nOptions:");
        println!("  --fail-below <1-7>  Quality gate: Fail if the final rating is below this threshold");
        println!("  --format <format>   Output format (terminal, json, html) [default: terminal]");
        println!("  -h, --help          Print help");
    }
}
