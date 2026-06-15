# sshx

`sshx` is a Rust CLI for working with SSH configs that have outgrown plain `ssh host-name`.

It reads your OpenSSH config, understands a small set of `## sshx:` annotations, gives you an interactive selector, and builds the exact `ssh` command it will run. It also handles grouped hosts, aliases, background tunnel setup, and a few config-management commands for large personal or team SSH inventories.

## Why this project exists

Plain SSH config files scale poorly once you have dozens of hosts:

- host names stop being memorable
- environments need grouping
- common shortcuts become tribal knowledge
- tunnel/jump-host setup gets repetitive
- editing the right `Host` block becomes annoying when configs are split with `Include`

`sshx` exists to keep using standard OpenSSH config files, but add a lightweight layer for:

- discovery — interactive fuzzy host selection
- structure — groups, aliases, descriptions
- ergonomics — direct connect, dry-run, edit/show/list commands
- automation — background tunnel bootstrap and post-connect hooks

It is not a replacement for `ssh`. It is a smarter front-end for the SSH config you already have.

## What it does

Core behavior:

- `sshx` with no arguments opens an interactive selector
- `sshx <name>` resolves either an SSHX alias or a raw `Host` name
- if there is no exact match, the selector opens with your input prefilled
- `sshx --dry-run ...` prints the exact command without executing it
- `sshx` reads OpenSSH `Include` directives recursively
- `sshx` validates duplicate aliases, missing `requires` targets, and circular `requires` chains

SSH features preserved by command generation:

- `HostName`
- `Port`
- `User`
- `IdentityFile`
- `LocalForward`
- `StrictHostKeyChecking`
- `UserKnownHostsFile`
- extra SSH options passed through as `-o key=value`

SSHX annotations:

- `group`
- `alias`
- `description`
- `password`
- `requires`
- `background`
- `after_connect`

## How sshx works

`sshx` parses your SSH config into an index of hosts, groups, aliases, and source locations.

When you connect:

1. it resolves the host by alias or `Host` name
2. if the target has `## sshx: requires = <host>`, it ensures the required host's local forwards are up first
3. it builds the final `ssh` or `sshpass ssh` command
4. it runs the command
5. if `after_connect` is set, it runs that local shell command after SSH exits successfully

This keeps the source of truth in your SSH config, not in a second host database.

## Installation

### Install the latest release

```sh
curl -fsSL https://raw.githubusercontent.com/seenark/sshx-rust/main/install.sh | bash
```

By default the installer writes `sshx` to `/usr/local/bin`.

Install to a custom directory:

```sh
SSHX_INSTALL_DIR="$HOME/.local/bin" curl -fsSL https://raw.githubusercontent.com/seenark/sshx-rust/main/install.sh | bash
```

### Build from source

Requirements:

- Rust toolchain
- OpenSSH client (`ssh`)
- `sshpass` only if you use `## sshx: password = ...`

Install from a checked-out repository:

```sh
cargo install --path .
```

Build a release binary without installing:

```sh
cargo build --release
```

Binary location:

```text
target/release/sshx
```

## Quick start

Optional: create the SSHX config file:

```sh
sshx config init
```

This creates:

```text
~/.config/sshx/config.toml
```

Add a host to your SSH config:

```sshconfig
Host prod-app
    HostName 203.0.113.50
    Port 2222
    User deploy
    IdentityFile ~/.ssh/deploy_key
    ## sshx: group = production
    ## sshx: alias = prod
    ## sshx: description = "Main production app server"
```

Preview the exact SSH command:

```sh
sshx --dry-run prod
```

Connect:

```sh
sshx prod
```

Open the interactive selector:

```sh
sshx
```

## SSH config annotations

Annotations live inside a `Host` block and use the form:

```sshconfig
## sshx: key = value
```

Example:

```sshconfig
Host prod-app
    HostName 203.0.113.50
    User deploy
    LocalForward 8080 localhost:80
    LocalForward 3306 db.internal:3306
    StrictHostKeyChecking no
    UserKnownHostsFile /dev/null
    ## sshx: group = production
    ## sshx: alias = prod
    ## sshx: description = "Main production app server"
    ## sshx: password = s3cret
    ## sshx: after_connect = "curl -s http://localhost:8080/health"
```

