use std::env;
use std::io;

#[derive(Debug, Clone)]
pub struct Config {
    pub verbose: bool,
    pub selection: BackendSelection,
}

#[derive(Debug, Clone)]
pub enum BackendSelection {
    Auto,
    OpenSsh { pipe: String },
    Pageant,
}

#[derive(Debug, Clone)]
pub enum Command {
    ShowHelp,
    Run(Config),
}

impl Config {
    pub fn parse() -> io::Result<Command> {
        let mut verbose = false;
        let mut selection = BackendSelection::Auto;
        let mut forward = false;
        let mut saw_args = false;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            saw_args = true;
            match arg.as_str() {
                "--forward" | "-f" => forward = true,
                "--auto" => selection = BackendSelection::Auto,
                "--verbose" | "-v" => verbose = true,
                "--pageant" => selection = BackendSelection::Pageant,
                "--pipe" => {
                    let pipe = args.next().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "--pipe requires a value")
                    })?;
                    selection = BackendSelection::OpenSsh { pipe };
                }
                "--openssh" => {
                    selection = BackendSelection::OpenSsh {
                        pipe: crate::DEFAULT_OPENSSH_PIPE.to_string(),
                    };
                }
                "--help" | "-h" => {
                    return Ok(Command::ShowHelp);
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("unknown argument: {arg}"),
                    ));
                }
            }
        }

        if !saw_args || !forward {
            return Ok(Command::ShowHelp);
        }

        Ok(Command::Run(Self { verbose, selection }))
    }
}

pub fn print_usage() {
    eprintln!(
        "\
Usage:
  wsl2-ssh-agent
  wsl2-ssh-agent -f [--verbose] [--auto | --openssh | --pipe <name> | --pageant]

Run without arguments to show this setup help.
Use -f / --forward to enter agent forwarding mode.

Forwarding mode defaults to:
  1. Try Windows OpenSSH named pipe.
  2. Fall back to Pageant WM_COPYDATA.

WSL example:
  export SSH_AUTH_SOCK=\"$HOME/.ssh/agent/s.wsl2-ssh-agent\"
  mkdir -p \"$(dirname \"$SSH_AUTH_SOCK\")\"
  rm -f \"$SSH_AUTH_SOCK\"
  socat UNIX-LISTEN:\"$SSH_AUTH_SOCK\",fork EXEC:'/path/to/wsl2-ssh-agent.exe -f --pageant'
  ssh-add -l

Options:
  -f, --forward   Enter forwarding mode.
  --auto          Try Windows OpenSSH first, then fall back to Pageant.
  --openssh       Force the default Windows OpenSSH pipe.
  --pipe <name>   Force a specific Windows named pipe.
  --pageant       Force Pageant WM_COPYDATA transport.
  --verbose, -v   Print diagnostics to stderr.
  --help, -h      Show this help text."
    );
}
