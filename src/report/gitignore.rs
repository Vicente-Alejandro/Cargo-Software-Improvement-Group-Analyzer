use std::fs::{self, OpenOptions};
use std::io::{self, IsTerminal, Write};
use std::path::Path;

pub fn ensure_gitignored(root_dir: &Path) -> io::Result<()> {
    let gitignore_path = root_dir.join(".gitignore");
    if !is_ignored(&gitignore_path)? {
        prompt_and_add(&gitignore_path)?;
    }
    Ok(())
}

fn is_ignored(gitignore_path: &Path) -> io::Result<bool> {
    if !gitignore_path.exists() {
        return Ok(false);
    }
    let content = fs::read_to_string(gitignore_path)?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "tools/cargo-sig"
            || trimmed == "tools/cargo-sig/"
            || trimmed == "/tools/cargo-sig"
            || trimmed == "/tools/cargo-sig/"
            || trimmed == "tools/"
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn prompt_and_add(gitignore_path: &Path) -> io::Result<()> {
    if !io::stdin().is_terminal() {
        return Ok(());
    }

    print!(
        "\n[cargo-sig] Directory 'tools/cargo-sig/' is not in .gitignore.\nAdd 'tools/cargo-sig/' to .gitignore? [Y/n]: "
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("y")
        || trimmed.eq_ignore_ascii_case("yes")
    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(gitignore_path)?;
        writeln!(file, "\n# cargo-sig artifacts\ntools/cargo-sig/")?;
        println!("[cargo-sig] ✅ Added 'tools/cargo-sig/' to .gitignore.");
    }
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
    fn test_ensure_gitignored_non_interactive() {
        let dir = tempdir().unwrap();
        // In test environments stdin is typically non-interactive, so ensure_gitignored should return Ok(()) without blocking
        let res = ensure_gitignored(dir.path());
        assert!(res.is_ok());
    }
}
