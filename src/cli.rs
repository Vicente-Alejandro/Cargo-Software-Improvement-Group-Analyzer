use owo_colors::OwoColorize;

#[derive(Debug)]
pub struct SigArgs {
    pub fail_below: u8,
    #[allow(dead_code)]
    pub format: String,
    pub auto_cov: bool,
    pub report: bool,
}

impl SigArgs {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Self {
        let mut sig = Self::default();
        while let Some(arg) = args.next() {
            sig.apply_arg(&arg, &mut args);
        }
        sig
    }

    fn default() -> Self {
        Self {
            fail_below: 0,
            format: "terminal".to_string(),
            auto_cov: false,
            report: false,
        }
    }

    fn apply_arg(&mut self, arg: &str, args: &mut impl Iterator<Item = String>) {
        if !self.apply_flag(arg) {
            self.apply_val(arg, args);
        }
    }

    fn apply_flag(&mut self, arg: &str) -> bool {
        match arg {
            "-r" | "--report" => {
                self.report = true;
                true
            }
            "-a" | "--auto-cov" => {
                self.auto_cov = true;
                true
            }
            "-h" | "--help" => Self::exit_with_help(),
            _ => false,
        }
    }

    fn apply_val(&mut self, arg: &str, args: &mut impl Iterator<Item = String>) {
        if arg == "--fail-below" {
            self.fail_below = args.next().unwrap_or_default().parse().unwrap_or(0);
        } else if arg == "--format" {
            self.format = args.next().unwrap_or_default();
        }
    }

    fn exit_with_help() -> ! {
        Self::print_help();
        std::process::exit(0);
    }

    pub fn print_help() {
        Self::print_header();
        Self::print_usage();
        Self::print_options();
    }

    fn print_header() {
        println!(
            "{} - Software Improvement Group Analyzer\n",
            "cargo-sig".bold().cyan()
        );
        println!("Scores your Rust project's maintainability against SIG's 10 Guidelines,");
        println!("cross-referencing complexity with Git Churn and Test Coverage.\n");
    }

    fn print_usage() {
        println!("{}", "Usage:".bold());
        println!("  cargo sig [OPTIONS]\n");
    }

    #[rustfmt::skip]
    fn print_options() {
        println!("{}", "Options:".bold());
        println!("  {}     Enable automatic test coverage generation via cargo-llvm-cov", "-a, --auto-cov".green());
        println!("  {} Fail if the final rating drops below this threshold (e.g., for CI)", "--fail-below <1-7>".green());
        println!("  {}  Output format: 'terminal' or 'json' [default: terminal]", "--format <format>".green());
        println!("  {}     Generate a full Markdown report (tools/cargo-sig/SIG_REPORT.md)", "-r, --report".green());
        println!("  {}         Print this help message\n", "-h, --help".green());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_defaults() {
        let args: Vec<String> = vec![];
        let sig = SigArgs::parse(args.into_iter());
        assert_eq!(sig.fail_below, 0);
        assert_eq!(sig.format, "terminal");
        assert!(!sig.auto_cov);
        assert!(!sig.report);
    }

    #[test]
    fn test_parse_fail_below() {
        let args = vec!["--fail-below".to_string(), "5".to_string()];
        let sig = SigArgs::parse(args.into_iter());
        assert_eq!(sig.fail_below, 5);

        let args_err = vec!["--fail-below".to_string(), "invalid".to_string()];
        let sig_err = SigArgs::parse(args_err.into_iter());
        assert_eq!(sig_err.fail_below, 0);
    }

    #[test]
    fn test_parse_format() {
        let args = vec!["--format".to_string(), "json".to_string()];
        let parsed = SigArgs::parse(args.into_iter());
        assert_eq!(parsed.format, "json");
    }

    #[test]
    fn test_print_help() {
        SigArgs::print_help();
    }

    #[test]
    fn test_parse_flags() {
        let args = vec!["-a".to_string(), "-r".to_string()];
        let sig = SigArgs::parse(args.into_iter());
        assert!(sig.auto_cov);
        assert!(sig.report);

        let args2 = vec!["--auto-cov".to_string(), "--report".to_string()];
        let sig2 = SigArgs::parse(args2.into_iter());
        assert!(sig2.auto_cov);
        assert!(sig2.report);
    }
}
