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

## WSL Setup

Install `socat` in your WSL distro if needed:

```bash
sudo apt-get update
sudo apt-get install -y socat
```

Create a Unix socket that forwards to the Windows binary:

```bash
export SSH_AUTH_SOCK="$HOME/.ssh/agent.sock"
rm -f "$SSH_AUTH_SOCK"
socat UNIX-LISTEN:"$SSH_AUTH_SOCK",fork EXEC:'/path/to/wsl2-ssh-agent.exe -f'
```

Then point SSH clients at that socket:

```bash
export SSH_AUTH_SOCK="$HOME/.ssh/agent.sock"
ssh-add -l
```

Replace `/path/to/wsl2-ssh-agent.exe` with the WSL-visible path to your Windows build output.

## Backend Selection

```text
wsl2-ssh-agent.exe
wsl2-ssh-agent.exe -f
wsl2-ssh-agent.exe -f --auto
wsl2-ssh-agent.exe -f --openssh
wsl2-ssh-agent.exe -f --pipe \\.\pipe\openssh-ssh-agent
wsl2-ssh-agent.exe -f --pageant
```

Run without arguments to print setup help. Use `-f` to enter forwarding mode. `-f --auto` explicitly selects the default backend order. Add `--verbose` to print backend selection diagnostics to stderr.

## Notes

- The OpenSSH backend assumes the Windows agent is listening on the standard named pipe.
- The Pageant backend uses the older shared-memory plus `WM_COPYDATA` protocol.
- This project has passed `cargo check` on Windows, but live validation against a running OpenSSH agent and Pageant is still pending.
