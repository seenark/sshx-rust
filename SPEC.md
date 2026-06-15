# SSHX — Complete Architecture Specification

---

## 1. Full CLI Surface

### Top-Level

```
sshx [HOST_ALIAS]          # main entry — selector or direct connect
sshx connect [HOST_ALIAS]  # explicit connect subcommand
sshx config                # config management subcommands
sshx version               # print version
sshx help                  # print help
```

---

### `sshx` / `sshx connect`

```
sshx
sshx [HOST_ALIAS]
sshx connect
sshx connect [HOST_ALIAS]

Behavior:
  No arg    → open fuzzy selector with all hosts grouped
  With arg  → match against alias or host name, skip selector
              if no match found → show selector pre-filtered with arg as query
```

---

### `sshx config`

```
sshx config add                    # wizard to add new host
sshx config edit [HOST_ALIAS]      # open host in $EDITOR
sshx config remove [HOST_ALIAS]    # remove host with confirmation
sshx config list                   # list all hosts with annotations
sshx config list --group GROUP     # filter by group
sshx config validate               # parse all configs, report errors
sshx config show [HOST_ALIAS]      # print raw SSH config block + annotations
```

---

### Global Flags

```
--config PATH     override SSH config file (default: ~/.ssh/config)
--dry-run         show what would happen, execute nothing
--verbose         debug output
--version         same as sshx version
--help            same as sshx help
```

---

## 2. Data Model

### Core Types

```rust
/// A fully parsed SSH host with all SSHX annotations resolved
pub struct SSHHost {
    // SSH config native fields
    pub name: String,                          // Host directive value
    pub hostname: String,                      // HostName directive
    pub port: Option<u16>,                     // Port directive (default 22)
    pub user: Option<String>,                  // User directive
    pub identity_file: Option<PathBuf>,        // IdentityFile directive
    pub local_forwards: Vec<LocalForward>,     // LocalForward directives (can be multiple)
    pub strict_host_checking: Option<StrictHostChecking>,
    pub user_known_hosts_file: Option<String>,
    pub extra_options: Vec<(String, String)>,  // any other SSH options we don't parse specially

    // SSHX annotations
    pub sshx: SSHXAnnotations,

    // source location (for editing)
    pub source: SourceLocation,
}

pub struct LocalForward {
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
}

pub enum StrictHostChecking {
    Yes,
    No,
    Ask,
    AcceptNew,
}

pub struct SSHXAnnotations {
    pub group: Option<String>,
    pub password: Option<String>,
    pub requires: Option<String>,       // host name of jump host
    pub alias: Option<String>,
    pub background: bool,               // -f -N mode
    pub description: Option<String>,
    pub after_connect: Option<String>,  // local shell command
}

pub struct SourceLocation {
    pub file: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
}
```

---

### Config Index

```rust
/// Full parsed state — everything SSHX knows
pub struct ConfigIndex {
    pub hosts: Vec<SSHHost>,
    pub groups: IndexMap<String, Vec<usize>>,   // group name → host indices
    pub aliases: HashMap<String, usize>,         // alias → host index
    pub source_files: Vec<PathBuf>,              // all files parsed (includes Includes)
}

impl ConfigIndex {
    pub fn resolve_alias(&self, input: &str) -> Option<&SSHHost>
    pub fn find_host(&self, name: &str) -> Option<&SSHHost>
    pub fn hosts_in_group(&self, group: &str) -> Vec<&SSHHost>
    pub fn jump_host_for(&self, host: &SSHHost) -> Option<&SSHHost>
}
```

---

### Chain State

```rust
/// Runtime state of a background tunnel
pub struct TunnelProcess {
    pub jump_host: String,
    pub pid: u32,
    pub local_ports: Vec<u16>,      // ports we bound locally
    pub started_at: SystemTime,
}

pub enum TunnelStatus {
    NotRunning,
    Running(TunnelProcess),
    PortConflict(u16),              // port already in use by something else
    Failed(String),                 // process exited with error
}
```

---

### SSH Command Builder

```rust
pub struct SSHCommand {
    pub host: String,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub identity_file: Option<PathBuf>,
    pub local_forwards: Vec<LocalForward>,
    pub background: bool,            // -f -N
    pub extra_args: Vec<String>,
    pub password: Option<String>,    // if Some → wrap with sshpass
}

impl SSHCommand {
    pub fn build(&self) -> String    // returns final shell command string
    pub fn build_parts(&self) -> Vec<String>  // returns argv vec
}
```

---

### SSHX Own Config (toml)

```rust
/// ~/.config/sshx/config.toml
pub struct SshxConfig {
    pub general: GeneralConfig,
    pub ui: UIConfig,
    pub tunnel: TunnelConfig,
}

pub struct GeneralConfig {
    pub ssh_config_path: Option<PathBuf>,   // override default ~/.ssh/config
    pub default_user: Option<String>,
}

pub struct UIConfig {
    pub fuzzy_threshold: f32,               // default 0.4
    pub host_weight: f32,                   // default 0.7
    pub hostname_weight: f32,               // default 0.3
    pub show_descriptions: bool,            // default true
    pub show_hostnames: bool,               // default true
    pub group_sort: GroupSort,
}

pub enum GroupSort {
    Alphabetical,
    ConfigOrder,                            // order they appear in SSH config
}

pub struct TunnelConfig {
    pub check_interval_ms: u64,             // how often to poll tunnel alive, default 500
    pub connect_timeout_s: u64,             // default 10
}
```

