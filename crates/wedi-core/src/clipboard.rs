#[cfg(windows)]
use anyhow::anyhow;
use anyhow::Result;

// ────────────────────────────────────────────────────────────────
// Clipboard Manager
// ────────────────────────────────────────────────────────────────

#[allow(dead_code)]
pub struct ClipboardManager;

#[allow(dead_code)]
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
                if OpenClipboard(ptr::null_mut()) == 0 {
                    return Err(anyhow!("OpenClipboard failed"));
                }
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

                // 成功時記憶體歸系統所有；失敗時需自行釋放
                if SetClipboardData(CF_UNICODETEXT, h_mem).is_null() {
                    GlobalFree(h_mem);
                    CloseClipboard();
                    return Err(anyhow!("SetClipboardData failed"));
                }
                CloseClipboard();
            }
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            pipe_to("pbcopy", &[], text)
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // 先試 wl-copy，再試 xclip；兩者都失敗（如 SSH 無顯示伺服器）時回傳錯誤
            pipe_to("wl-copy", &[], text)
                .or_else(|_| pipe_to("xclip", &["-selection", "clipboard"], text))
        }
    }

    pub fn get_text(&self) -> Result<String> {
        #[cfg(windows)]
        {
            use std::ptr;
            use winapi::um::winbase::*;
            use winapi::um::winuser::*;

            unsafe {
                if OpenClipboard(ptr::null_mut()) == 0 {
                    return Err(anyhow!("OpenClipboard failed"));
                }
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
            read_from("pbpaste", &[])
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // 先試 wl-paste，再試 xclip；結束碼非 0 視為失敗
            read_from("wl-paste", &["--no-newline"])
                .or_else(|_| read_from("xclip", &["-selection", "clipboard", "-o"]))
        }
    }

    pub fn is_available(&self) -> bool {
        true // 自製實現總是可用的
    }
}

/// 將文字寫入外部剪貼簿程式的 stdin；程式不存在或結束碼非 0 時回傳錯誤
#[cfg(unix)]
fn pipe_to(program: &str, args: &[&str], text: &str) -> Result<()> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    } // stdin 在此關閉，程式才會結束
    let status = child.wait()?;
    anyhow::ensure!(status.success(), "{program} exited with {status}");
    Ok(())
}

/// 讀取外部剪貼簿程式的輸出；程式不存在或結束碼非 0 時回傳錯誤
#[cfg(unix)]
fn read_from(program: &str, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .stderr(std::process::Stdio::null())
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "{program} exited with {}",
        output.status
    );
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize clipboard manager")
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_command_failures_are_errors() {
        // 稽核 F19：結束碼非 0（如 SSH 下 xclip 無 DISPLAY）曾被當成成功、貼上空字串
        assert!(pipe_to("false", &[], "x").is_err());
        assert!(read_from("false", &[]).is_err());
        assert!(pipe_to("wedi-no-such-program", &[], "x").is_err());
        assert!(pipe_to("cat", &[], "x").is_ok());
        assert_eq!(read_from("printf", &["ab"]).unwrap(), "ab");
    }
}
