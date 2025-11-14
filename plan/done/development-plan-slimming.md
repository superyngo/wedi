## Phase 6: 超小型化開發計畫 (Week 11-12) 🧭

### 6.1 目標與背景

**🎯 目標**
- Binary 壓縮到 300–600 KB
- 全平台（Windows / Linux / macOS）完整功能保留
- 提升可維護性，移除重型依賴

**📊 當前狀態**
- 現有二進制大小: ~1.7 MB
- 主要依賴: arboard (剪貼板), clap (CLI), env_logger (日誌)
- 預估壓縮潛力: 900-1500 KB

### 6.2 最小依賴跨平台 Clipboard 實現

**🔧 STEP 1 — 移除 arboard，自製最小 clipboard 模組**

**實現策略**:
- **Windows**: 使用 Win32 API (已存在 windows crate)
  - `OpenClipboard` / `GlobalAlloc` / `SetClipboardData` (文本)
  - `GetClipboardData` / `CloseClipboard` (讀取)
- **Linux**: 依序偵測外部工具 (無 Rust crate)
  - 優先: `wl-copy` (Wayland)
  - 備用: `xclip` (X11)
  - 實現: `std::process::Command` 調用
- **macOS**: 使用系統工具 (無 Rust crate)
  - `pbcopy` (寫入剪貼板)
  - `pbpaste` (讀取剪貼板)
  - 實現: `std::process::Command` 調用

