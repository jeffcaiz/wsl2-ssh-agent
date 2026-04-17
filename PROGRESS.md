# Progress

## Scope Decisions

- Goal: forward a Windows SSH agent into WSL for use via `SSH_AUTH_SOCK`.
- Keep `Pageant` support through the legacy `WM_COPYDATA` transport.
- Keep Windows OpenSSH agent support through the fixed named pipe `\\.\pipe\openssh-ssh-agent`.
- Drop all `gpg` / `gpg-agent` / YubiKey-specific forwarding work for now.
- Do not support new Pageant named-pipe discovery for the first version.

## Current Project State

- Rust binary crate initialized as `wsl2-ssh-agent`.
- CLI behavior updated:
  - no arguments: print setup/help text and exit
  - `--auto`
  - `--openssh`
  - `--pipe <name>`
  - `--pageant`
  - forwarding mode requires an explicit backend selection
  - `--auto` means OpenSSH pipe first, then Pageant fallback
- Common SSH agent framing is implemented in `src/agent.rs`.
- Windows backend skeletons exist:
  - `src/backends/openssh_pipe.rs`
  - `src/backends/pageant.rs`
- Entry-point wiring is in `src/main.rs`.

## Validation Status

- Linux-host `cargo check` completed for the non-Windows build path.
- Native Windows `cargo check` completed successfully on April 17, 2026.
- `windows 0.61` binding mismatches in the Pageant backend were fixed.
- Live verification against a running Windows OpenSSH agent is still pending.
- Live verification against a real Pageant instance completed successfully on April 17, 2026.
- Verified `--pageant` by sending `SSH2_AGENTC_REQUEST_IDENTITIES` and receiving a valid `SSH2_AGENT_IDENTITIES_ANSWER`.
- Verified auto mode falls back from the default OpenSSH named pipe to Pageant when `\\.\pipe\openssh-ssh-agent` is unavailable.
- Verified WSL end-to-end use through `socat` and `SSH_AUTH_SOCK`.

## Likely Next Steps On Windows

1. Verify `openssh_pipe` behavior against the Windows OpenSSH agent.
2. Decide whether any further hardening is needed beyond the current forwarding flow.

## Files To Resume From

- `Cargo.toml`
- `src/main.rs`
- `src/agent.rs`
- `src/cli.rs`
- `src/backends/openssh_pipe.rs`
- `src/backends/pageant.rs`
- `README.md`
