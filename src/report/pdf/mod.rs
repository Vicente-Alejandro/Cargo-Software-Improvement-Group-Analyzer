use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub enum PdfError {
    BrowserNotFound,
    CommandFailed(i32),
    Io(std::io::Error),
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BrowserNotFound => write!(
                f,
                "No headless browser (Edge, Chrome, Chromium) detected on the system."
            ),
            Self::CommandFailed(code) => {
                write!(f, "Headless browser process exited with status code {code}")
            }
            Self::Io(e) => write!(f, "I/O error executing headless browser: {e}"),
        }
    }
}

impl std::error::Error for PdfError {}

/// Generates a PDF report from the HTML report using an available headless browser.
pub fn generate_pdf_report(html_path: &Path, root_dir: &Path) -> Result<PathBuf, PdfError> {
    let out_dir = root_dir.join("tools").join("cargo-sig");
    if !out_dir.exists() {
        let _ = std::fs::create_dir_all(&out_dir);
    }
    let pdf_path = out_dir.join("SIG_REPORT.pdf");

    let browser = find_browser().ok_or(PdfError::BrowserNotFound)?;
    let html_url = to_file_url(html_path);

    let status = Command::new(&browser)
        .arg("--headless=new")
        .arg("--disable-gpu")
        .arg("--no-pdf-header-footer")
        .arg(format!("--print-to-pdf={}", pdf_path.display()))
        .arg(&html_url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();

    match status {
        Ok(s) if s.success() && pdf_path.exists() => Ok(pdf_path),
        Ok(_) => {
            // Fallback to legacy --headless flag for older Chromium / Edge versions
            let retry = Command::new(&browser)
                .arg("--headless")
                .arg("--disable-gpu")
                .arg("--no-pdf-header-footer")
                .arg(format!("--print-to-pdf={}", pdf_path.display()))
                .arg(&html_url)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            match retry {
                Ok(s) if s.success() && pdf_path.exists() => Ok(pdf_path),
                Ok(s) => Err(PdfError::CommandFailed(s.code().unwrap_or(-1))),
                Err(e) => Err(PdfError::Io(e)),
            }
        }
        Err(e) => Err(PdfError::Io(e)),
    }
}

#[allow(dead_code)]
pub fn has_headless_browser() -> bool {
    find_browser().is_some()
}

pub fn find_browser() -> Option<PathBuf> {
    for c in get_browser_candidates() {
        if c.is_file() {
            return Some(c);
        }
        if let Ok(path) = which_in_path(&c) {
            return Some(path);
        }
    }
    None
}

fn to_file_url(path: &Path) -> String {
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_default()
            .join(path)
    };
    let path_str = abs_path.to_string_lossy().replace('\\', "/");
    let clean = path_str.trim_start_matches('/');
    format!("file:///{clean}")
}

fn get_browser_candidates() -> Vec<PathBuf> {
    let mut list = Vec::new();
    #[cfg(target_os = "windows")]
    {
        list.push(PathBuf::from(
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        ));
        list.push(PathBuf::from(
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
        ));
        list.push(PathBuf::from(
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        ));
        list.push(PathBuf::from(
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ));
        list.push(PathBuf::from(
            r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe",
        ));
        list.push(PathBuf::from("msedge.exe"));
        list.push(PathBuf::from("chrome.exe"));
    }
    #[cfg(target_os = "macos")]
    {
        list.push(PathBuf::from(
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        ));
        list.push(PathBuf::from(
            "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        ));
        list.push(PathBuf::from(
            "/Applications/Brave Browser.app/Contents/MacOS/Brave Browser",
        ));
        list.push(PathBuf::from(
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ));
        list.push(PathBuf::from("google-chrome"));
        list.push(PathBuf::from("chromium"));
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        list.push(PathBuf::from("google-chrome"));
        list.push(PathBuf::from("google-chrome-stable"));
        list.push(PathBuf::from("chromium"));
        list.push(PathBuf::from("chromium-browser"));
        list.push(PathBuf::from("microsoft-edge"));
        list.push(PathBuf::from("brave-browser"));
    }
    list
}

fn which_in_path(cmd: &Path) -> Result<PathBuf, ()> {
    let Some(cmd_str) = cmd.to_str() else {
        return Err(());
    };
    if cmd_str.contains('/') || cmd_str.contains('\\') {
        return Err(());
    }

    #[cfg(target_os = "windows")]
    let check = Command::new("where").arg(cmd_str).output();
    #[cfg(not(target_os = "windows"))]
    let check = Command::new("which").arg(cmd_str).output();

    if let Ok(out) = check {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some(first_line) = s.lines().next() {
                let p = PathBuf::from(first_line.trim());
                if p.exists() {
                    return Ok(p);
                }
            }
        }
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_file_url() {
        let p = Path::new("tools/cargo-sig/SIG_REPORT.html");
        let url = to_file_url(p);
        assert!(url.starts_with("file:///"));
        assert!(url.contains("SIG_REPORT.html"));
    }

    #[test]
    fn test_find_browser_or_candidates() {
        let candidates = get_browser_candidates();
        assert!(!candidates.is_empty());
        let _ = find_browser();
        let _ = has_headless_browser();
    }
}
