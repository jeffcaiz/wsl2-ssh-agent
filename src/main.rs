mod agent;
mod cli;
mod backends;

use std::io;

use agent::serve_stdio;
use cli::{BackendSelection, Command, Config};

pub const DEFAULT_OPENSSH_PIPE: &str = r"\\.\pipe\openssh-ssh-agent";

#[cfg(not(windows))]
fn main() {
    eprintln!("wsl2-ssh-agent only runs on Windows.");
    std::process::exit(1);
}

#[cfg(windows)]
fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn run() -> io::Result<()> {
    let config = match Config::parse()? {
        Command::ShowHelp => {
            cli::print_usage();
            return Ok(());
        }
        Command::Run(config) => config,
    };

    match config.selection {
        BackendSelection::Auto => {
            if config.verbose {
                eprintln!("trying OpenSSH pipe backend at {}", DEFAULT_OPENSSH_PIPE);
            }

            match backends::openssh_pipe::NamedPipeBackend::connect(DEFAULT_OPENSSH_PIPE) {
                Ok(backend) => serve_stdio(backend),
                Err(pipe_err) => {
                    if config.verbose {
                        eprintln!("OpenSSH pipe unavailable: {pipe_err}");
                        eprintln!("falling back to Pageant WM_COPYDATA");
                    }
                    let backend = backends::pageant::PageantBackend::connect()?;
                    serve_stdio(backend)
                }
            }
        }
        BackendSelection::OpenSsh { pipe } => {
            if config.verbose {
                eprintln!("using named pipe backend at {pipe}");
            }
            let backend = backends::openssh_pipe::NamedPipeBackend::connect(&pipe)?;
            serve_stdio(backend)
        }
        BackendSelection::Pageant => {
            if config.verbose {
                eprintln!("using Pageant WM_COPYDATA backend");
            }
            let backend = backends::pageant::PageantBackend::connect()?;
            serve_stdio(backend)
        }
    }
}
