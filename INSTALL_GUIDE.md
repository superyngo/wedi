# Installation Guide

This guide provides detailed instructions for installing **wedi** on different platforms.

## Table of Contents

- [Quick Install](#quick-install)
- [Manual Installation](#manual-installation)
- [From Source](#from-source)
- [Uninstallation](#uninstallation)
- [Troubleshooting](#troubleshooting)

## Quick Install

### Windows (PowerShell)

Run this command in PowerShell:

```powershell
irm https://raw.githubusercontent.com/superyngo/wedi/main/install.ps1 | iex
```

> **Note:** Replace `superyngo` with the actual GitHub superyngo/organization name.

The installer will:
1. Detect your system architecture (x64 or ARM64)
2. Download the latest precompiled binary from GitHub Releases
3. Install it to `%LOCALAPPDATA%\Programs\wedi`
4. Automatically add the installation directory to your PATH

### Linux / macOS (Bash)

Run this command in your terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/wedi/main/install.sh | bash
```

> **Note:** Replace `superyngo` with the actual GitHub superyngo/organization name.

The installer will:
1. Detect your OS (Linux/macOS) and architecture (x86_64/aarch64)
2. Download the latest precompiled binary from GitHub Releases
3. Install it to `~/.local/bin`
4. Optionally add the installation directory to your PATH

**Supported Platforms:**
- Linux: x86_64, i686, aarch64, armv7 (with various libc variants)
- macOS: x86_64 (Intel), aarch64 (Apple Silicon)
- Windows: x86_64, i686

## Manual Installation

### From Precompiled Binaries

1. Visit the [Releases page](https://github.com/superyngo/wedi/releases)
2. Download the appropriate binary for your platform:

   **Windows:**
   - `wedi-windows-x86_64.exe` - Windows 64-bit
   - `wedi-windows-i686.exe` - Windows 32-bit

   **Linux:**
   - `wedi-linux-x86_64.tar.gz` - Linux 64-bit (GNU libc)
   - `wedi-linux-x86_64-musl.tar.gz` - Linux 64-bit (musl libc)
   - `wedi-linux-aarch64.tar.gz` - Linux ARM64
   - `wedi-linux-armv7.tar.gz` - Linux ARMv7
   - `wedi-linux-i686.tar.gz` - Linux 32-bit

   **macOS:**
   - `wedi-macos-x86_64.tar.gz` - macOS Intel
   - `wedi-macos-aarch64.tar.gz` - macOS Apple Silicon

3. Extract and install:

   **Windows:**
   ```powershell
   # Create installation directory
   New-Item -ItemType Directory -Force -Path "$env:LOCALAPPDATA\Programs\wedi"

   # Move the downloaded .exe file
   Move-Item wedi-windows-x86_64.exe "$env:LOCALAPPDATA\Programs\wedi\wedi.exe"

   # Add to PATH (if not already)
   $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
   if ($userPath -notlike "*$env:LOCALAPPDATA\Programs\wedi*") {
       [Environment]::SetEnvironmentVariable("PATH", "$userPath;$env:LOCALAPPDATA\Programs\wedi", "User")
   }
   ```

   **Linux/macOS:**
   ```bash
   # Extract the archive
   tar -xzf wedi-linux-x86_64.tar.gz
   # or for macOS:
   # tar -xzf wedi-macos-x86_64.tar.gz

   # Make executable
   chmod +x wedi

   # Move to a directory in your PATH
   mkdir -p ~/.local/bin
   mv wedi ~/.local/bin/

   # Add to PATH (if not already)
   # For bash:
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc

   # For zsh:
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
   source ~/.zshrc
   ```

### Verify Installation

After installation, verify it works:

```bash
wedi --version
```

## From Source

If you prefer to build from source or need to customize the build:

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version recommended)
- Git

### Build Steps

```bash
# Clone the repository
git clone https://github.com/superyngo/wedi.git
cd wedi

# Build in release mode (optimized)
cargo build --release

# The binary will be at target/release/wedi (or wedi.exe on Windows)

# Optionally, install with cargo
cargo install --path .

# Or manually copy the binary
# Windows:
copy target\release\wedi.exe %LOCALAPPDATA%\Programs\wedi\

# Linux/macOS:
cp target/release/wedi ~/.local/bin/
chmod +x ~/.local/bin/wedi
```

## Uninstallation

### Windows

Using the installation script:
```powershell
irm https://raw.githubusercontent.com/superyngo/wedi/main/install.ps1 | iex -Uninstall
```

Or manually:
```powershell
# Remove the binary
Remove-Item "$env:LOCALAPPDATA\Programs\wedi\wedi.exe"

# Remove from PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$newPath = ($userPath -split ';' | Where-Object { $_ -ne "$env:LOCALAPPDATA\Programs\wedi" }) -join ';'
[Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
```

### Linux/macOS

Using the installation script:
```bash
curl -fsSL https://raw.githubusercontent.com/superyngo/wedi/main/install.sh | bash -s uninstall
```

Or manually:
```bash
# Remove the binary
rm ~/.local/bin/wedi

# Remove PATH entry from your shell config (if you added it)
# Edit ~/.bashrc or ~/.zshrc and remove the line:
# export PATH="$HOME/.local/bin:$PATH"
```

## Troubleshooting

### "Command not found" after installation

**Windows:**
1. Restart your PowerShell/terminal
2. Check if the directory is in PATH:
   ```powershell
   $env:PATH -split ';' | Select-String wedi
   ```
3. If not, manually add it as shown in the installation steps

**Linux/macOS:**
1. Restart your terminal or run `source ~/.bashrc` (or `~/.zshrc`)
2. Check if the directory is in PATH:
   ```bash
   echo $PATH | grep -o "[^:]*\.local/bin[^:]*"
   ```
3. If not, manually add it as shown in the installation steps

### Download fails

- Check your internet connection
- Try using a VPN if GitHub is blocked in your region
- Manually download from the Releases page and follow manual installation

### Permission denied (Linux/macOS)

Make sure the binary is executable:
```bash
chmod +x ~/.local/bin/wedi
```

### Wrong architecture downloaded

The installation scripts auto-detect your system. If it fails:
1. Check your architecture:
   - Windows: `echo $env:PROCESSOR_ARCHITECTURE`
   - Linux/macOS: `uname -m`
2. Manually download the correct binary from the Releases page

### Anti-virus/SmartScreen warnings (Windows)

New unsigned binaries may trigger warnings. You can:
1. Click "More info" → "Run anyway"
2. Add an exception in your anti-virus software
3. Build from source instead

## Getting Help

If you encounter issues not covered here:
1. Check existing [GitHub Issues](https://github.com/superyngo/wedi/issues)
2. Create a new issue with details about your system and the error
3. Include output of:
   - `wedi --version` (if it runs)
   - Your OS and architecture
   - Error messages