**代碼結構**:
```rust
use anyhow::{Result, anyhow};
use std::io::Write;

// ────────────────────────────────────────────────────────────────
// Clipboard Backend Enum
// ────────────────────────────────────────────────────────────────

pub struct ClipboardManager {
    backend: ClipboardBackend,
}

pub enum ClipboardBackend {
    Windows(WindowsClipboard),
    Linux(LinuxClipboard),
    MacOS(MacOSClipboard),
}

impl ClipboardManager {
    pub fn new() -> Result<Self> {
        let backend = if cfg!(windows) {
            ClipboardBackend::Windows(WindowsClipboard::new()?)
        } else if cfg!(target_os = "macos") {
            ClipboardBackend::MacOS(MacOSClipboard::new())
        } else {
            ClipboardBackend::Linux(LinuxClipboard::new()?)
        };

        Ok(Self { backend })
    }

    pub fn set_text(&self, text: &str) -> Result<()> {
        match &self.backend {
            ClipboardBackend::Windows(b) => b.set_text(text),
            ClipboardBackend::Linux(b) => b.set_text(text),
            ClipboardBackend::MacOS(b) => b.set_text(text),
        }
    }

    pub fn get_text(&self) -> Result<String> {
        match &self.backend {
            ClipboardBackend::Windows(b) => b.get_text(),
            ClipboardBackend::Linux(b) => b.get_text(),
            ClipboardBackend::MacOS(b) => b.get_text(),
        }
    }
}

// ────────────────────────────────────────────────────────────────
// Windows Clipboard
// ────────────────────────────────────────────────────────────────

#[cfg(windows)]
struct WindowsClipboard;

#[cfg(windows)]
impl WindowsClipboard {
    fn new() -> Result<Self> {
        Ok(Self)
    }

    fn set_text(&self, text: &str) -> Result<()> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::DataExchange::*;
        use windows::Win32::System::Memory::*;

        unsafe {
            OpenClipboard(HWND(0))?;
            EmptyClipboard();

            let size = text.len() + 1;
            let h_mem = GlobalAlloc(GMEM_MOVEABLE, size);
            if h_mem.0.is_null() {
                CloseClipboard();
                return Err(anyhow!("GlobalAlloc failed"));
            }

            let ptr = GlobalLock(h_mem) as *mut u8;
            if ptr.is_null() {
                GlobalFree(h_mem);
                CloseClipboard();
                return Err(anyhow!("GlobalLock failed"));
            }

            std::ptr::copy_nonoverlapping(text.as_ptr(), ptr, size - 1);
            *ptr.add(size - 1) = 0;

            GlobalUnlock(h_mem);

            SetClipboardData(CF_TEXT.0 as u32, HANDLE(h_mem.0))?;
            CloseClipboard();
        }

        Ok(())
    }

    fn get_text(&self) -> Result<String> {
        use windows::Win32::Foundation::*;
        use windows::Win32::System::DataExchange::*;
        use windows::Win32::System::Memory::*;

        unsafe {
            OpenClipboard(HWND(0))?;
            let handle = GetClipboardData(CF_TEXT.0 as u32);

            if handle.0.is_null() {
                CloseClipboard();
                return Ok("".into());
            }

            let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u8;
            if ptr.is_null() {
                CloseClipboard();
                return Err(anyhow!("GlobalLock failed"));
            }

            let mut out = Vec::new();
            let mut i = 0;
            loop {
                let b = *ptr.add(i);
                if b == 0 { break; }
                out.push(b);
                i += 1;
            }

            GlobalUnlock(HGLOBAL(handle.0));
            CloseClipboard();
            Ok(String::from_utf8_lossy(&out).to_string())
        }
    }
}

// ────────────────────────────────────────────────────────────────
// Linux Clipboard (wl-copy / xclip)
// ────────────────────────────────────────────────────────────────

#[cfg(all(unix, not(target_os = "macos")))]
struct LinuxClipboard {
    use_wl_copy: bool,
}

#[cfg(all(unix, not(target_os = "macos")))]
impl LinuxClipboard {
    fn new() -> Result<Self> {
        let use_wl_copy = std::process::Command::new("wl-copy").output().is_ok();

        if !use_wl_copy {
            // check xclip availability
            std::process::Command::new("xclip")
                .arg("-version")
                .output()
                .map_err(|_| anyhow!("No clipboard tool available (install wl-copy or xclip)"))?;
        }

        Ok(Self { use_wl_copy })
    }

    fn set_text(&self, text: &str) -> Result<()> {
        let cmd = if self.use_wl_copy { "wl-copy" } else { "xclip" };

        let mut command = std::process::Command::new(cmd);
        if !self.use_wl_copy {
            command.args(&["-selection", "clipboard"]);
        }

        let mut child = command.stdin(std::process::Stdio::piped()).spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }

        child.wait()?;
        Ok(())
    }

    fn get_text(&self) -> Result<String> {
        let cmd = if self.use_wl_copy { "wl-paste" } else { "xclip" };

        let mut command = std::process::Command::new(cmd);

        if !self.use_wl_copy {
            command.args(&["-selection", "clipboard", "-o"]);
        }

        let output = command.output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

// ────────────────────────────────────────────────────────────────
// macOS Clipboard (pbcopy/pbpaste)
// ────────────────────────────────────────────────────────────────

#[cfg(target_os = "macos")]
struct MacOSClipboard;

#[cfg(target_os = "macos")]
impl MacOSClipboard {
    fn new() -> Self {
        Self
    }

    fn set_text(&self, text: &str) -> Result<()> {
        let mut child = std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(text.as_bytes())?;
        }

        child.wait()?;
        Ok(())
    }

    fn get_text(&self) -> Result<String> {
        let output = std::process::Command::new("pbpaste").output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
```

**測試要點**:
- Windows: 驗證 Win32 API 正確調用
- Linux: wl-copy/xclip 優先順序
- macOS: pbcopy/pbpaste 可用性
- 錯誤處理: 工具不可用時降級到內部剪貼板

### 6.3 CLI Parser 輕量化

**🎯 STEP 2 — 改用 pico-args**

**遷移步驟**:
1. 移除 `clap` 依賴
2. 添加 `pico-args = "0.5"`
3. 重構 `Args` 結構和解析邏輯

