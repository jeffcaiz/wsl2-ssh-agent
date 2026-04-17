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
        let mut custom_pipe = None;

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--auto" => selection = BackendSelection::Auto,
                "--verbose" | "-v" => verbose = true,
                "--pageant" => selection = BackendSelection::Pageant,
                "--pipe" => {
                    let pipe = args.next().ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "--pipe requires a value")
                    })?;
                    custom_pipe = Some(pipe);
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

        if custom_pipe.is_some() && !matches!(selection, BackendSelection::OpenSsh { .. }) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--pipe can only be used with --openssh",
            ));
        }

        if let (BackendSelection::OpenSsh { pipe }, Some(custom_pipe)) = (&mut selection, custom_pipe) {
            *pipe = custom_pipe;
        }

        Ok(Command::Run(Self { verbose, selection }))
    }
}

pub fn print_usage() {
    eprintln!(
        "\
Usage:
  wsl2-ssh-agent [--verbose]
  wsl2-ssh-agent [--verbose] --pageant
  wsl2-ssh-agent [--verbose] --openssh [--pipe <name>]

Runs in auto mode by default:
  1. Try Windows OpenSSH named pipe.
  2. Fall back to Pageant WM_COPYDATA.

Options:
  --auto          Try Windows OpenSSH first, then fall back to Pageant.
  --openssh       Force the OpenSSH backend.
  --pipe <name>   Override the OpenSSH named pipe. Only valid with --openssh.
  --pageant       Force Pageant WM_COPYDATA transport.
  --verbose, -v   Print diagnostics to stderr.
  --help, -h      Show this help text."
    );
}
