#[derive(Debug)]
pub struct SigArgs {
    pub fail_below: u8,
    #[allow(dead_code)]
    pub format: String,
    pub no_auto_cov: bool,
}

impl SigArgs {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Self {
        let mut sig = Self {
            fail_below: 0,
            format: "terminal".to_string(),
            no_auto_cov: false,
        };
        while let Some(arg) = args.next() {
            sig.apply_arg(&arg, &mut args);
        }
        sig
    }

    fn apply_arg(&mut self, arg: &str, args: &mut impl Iterator<Item = String>) {
        if arg == "--fail-below" {
            self.fail_below = args.next().unwrap_or_default().parse().unwrap_or(0);
        } else if arg == "--format" {
            self.format = args.next().unwrap_or_default();
        } else if arg == "--no-auto-cov" {
            self.no_auto_cov = true;
        } else if arg == "-h" || arg == "--help" {
            Self::print_help();
            std::process::exit(0);
        }
    }

    fn print_help() {
        println!("cargo-sig - Software Improvement Group Analyzer");
        println!("\nUsage: cargo sig [OPTIONS]");
        println!("  --fail-below <1-7>  Fail if rating is below threshold");
        println!("  --format <format>   Output format [default: terminal]");
        println!("  --no-auto-cov       Disable automatic coverage generation");
        println!("  -h, --help          Print help");
    }
}
