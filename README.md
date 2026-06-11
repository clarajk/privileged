# privileged

A small Linux-only utility crate for constructing privileged commands using `run0`, `sudo`, or `doas`.

`privileged` automatically detects an available privilege elevation command and returns a standard `std::process::Command`, allowing applications to support multiple elevation mechanisms without duplicating detection logic.

## Features

* Automatic detection of `run0`, `sudo`, and `doas`
* Optional `pkexec` support behind a feature flag
* Skips elevation when already running as root
* Returns standard `std::process::Command` values
* Linux-only

## Usage

```rust
use privileged::{command, PrivilegeMethod};

command("xbps-install", PrivilegeMethod::Auto)?
    .arg("-Syu")
    .status()?;
```

Explicitly select a privilege elevation command:

```rust
use privileged::{command, PrivilegeMethod};

command("sv", PrivilegeMethod::Run0)?
    .arg("up")
    .arg("sshd")
    .status()?;
```

## Environment Variable

The privilege method can be selected using the `PRIVILEGED_COMMAND` environment variable:

```text
auto
run0
sudo
doas
```

When the `pkexec` feature is enabled:

```text
pkexec
```

Example:

```bash
export PRIVILEGED_COMMAND=doas
```

```rust
let method = PrivilegeMethod::from_env()?;
```

## Supported Methods

Automatic detection checks the following commands in order:

1. `run0`
2. `sudo`
3. `doas`

`pkexec` is never selected automatically and must be requested explicitly.