**實現示例**:
```rust
// main.rs 重構
use pico_args::Arguments;

#[derive(Debug)]
struct Args {
    file: PathBuf,
    debug: bool,
    dec: Option<String>,
    en: Option<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut pargs = Arguments::from_env();
        
        let file = pargs.free_from_str()
            .unwrap_or_else(|_| PathBuf::from("Untitled"));
            
        let debug = pargs.contains("--debug");
        let dec = pargs.opt_value_from_str("--dec")?;
        let en = pargs.opt_value_from_str("--en")?;
        
        // 檢查未處理的參數
        let remaining = pargs.finish();
        if !remaining.is_empty() {
            eprintln!("Warning: unused arguments {:?}", remaining);
        }
        
        Ok(Self { file, debug, dec, en })
    }
}
```

**預期壓縮**: 150-250 KB

### 6.4 日誌系統簡化

**🖨 STEP 3 — 移除 env_logger，全部改用 eprintln!**

**實現策略**:
- 移除 `log` 和 `env_logger` crate
- 移除 `utils/logger.rs`
- 直接使用條件 `eprintln!`

**重構示例**:
```rust
// main.rs
fn main() -> Result<()> {
    let args = Args::parse();
    
    // 移除: utils::init_logger(args.debug);
    
    // 替換為直接條件輸出，使用 cfg!(debug_assertions) 自動禁用
    macro_rules! debug_log {
        ($($arg:tt)*) => {{
            if cfg!(debug_assertions) {
                eprintln!("[DEBUG] {}", format_args!($($arg)*));
            }
        }};
    }
    
    macro_rules! error_log {
        ($($arg:tt)*) => {
            eprintln!("[ERROR] {}", format_args!($($arg)*));
        };
    }
    
    // 在需要的地方使用
    debug_log!("Starting wedi with file: {:?}", args.file);
    
    Ok(())
}
```

**優點**: 在 release 模式下，debug_log 宏會被編譯器完全移除，進一步減小二進制大小。

**預期壓縮**: 100-150 KB

### 6.5 Crossterm Features 優化

**⌨ STEP 4 — 只開啟需要的 features**

**分析當前使用**:
- `cursor`: MoveTo, Hide, Show
- `event`: read, KeyEvent, KeyCode, KeyModifiers, Event::Resize (同步事件處理)
- `execute`: 執行命令到 stdout
- `terminal`: size, enable/disable_raw_mode, Clear, Enter/LeaveAlternateScreen
- `style`: 對話框使用 (Color, SetBackgroundColor, SetForegroundColor, ResetColor)

**優化配置**:
```toml
# Cargo.toml
[dependencies]
crossterm = { version = "0.27", default-features = false, features = [
    "cursor",    # 光標操作
    "event",     # 事件處理 (同步，替換 event-stream)
    "terminal",  # 終端控制
    "style",     # 樣式 (對話框)
] }
```

**說明**: 由於編輯器使用同步的 `crossterm::event::read()`，不需要 async 的 "event-stream" feature，使用 "event" 即可滿足需求並減少依賴大小。

**預期壓縮**: 50-100 KB

### 6.6 Win32 API 最小化

**🪟 STEP 5 — 使用 windows crate 的 build macro 生成最小綁定**

**當前狀態**: 使用 `windows = "0.58"` with specific features

**分析使用場景**:
- **Clipboard**: OpenClipboard, GlobalAlloc, SetClipboardData, GetClipboardData, CloseClipboard
- **編碼偵測**: GetACP, MultiByteToWideChar, WideCharToMultiByte (系統編碼)

**windows build macro 方案**:
1. 保留 `windows` crate 依賴
2. 使用 `windows::build!` 宏生成最小綁定
3. 只綁定需要的函數和結構體
4. 產物小於 20 KB，完全可控

