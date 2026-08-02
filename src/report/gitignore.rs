use std::fs::{self, OpenOptions};
use std::io::{self, IsTerminal, Write};
use std::path::Path;

pub fn ensure_gitignored(root_dir: &Path) -> io::Result<()> {
    let gitignore_path = root_dir.join(".gitignore");
    if !is_ignored(&gitignore_path)? && io::stdin().is_terminal() {
        prompt_and_add(&gitignore_path)?;
    }
    Ok(())
}

fn is_ignored(gitignore_path: &Path) -> io::Result<bool> {
    if !gitignore_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(gitignore_path)?;
    Ok(content.lines().any(is_sig_line))
}

fn is_sig_line(line: &str) -> bool {
    let t = line.trim();
    t == "tools/cargo-sig"
        || t == "tools/cargo-sig/"
        || t == "/tools/cargo-sig"
        || t == "/tools/cargo-sig/"
        || t == "tools/"
}

fn prompt_and_add(gitignore_path: &Path) -> io::Result<()> {
    print!(
        "\n[cargo-sig] Directory 'tools/cargo-sig/' is not in .gitignore.\nAdd 'tools/cargo-sig/' to .gitignore? [Y/n]: "
    );
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if user_confirmed(&input) {
        append_sig_ignore(gitignore_path)?;
    }
    Ok(())
}

fn user_confirmed(input: &str) -> bool {
    let t = input.trim();
    t.is_empty() || t.eq_ignore_ascii_case("y") || t.eq_ignore_ascii_case("yes")
}

fn append_sig_ignore(gitignore_path: &Path) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(gitignore_path)?;
    writeln!(file, "\n# cargo-sig artifacts\ntools/cargo-sig/")?;
    println!("[cargo-sig] ✅ Added 'tools/cargo-sig/' to .gitignore.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_is_ignored_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".gitignore");
        assert!(!is_ignored(&path).unwrap());
    }

    #[test]
    fn test_is_ignored_when_present() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".gitignore");
        fs::write(&path, "target/\ntools/cargo-sig/\n").unwrap();
        assert!(is_ignored(&path).unwrap());
    }

    #[test]
    fn test_is_ignored_with_leading_slash() {
        let dir = tempdir().unwrap();
        let path = dir.path().join(".gitignore");
        fs::write(&path, "/tools/cargo-sig\n").unwrap();
        assert!(is_ignored(&path).unwrap());
    }

    #[test]
    fn test_user_confirmed() {
        assert!(user_confirmed(""));
        assert!(user_confirmed("y"));
        assert!(user_confirmed("Y"));
        assert!(user_confirmed("yes"));
        assert!(!user_confirmed("n"));
        assert!(!user_confirmed("no"));
    }

    #[test]
    fn test_ensure_gitignored_non_interactive() {
        let dir = tempdir().unwrap();
        let res = ensure_gitignored(dir.path());
        assert!(res.is_ok());
    }
}