---

## 3. Error Taxonomy

### Design Principle

Every error has:
- A **code** — machine readable, stable across versions
- A **message** — human readable, shown in terminal  
- A **hint** — what to do about it
- A **severity** — fatal stops execution, warning continues

```rust
pub enum SshxError {
    // ── Config Parsing ──────────────────────────────────────────────
    ConfigFileNotFound { path: PathBuf },
    // E001: ~/.ssh/config not found — create it or use --config PATH

    ConfigFileUnreadable { path: PathBuf, reason: String },
    // E002: permission denied or IO error reading config

    AnnotationParseError { file: PathBuf, line: usize, raw: String },
    // E003: ## sshx: line found but key=value could not be parsed

    UnknownAnnotationKey { file: PathBuf, line: usize, key: String },
    // E004: ## sshx: unknownkey = value — typo guard

    DuplicateAlias { alias: String, host1: String, host2: String },
    // E005: two hosts claim the same alias

    // ── Host Resolution ─────────────────────────────────────────────
    HostNotFound { input: String },
    // E010: no host or alias matches input

    AliasAmbiguous { input: String, matches: Vec<String> },
    // E011: input partially matches multiple hosts (shouldn't happen with aliases, can with fuzzy)

    RequiresHostNotFound { host: String, requires: String },
    // E012: ## sshx: requires = X but X doesn't exist in config

    CircularRequires { host: String },
    // E013: A requires B requires A (safety check even though we only support one level)

    // ── Tunnel / Chain ───────────────────────────────────────────────
    TunnelSpawnFailed { jump_host: String, reason: String },
    // E020: failed to start background ssh -f -N process

    TunnelPortBusy { port: u16 },
    // E021: local port already in use before tunnel starts

    TunnelTimeout { jump_host: String, timeout_s: u64 },
    // E022: tunnel started but port never became reachable within timeout

    TunnelDiedEarly { jump_host: String, exit_code: Option<i32> },
    // E023: tunnel process exited before we connected

    // ── SSH Execution ────────────────────────────────────────────────
    SshpassNotFound,
    // E030: password annotation exists but sshpass binary not in PATH
    // hint: brew install hudochenkov/sshpass/sshpass  OR  apt install sshpass

    SshNotFound,
    // E031: ssh binary not in PATH (very unusual)

    SshCommandFailed { exit_code: Option<i32> },
    // E032: ssh exited non-zero (wrong password, host unreachable, etc.)


    // ── Config Wizard ────────────────────────────────────────────────
    ConfigWriteFailed { path: PathBuf, reason: String },
    // E050: cannot write to SSH config file

    HostAlreadyExists { name: String },
    // E051: host with this name already in config

    InvalidHostName { name: String },
    // E052: contains spaces or illegal chars

    InvalidPort { input: String },
    // E053: not a valid u16

    // ── SSHX Own Config ──────────────────────────────────────────────
    SshxConfigParseFailed { path: PathBuf, reason: String },
    // E060: ~/.config/sshx/config.toml is malformed TOML

    SshxConfigWriteFailed { path: PathBuf, reason: String },
    // E061: cannot write sshx config
}
```

---

### Error Display Contract

```
✗ [E022] Tunnel timeout — jump host did not become ready in 10s
  Host:    inspireivf-proxmox (61.47.34.14:10522)
  Hint:    Check that the host is reachable and LocalForward port is not blocked
           Try: ssh -v forward-port@61.47.34.14 -p 10522
```

Every error renders as:
```
✗ [EXXX] Short title
  Context: relevant values
  Hint:    what to do + example command if applicable
```

---

## 4. `sshx.toml` Schema

Location: `~/.config/sshx/config.toml`

```toml
[general]
ssh_config_path = "~/.ssh/config"   # optional override
default_user    = ""                # optional fallback user

[ui]
fuzzy_threshold  = 0.4
host_weight      = 0.7
hostname_weight  = 0.3
show_descriptions = true
show_hostnames    = true
group_sort        = "alphabetical"   # or "config_order"

[tunnel]
check_interval_ms = 500
connect_timeout_s = 10
```

### Defaults Behavior

```
If file does not exist   → all defaults apply, no error
If file exists but empty → all defaults apply, no error
If key missing           → default for that key applies
If file malformed TOML   → E060 fatal error
```

### Create Default Config

```
sshx config init    ← writes default toml with comments explaining each field
```

Full annotated default file SSHX writes:

