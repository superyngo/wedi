use anyhow::{anyhow, Result};

// ────────────────────────────────────────────────────────────────
// Clipboard Manager
// ────────────────────────────────────────────────────────────────

pub struct ClipboardManager;

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }

    pub fn set_text(&self, text: &str) -> Result<()> {
        #[cfg(windows)]
        {
            use std::ptr;
            use winapi::um::winbase::*;
            use winapi::um::winuser::*;

            unsafe {
                OpenClipboard(ptr::null_mut());
                EmptyClipboard();

                // Convert UTF-8 string to UTF-16LE for Windows clipboard
                let utf16: Vec<u16> = text.encode_utf16().collect();
                let size = (utf16.len() + 1) * 2; // +1 for null terminator, *2 for u16 size

                let h_mem = GlobalAlloc(GMEM_MOVEABLE, size);
                if h_mem.is_null() {
                    CloseClipboard();
                    return Err(anyhow!("GlobalAlloc failed"));
                }

                let ptr = GlobalLock(h_mem) as *mut u16;
                if ptr.is_null() {
                    GlobalFree(h_mem);
                    CloseClipboard();
                    return Err(anyhow!("GlobalLock failed"));
                }

                // Copy UTF-16 data and add null terminator
                std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
                *ptr.add(utf16.len()) = 0;

                GlobalUnlock(h_mem);

                SetClipboardData(CF_UNICODETEXT, h_mem);
                CloseClipboard();
            }
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            let mut child = std::process::Command::new("pbcopy")
                .stdin(std::process::Stdio::piped())
                .spawn()?;

            if let Some(stdin) = child.stdin.as_mut() {
                std::io::Write::write_all(stdin, text.as_bytes())?;
            }

            child.wait()?;
            Ok(())
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // Try wl-copy first, then xclip
            let result = std::process::Command::new("wl-copy")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    if let Some(stdin) = child.stdin.as_mut() {
                        std::io::Write::write_all(stdin, text.as_bytes())?;
                    }
                    child.wait()
                });

            if result.is_err() {
                // Fallback to xclip
                let mut child = std::process::Command::new("xclip")
                    .args(&["-selection", "clipboard"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()?;

                if let Some(stdin) = child.stdin.as_mut() {
                    std::io::Write::write_all(stdin, text.as_bytes())?;
                }

                child.wait()?;
            }
            Ok(())
        }
    }

    pub fn get_text(&self) -> Result<String> {
        #[cfg(windows)]
        {
            use std::ptr;
            use winapi::um::winbase::*;
            use winapi::um::winuser::*;

            unsafe {
                OpenClipboard(ptr::null_mut());
                let handle = GetClipboardData(CF_UNICODETEXT);

                if handle.is_null() {
                    CloseClipboard();
                    return Ok("".into());
                }

                let ptr = GlobalLock(handle) as *const u16;
                if ptr.is_null() {
                    CloseClipboard();
                    return Err(anyhow!("GlobalLock failed"));
                }

                // Read UTF-16 data until null terminator
                let mut out = Vec::new();
                let mut i = 0;
                loop {
                    let ch = *ptr.add(i);
                    if ch == 0 {
                        break;
                    }
                    out.push(ch);
                    i += 1;
                }

                GlobalUnlock(handle);
                CloseClipboard();

                // Convert UTF-16LE to UTF-8 string
                Ok(String::from_utf16_lossy(&out))
            }
        }

        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("pbpaste").output()?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // Try wl-paste first, then xclip
            let result = std::process::Command::new("wl-paste").output();

            match result {
                Ok(output) => Ok(String::from_utf8_lossy(&output.stdout).to_string()),
                Err(_) => {
                    // Fallback to xclip
                    let output = std::process::Command::new("xclip")
                        .args(&["-selection", "clipboard", "-o"])
                        .output()?;
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                }
            }
        }
    }

    pub fn is_available(&self) -> bool {
        true // 自製實現總是可用的
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize clipboard manager")
    }
}
