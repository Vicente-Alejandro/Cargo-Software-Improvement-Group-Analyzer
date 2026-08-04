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
            Self::BrowserNotFound => {
                write!(f, "No headless browser (Edge, Chrome, Chromium) detected.")
            }
            Self::CommandFailed(code) => {
                write!(f, "Headless browser process exited with code {code}")
            }
            Self::Io(e) => write!(f, "I/O error executing headless browser: {e}"),
        }
    }
}

impl std::error::Error for PdfError {}

fn prepare_pdf_path(root_dir: &Path) -> PathBuf {
    let out_dir = root_dir.join("tools").join("cargo-sig");
    if !out_dir.exists() {
        let _ = std::fs::create_dir_all(&out_dir);
    }
    out_dir.join("SIG_REPORT.pdf")
}

#[rustfmt::skip]
fn run_browser_print(browser: &Path, html_url: &str, pdf_path: &Path, flag: &str) -> Result<(), PdfError> {
    let status = Command::new(browser).arg(flag).arg("--disable-gpu").arg("--no-pdf-header-footer")
        .arg(format!("--print-to-pdf={}", pdf_path.display())).arg(html_url)
        .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).status();
    match status {
        Ok(s) if s.success() && pdf_path.exists() => Ok(()),
        Ok(s) => Err(PdfError::CommandFailed(s.code().unwrap_or(-1))),
        Err(e) => Err(PdfError::Io(e)),
    }
}

/// Generates a PDF report from the HTML report using an available headless browser.
pub fn generate_pdf_report(html_path: &Path, root_dir: &Path) -> Result<PathBuf, PdfError> {
    let pdf_path = prepare_pdf_path(root_dir);
    let browser = find_browser().ok_or(PdfError::BrowserNotFound)?;
    let html_url = to_file_url(html_path);
    if run_browser_print(&browser, &html_url, &pdf_path, "--headless=new").is_ok() {
        return Ok(pdf_path);
    }
    run_browser_print(&browser, &html_url, &pdf_path, "--headless")?;
    Ok(pdf_path)
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
        std::env::current_dir().unwrap_or_default().join(path)
    };
    let path_str = abs_path.to_string_lossy().replace('\\', "/");
    let clean = path_str.trim_start_matches('/');
    format!("file:///{clean}")
}

#[cfg(target_os = "windows")]
fn get_browser_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from(r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"),
        PathBuf::from(r"C:\Program Files\Microsoft\Edge\Application\msedge.exe"),
        PathBuf::from(r"C:\Program Files\Google\Chrome\Application\chrome.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"),
        PathBuf::from(r"C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe"),
        PathBuf::from("msedge.exe"),
        PathBuf::from("chrome.exe"),
    ]
}

#[cfg(target_os = "macos")]
fn get_browser_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
        PathBuf::from("/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge"),
        PathBuf::from("/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"),
        PathBuf::from("/Applications/Chromium.app/Contents/MacOS/Chromium"),
        PathBuf::from("google-chrome"),
        PathBuf::from("chromium"),
    ]
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn get_browser_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from("google-chrome"),
        PathBuf::from("google-chrome-stable"),
        PathBuf::from("chromium"),
        PathBuf::from("chromium-browser"),
        PathBuf::from("microsoft-edge"),
        PathBuf::from("brave-browser"),
    ]
}

#[rustfmt::skip]
fn which_in_path(cmd: &Path) -> Result<PathBuf, ()> {
    let cmd_str = cmd.to_str().filter(|s| !s.contains('/') && !s.contains('\\')).ok_or(())?;
    #[cfg(target_os = "windows")]
    let check = Command::new("where").arg(cmd_str).output();
    #[cfg(not(target_os = "windows"))]
    let check = Command::new("which").arg(cmd_str).output();
    let out = check.map_err(|_| ())?;
    if !out.status.success() { return Err(()); }
    let s = String::from_utf8_lossy(&out.stdout);
    s.lines().next().map(|l| PathBuf::from(l.trim())).filter(|p| p.exists()).ok_or(())
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
