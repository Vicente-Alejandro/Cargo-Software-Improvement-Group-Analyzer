#[derive(Debug)]
pub struct SigArgs {
    pub fail_below: u8,
    #[allow(dead_code)]
    pub format: String,
    pub auto_cov: bool,
}

impl SigArgs {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Self {
        let mut sig = Self {
            fail_below: 0,
            format: "terminal".to_string(),
            auto_cov: false,
        };
        while let Some(arg) = args.next() {
            sig.apply_arg(&arg, &mut args);
        }
        sig
    }

    #[rustfmt::skip]
    fn apply_arg(&mut self, arg: &str, args: &mut impl Iterator<Item = String>) {
        if arg == "--fail-below" { self.fail_below = args.next().unwrap_or_default().parse().unwrap_or(0); return; }
        if arg == "--format" { self.format = args.next().unwrap_or_default(); return; }
        if arg == "-a" || arg == "--auto-cov" { self.auto_cov = true; return; }
        if arg == "-h" || arg == "--help" { Self::print_help(); std::process::exit(0); }
    }

    fn print_help() {
        println!("cargo-sig - Software Improvement Group Analyzer");
        println!("\nUsage: cargo sig [OPTIONS]");
        println!("  -a, --auto-cov      Enable automatic coverage generation");
        println!("  --fail-below <1-7>  Fail if rating is below threshold");
        println!("  --format <format>   Output format [default: terminal]");
        println!("  -h, --help          Print help");
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
    fn test_parse_auto_cov() {
        let args = vec!["-a".to_string()];
        let sig = SigArgs::parse(args.into_iter());
        assert!(sig.auto_cov);

        let args2 = vec!["--auto-cov".to_string()];
        let sig2 = SigArgs::parse(args2.into_iter());
        assert!(sig2.auto_cov);
    }
}