```toml
# SSHX Configuration
# Location: ~/.config/sshx/config.toml

[general]
# Path to your SSH config file. Default: ~/.ssh/config
# ssh_config_path = "~/.ssh/config"

# Fallback username if not specified in SSH config
# default_user = ""

[ui]
# Fuzzy search sensitivity (0.0 = exact, 1.0 = anything matches)
fuzzy_threshold = 0.4

# How much weight to give host name vs hostname in fuzzy scoring
host_weight     = 0.7
hostname_weight = 0.3

# Show description annotation in selector list
show_descriptions = true

# Show raw HostName (IP/domain) next to host name in selector
show_hostnames = true

# How to sort groups: "alphabetical" or "config_order"
group_sort = "alphabetical"

[tunnel]
# How often (ms) to check if background tunnel port is ready
check_interval_ms = 500

# How long (seconds) to wait for tunnel to become ready before giving up
connect_timeout_s = 10
```

---

## 5. GitHub Actions — Build & Release

### Target Matrix

```
Platform          Target Triple                  Notes
─────────────────────────────────────────────────────────────
macOS arm64       aarch64-apple-darwin           M1/M2/M3
macOS x86_64      x86_64-apple-darwin            Intel Mac
Linux x86_64      x86_64-unknown-linux-musl      static binary
Linux arm64       aarch64-unknown-linux-musl      static binary (Raspberry Pi etc)
```

No Windows for now.

---

### Release Workflow

```yaml
# .github/workflows/release.yml
# Triggers on: push of tag v*.*.*

name: Release

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
          - target: aarch64-unknown-linux-musl
            os: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install musl tools (Linux only)
        if: contains(matrix.target, 'musl')
        run: |
          sudo apt-get update
          sudo apt-get install -y musl-tools cross

      - name: Build
        run: |
          cargo build --release --target ${{ matrix.target }}

      - name: Package binary
        run: |
          BINARY=target/${{ matrix.target }}/release/sshx
          ARCHIVE=sshx-${{ github.ref_name }}-${{ matrix.target }}.tar.gz
          tar czf $ARCHIVE -C target/${{ matrix.target }}/release sshx
          echo "ARCHIVE=$ARCHIVE" >> $GITHUB_ENV

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: ${{ env.ARCHIVE }}

  release:
    name: Create GitHub Release
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: artifacts/
          merge-multiple: true

      - name: Generate checksums
        run: |
          cd artifacts
          sha256sum *.tar.gz > checksums.txt

      - name: Create Release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            artifacts/*.tar.gz
            artifacts/checksums.txt
          generate_release_notes: true
```

---

### Install Script — `install.sh`

```bash
#!/usr/bin/env bash
set -euo pipefail

REPO="yourname/sshx"
BINARY="sshx"
INSTALL_DIR="${SSHX_INSTALL_DIR:-/usr/local/bin}"

# Detect platform
OS=$(uname -s)
ARCH=$(uname -m)

case "$OS-$ARCH" in
  Darwin-arm64)  TARGET="aarch64-apple-darwin" ;;
  Darwin-x86_64) TARGET="x86_64-apple-darwin" ;;
  Linux-x86_64)  TARGET="x86_64-unknown-linux-musl" ;;
  Linux-aarch64) TARGET="aarch64-unknown-linux-musl" ;;
  *)
    echo "✗ Unsupported platform: $OS-$ARCH"
    exit 1
    ;;
esac

# Get latest version tag
VERSION=$(curl -sf "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' \
  | sed -E 's/.*"v([^"]+)".*/\1/')

echo "→ Installing sshx v$VERSION for $TARGET"

URL="https://github.com/$REPO/releases/download/v$VERSION/sshx-v$VERSION-$TARGET.tar.gz"

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

curl -sfL "$URL" | tar xz -C "$TMP"

chmod +x "$TMP/$BINARY"

if [[ -w "$INSTALL_DIR" ]]; then
  mv "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"
else
  sudo mv "$TMP/$BINARY" "$INSTALL_DIR/$BINARY"
fi

echo "✓ sshx installed to $INSTALL_DIR/$BINARY"
echo "  Run: sshx --help"
```

---

### Homebrew Tap (later)

```
Repo:  github.com/yourname/homebrew-sshx

Formula file:  Formula/sshx.rb

brew tap yourname/sshx
brew install sshx
```

Formula auto-updates via a second GitHub Action that triggers after release and updates the `url` + `sha256` in the formula file.

---

## Complete Picture — One Page Summary

```
CLI Surface
  sshx [alias]              → connect
  sshx config add/edit/     → manage hosts
         remove/list/show/
         validate/init

Data Flow
  SSH config files
    + ## sshx: annotations
    → ConfigIndex (hosts, groups, aliases)
    → selector UI or alias resolve
    → if requires= → tunnel lifecycle
    → SSHCommand build
    → sshpass wrap if password
    → execute SSH command

Key Types
  SSHHost          full host + annotations
  ConfigIndex      queryable index of all hosts
  TunnelProcess    runtime tunnel state
  SSHCommand       command builder
  SshxConfig       ~/.config/sshx/config.toml
  SshxError        typed errors E001–E061

Annotation Format
  ## sshx: key = value
  Keys: group, password, requires, alias,
        background, description, after_connect

Release
  4 targets: macOS arm64/x86, Linux x86/arm64
  install.sh  curl | bash
  Homebrew tap (later)
```

---

This is the complete blueprint. Nothing left ambiguous. Ready to write Rust.