### Annotation reference

#### `group`

Logical group name used by `sshx config list` and the selector display.

```sshconfig
## sshx: group = production
```

#### `alias`

Short name accepted by `sshx <alias>`.

```sshconfig
## sshx: alias = prod
```

#### `description`

Human-readable text shown in list and selector output.

```sshconfig
## sshx: description = "Main production app server"
```

#### `password`

Uses `sshpass -p <password> ssh ...` instead of plain `ssh`.

```sshconfig
## sshx: password = s3cret
```

Use only if you intentionally rely on password auth.

#### `requires`

Declares that another host must be available first. `sshx` uses this to bring up the required host's local forwards before connecting to the target host.

```sshconfig
## sshx: requires = bastion
```

This is useful when a host depends on a background tunnel or bastion-like forward chain defined elsewhere in your SSH config.

#### `background`

Marks the host as a background SSH process by adding `-f -N`.

```sshconfig
## sshx: background = true
```

#### `after_connect`

Runs a local shell command after the SSH session exits successfully.

```sshconfig
## sshx: after_connect = "curl -s http://localhost:8080/health"
```

## Usage

### Top-level commands

```text
sshx [HOST_ALIAS]
sshx connect [HOST_ALIAS]
sshx config <SUBCOMMAND>
sshx version
```

### Global options

```text
--config PATH   Override the SSH config file path
--dry-run       Print what would run without executing it
-v, --verbose   Print debug output
--help
--version
```

## Config commands

### Connect

```sh
sshx prod
sshx connect prod
```

- resolves by alias first, then by raw host name
- falls back to the selector if there is no exact match

### List hosts

```sh
sshx config list
sshx config list --group production
```

Shows hosts grouped by `## sshx: group = ...` when groups exist.

### Show a host

```sh
sshx config show prod
```

Prints parsed SSH fields, SSHX annotations, and the source file/line range for the host block.

### Edit a host

```sh
sshx config edit prod
```

Opens the source file in `$EDITOR` at the beginning of the host block.

### Add a host

```sh
sshx config add
```

Starts an interactive prompt that appends a new `Host` block to the active SSH config file.

### Remove a host

```sh
sshx config remove prod
```

Removes the matching host block from the source config file after confirmation.

### Validate configs

```sh
sshx config validate
```

Parses the SSH config and fails on structural issues such as:

- unreadable or missing config files
- malformed SSHX annotations
- unknown SSHX annotation keys
- duplicate aliases
- missing `requires` targets
- circular `requires` chains

### Initialize SSHX config

```sh
sshx config init
```

Creates the default SSHX TOML config at `~/.config/sshx/config.toml` if it does not already exist.

## SSHX config file

SSHX also has its own optional TOML config:

```text
~/.config/sshx/config.toml
```

Current sections:

- `[general]`
  - `ssh_config_path`
  - `default_user`
- `[ui]`
  - `fuzzy_threshold`
  - `host_weight`
  - `hostname_weight`
  - `show_descriptions`
  - `show_hostnames`
  - `group_sort`
- `[tunnel]`
  - `check_interval_ms`
  - `connect_timeout_s`

Behavior:

- if `--config PATH` is passed, it wins
- otherwise `sshx` reads `ssh_config_path` from the SSHX config
- otherwise it falls back to `~/.ssh/config`

## Examples

### Basic alias-driven workflow

```sshconfig
Host prod-app
    HostName 203.0.113.50
    User deploy
    ## sshx: alias = prod
    ## sshx: group = production
```

```sh
sshx prod
```

### Split configs with `Include`

```sshconfig
Include ~/.ssh/conf.d/*
```

`sshx` follows those includes and builds one combined host index.

### Tunnel bootstrap with `requires`

```sshconfig
Host bastion
    HostName 198.51.100.10
    User tunnel
    LocalForward 15432 db.internal:5432
    ## sshx: background = true

Host reporting-db
    HostName 127.0.0.1
    Port 15432
    ## sshx: requires = bastion
```

Running:

```sh
sshx reporting-db
```

`sshx` first makes sure the `bastion` tunnel is up, then connects to `reporting-db`.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```