# Release Process Guide

This document describes how to create a new release for **wedi**.

## Prerequisites

- Push access to the repository
- All changes committed and pushed to `main` branch
- Tests passing
- Version number decided (following [Semantic Versioning](https://semver.org/))

## Release Steps

### 1. Update Version

Update the version in `Cargo.toml`:

```toml
[package]
name = "wedi"
version = "0.1.17"  # <- Update this
```

### 2. Update Changelog (Optional but Recommended)

Create or update `CHANGELOG.md` with the new version changes:

```markdown
# Changelog

## [0.1.17] - 2025-11-17

### Added
- New feature X
- Enhancement to Y

### Fixed
- Bug fix Z

### Changed
- Updated dependency A
```

### 3. Commit Changes

```bash
# Commit version bump
git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 0.1.17"
git push origin main
```

### 4. Create and Push Tag

Create an annotated tag with release notes:

```bash
# Create annotated tag with message
git tag -a v0.1.17 -m "Release v0.1.17

## New Features
- Feature X: description
- Enhancement Y: description

## Bug Fixes
- Fixed Z: description

## Breaking Changes
None
"

# Push the tag to trigger release workflow
git push origin v0.1.17
```

**Important:** The tag must follow the format `v*.*.*` (e.g., `v0.1.17`, `v1.0.0`) to trigger the release workflow.

### 5. Monitor Release Build

1. Go to the **Actions** tab in GitHub
2. Watch the "Release Build" workflow
3. The workflow will:
   - Build binaries for all supported platforms
   - Run tests
   - Create a GitHub Release
   - Upload all platform binaries
   - Generate checksums (SHA256SUMS)
   - Trigger WinGet update (if configured)

Build typically takes 10-15 minutes to complete.

### 6. Verify Release

Once the workflow completes:

1. Check the [Releases page](https://github.com/superyngo/wedi/releases)
2. Verify the new release appears with all assets:
   - Windows binaries (x86_64, i686)
   - Linux binaries (x86_64, i686, aarch64, armv7, musl variants)
   - macOS binaries (x86_64, aarch64)
   - SHA256SUMS file

### 7. Test Installation Scripts

Test the installation scripts to ensure they work with the new release:

**Windows:**
```powershell
# In a fresh PowerShell session
irm https://raw.githubusercontent.com/superyngo/wedi/main/install.ps1 | iex
wedi --version  # Should show the new version
```

**Linux/macOS:**
```bash
# In a fresh terminal
curl -fsSL https://raw.githubusercontent.com/superyngo/wedi/main/install.sh | bash
wedi --version  # Should show the new version
```

### 8. Edit Release Notes (Optional)

You can edit the release notes on GitHub to add more details:

1. Go to the release page
2. Click "Edit release"
3. Add additional information, screenshots, or highlights
4. Save changes

## Supported Platforms

The release workflow automatically builds for:

### Windows
- `wedi-windows-x86_64.exe` - 64-bit Windows
- `wedi-windows-i686.exe` - 32-bit Windows

### Linux (GNU libc)
- `wedi-linux-x86_64.tar.gz` - 64-bit
- `wedi-linux-i686.tar.gz` - 32-bit
- `wedi-linux-aarch64.tar.gz` - ARM64
- `wedi-linux-armv7.tar.gz` - ARMv7

### Linux (musl libc)
- `wedi-linux-x86_64-musl.tar.gz` - 64-bit
- `wedi-linux-i686-musl.tar.gz` - 32-bit
- `wedi-linux-aarch64-musl.tar.gz` - ARM64
- `wedi-linux-armv7-musl.tar.gz` - ARMv7

### macOS
- `wedi-macos-x86_64.tar.gz` - Intel Mac
- `wedi-macos-aarch64.tar.gz` - Apple Silicon (M1/M2/M3)

## Troubleshooting

### Build Fails

1. Check the Actions log for errors
2. Common issues:
   - Compilation errors (fix and push)
   - Test failures (fix and push)
   - Cross-compilation issues (check toolchain configuration)

### Tag Already Exists

If you need to redo a release:

```bash
# Delete local tag
git tag -d v0.1.17

# Delete remote tag
git push origin :refs/tags/v0.1.17

# Delete the release on GitHub (via web interface)

# Then recreate the tag
git tag -a v0.1.17 -m "Release notes..."
git push origin v0.1.17
```

### Wrong Version in Binary

Make sure you updated `Cargo.toml` and committed before creating the tag.

### Missing Platforms

Check the workflow file (`.github/workflows/release.yml`) to ensure all platforms are configured.

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** version (X.0.0): Incompatible API changes
- **MINOR** version (0.X.0): New functionality in a backwards compatible manner
- **PATCH** version (0.0.X): Backwards compatible bug fixes

Examples:
- `v0.1.16` → `v0.1.17`: Bug fixes or minor improvements
- `v0.1.17` → `v0.2.0`: New features added
- `v0.2.0` → `v1.0.0`: First stable release or breaking changes

## Pre-releases

For beta or release candidate versions:

```bash
# Beta release
git tag -a v0.2.0-beta.1 -m "Beta release v0.2.0-beta.1"

# Release candidate
git tag -a v1.0.0-rc.1 -m "Release candidate v1.0.0-rc.1"
```

Mark as pre-release on GitHub after workflow completes.

## Automation

The release process is automated via GitHub Actions:

1. **Trigger**: Pushing a tag matching `v*.*.*`
2. **Build Job**: Compiles for all platforms in parallel
3. **Release Job**: Creates GitHub release with all binaries
4. **Post-release**: Triggers WinGet package update

## Manual Release (Emergency)

If GitHub Actions is unavailable:

1. Build locally for your platform:
   ```bash
   cargo build --release
   ```

2. Create release manually on GitHub
3. Upload the binary
4. Update release notes

## Notes

- Always test on multiple platforms if possible
- Keep release notes clear and user-focused
- Include migration guides for breaking changes
- Consider announcing releases on relevant platforms (if applicable)