**實現示例**:
```rust
// build.rs
fn main() {
    windows::build!(
        Windows::Win32::System::DataExchange::{
            OpenClipboard, CloseClipboard, SetClipboardData, GetClipboardData, EmptyClipboard
        },
        Windows::Win32::System::Memory::{
            GlobalAlloc, GlobalFree, GlobalLock, GlobalUnlock, GMEM_MOVEABLE
        },
        Windows::Win32::Foundation::{HWND, HANDLE},
        Windows::Win32::Globalization::GetACP
    );
}
```

**優點**: 
- 生成的綁定只有用到的函數 (<50 KB)
- 比 bindgen 小很多，避免龐大 unsafe 代碼
- 完全可控，不會引入不需要的依賴

**預期壓縮**: 200-400 KB

### 6.7 最大化 Release 壓縮

**🧨 STEP 6 — 最大化 release 壓縮**

**優化配置**:
```toml
[profile.release]
strip = true              # 移除符號表
lto = true                # 鏈接時優化
opt-level = "z"           # 最大化壓縮
codegen-units = 1         # 單編譯單元
panic = "abort"           # 減少 panic 處理代碼
incremental = false       # 避免未使用的編譯 cache，額外減少 20-50 KB
```

**額外優化**:
- 使用 `rustflags = ["-C", "target-feature=+crt-static"]` (Windows 靜態鏈接)
- 條件編譯移除調試代碼
- 手動內聯關鍵函數

**預期壓縮**: 額外 10-20%

### 6.8 實現時程與風險

**時程安排**:
- **Week 11**: STEP 1-3 (Clipboard, CLI, 日誌) - 4 天
- **Week 12**: STEP 4-6 (Crossterm, Win32, 壓縮) - 3 天

**風險評估**:
| 風險 | 影響 | 緩解措施 |
|------|------|----------|
| 跨平台 Clipboard 實現複雜 | 高 | 提供內部剪貼板 fallback |
| Win32 bindgen 綁定問題 | 中 | 保留 windows crate 作為備用 |
| 壓縮後功能異常 | 中 | 每個步驟後完整測試 |
| 編碼偵測功能喪失 | 低 | 實現簡單的編碼偵測邏輯 |

**測試策略**:
- 每個平台單獨測試 Clipboard 功能
- 壓縮前後功能對比測試
- 性能基準測試確保無回歸

### 6.9 交付物與驗收

**交付物**:
- 壓縮後的二進制文件 (300-600 KB)
- 更新的依賴清單
- 跨平台測試報告
- 性能對比數據

**驗收標準**:
- ✅ 二進制大小 ≤ 600 KB
- ✅ 全平台功能正常
- ✅ 啟動時間無顯著變化
- ✅ 編譯時間合理 (< 30s)
- ✅ 無新增崩潰或錯誤

**成功指標**:
- 壓縮率: > 65% (從 1.7MB 到 <600KB)
- 功能完整性: 100% 保留
- 可維護性: 代碼更清晰，依賴更少

---

## 更新依賴清單

**壓縮後依賴**:
```toml
[dependencies]
crossterm = { version = "0.27", default-features = false, features = ["cursor", "event", "terminal", "style"] }
pico-args = "0.5"          # 替換 clap
ropey = "1.6"              # 文本緩衝區 (保持)
unicode-width = "0.1"      # Unicode 寬度 (保持)
anyhow = "1.0"             # 錯誤處理 (保持)
encoding_rs = "0.8"        # 編碼處理 (保持)
windows = "0.58"           # Windows API (使用 build macro 生成最小綁定)

[build-dependencies]
windows = "0.58"           # 用於 build.rs 中的 build macro

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
tempfile = "3.8"
```

**移除的依賴**:
- `arboard = "3.3"` (~400-600KB)
- `clap = "4.5"` (~150-250KB)
- `log = "0.4"` (~50KB)
- `env_logger = "0.11"` (~50KB)
- `bindgen = "0.69"` (原本計劃使用，但改用 windows build macro)

**總預估壓縮**: 950-1650 KB (55-65% 壓縮率)

---

**文檔版本**: v1.2
**最後更新**: 2025-11-14
**維護者**: [Your Name]
