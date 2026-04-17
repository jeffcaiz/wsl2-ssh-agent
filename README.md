# wsl2-ssh-agent

Bridge WSL SSH agent requests to a Windows SSH agent.

## Supported Backends

- Windows OpenSSH agent over `\\.\pipe\openssh-ssh-agent`
- Pageant over legacy `WM_COPYDATA`

Auto mode tries the OpenSSH named pipe first and falls back to Pageant.

## Build

Build this binary on Windows:

```powershell
cargo build --release
```

The resulting executable will be at:

```text
target\release\wsl2-ssh-agent.exe
```

## Release

GitHub Actions builds release artifacts for version tags that match `v*`.

Example:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This creates a GitHub Release and uploads `wsl2-ssh-agent.exe` directly as the release asset.

## Recommended: systemd User Socket

For regular use, prefer the bundled `systemd --user` socket setup. Modern WSL distributions generally support `systemd`, and this gives you a persistent `SSH_AUTH_SOCK` without keeping a manual `socat` process around.

First, place `wsl2-ssh-agent.exe` at `$HOME/.local/bin/wsl2-ssh-agent.exe`.

Then install the bundled user units from [`systemd/`](systemd):

```bash
mkdir -p "$HOME/.config/systemd/user"
cp systemd/wsl2-ssh-agent.socket "$HOME/.config/systemd/user/"
cp systemd/wsl2-ssh-agent.service "$HOME/.config/systemd/user/"
systemctl --user daemon-reload
systemctl --user enable --now wsl2-ssh-agent.socket
```

Expose the socket to SSH clients:

```bash
export SSH_AUTH_SOCK="${XDG_RUNTIME_DIR:-/run/user/$UID}/ssh-agent.sock"
ssh-add -l
```

The sample service uses the default auto mode. If you want to force Pageant, edit `wsl2-ssh-agent.service` and add `--pageant`.

## Manual Test Setup

If you only want to test the bridge manually, run `socat` yourself instead of installing the user units.

Install `socat` in your WSL distro if needed:

```bash
sudo apt-get update
sudo apt-get install -y socat
```

Create a Unix socket that forwards to the Windows binary:

```bash
export SSH_AUTH_SOCK="$HOME/.ssh/agent.sock"
rm -f "$SSH_AUTH_SOCK"
socat UNIX-LISTEN:"$SSH_AUTH_SOCK",fork EXEC:'/path/to/wsl2-ssh-agent.exe'
```

Then point SSH clients at that socket:

```bash
export SSH_AUTH_SOCK="$HOME/.ssh/agent.sock"
ssh-add -l
```

Replace `/path/to/wsl2-ssh-agent.exe` with the path you actually want to execute from WSL.

## Backend Selection

```text
wsl2-ssh-agent.exe
wsl2-ssh-agent.exe --pageant
wsl2-ssh-agent.exe --openssh
wsl2-ssh-agent.exe --openssh --pipe \\.\pipe\openssh-ssh-agent
```

Run without arguments to use the default auto mode. Use `--pageant` to force Pageant, or `--openssh [--pipe <name>]` to force OpenSSH. Add `--verbose` to print backend selection diagnostics to stderr.

## Notes

- The OpenSSH backend assumes the Windows agent is listening on the standard named pipe.
- The Pageant backend uses the older shared-memory plus `WM_COPYDATA` protocol.
- This project has passed `cargo check` on Windows, but live validation against a running OpenSSH agent and Pageant is still pending.
