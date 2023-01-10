use anyhow::{bail, Context};
use brace_expand::brace_expand;
use clap::Parser;
use std::{
    io::{stdout, Write},
    process::{exit, Command},
};

/// A utility to expand brace notation arguments to a binary (i.e., `{a,b,c}`).
///
/// Note that this tool expects UTF-8 input.
#[derive(Debug, Parser)]
#[clap(about, author)]
enum Cli {
    Expand(ExpandSubcommand),
    Exec(ExecSubcommand),
}

/// Performs brace expansion on arguments provided after `--`.
#[derive(Debug, Parser)]
struct ExpandSubcommand {
    /// Separate output components with a null (`\0`) byte.
    #[clap(short = 'z', required(true))]
    null_delimited: bool,
    /// The arguments to expand into separated strings, if any.
    #[clap(raw(true))]
    args: Vec<String>,
}

#[derive(Debug, Parser)]
struct ExecSubcommand {
    /// The command to run and its arguments, if any.
    #[clap(raw(true))]
    command_and_args: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse() {
        Cli::Expand(ExpandSubcommand {
            null_delimited: _, // should unconditionally be `true` right now
            args,
        }) => {
            let stdout = stdout();
            let mut stdout = stdout.lock();

            let mut output = args.iter().flat_map(|arg| brace_expand(&arg));
            if let Some(component) = output.next() {
                write!(stdout, "{component}").unwrap();
            }
            for component in output {
                write!(stdout, "\0{component}").unwrap();
            }
            Ok(())
        }
        Cli::Exec(ExecSubcommand { command_and_args }) => {
            let mut args = command_and_args.into_iter();

            let Some(cmd) = args.next() else {
                bail!("no command or args specified");
            };
            let mut cmd = Command::new(cmd);

            for arg in args {
                for expansion in brace_expand(&arg) {
                    cmd.arg(expansion);
                }
            }

            // TODO: log what we're about to run
            dbg!(&cmd);
            let mut child = cmd.spawn().context("failed to spawn child process")?;
            let status = child.wait().context("failed to wait for child process")?;
            if let Some(code) = status.code() {
                exit(code);
            } else {
                bail!("child process did not return an error code; did it get killed by a signal?");
            }
        }
    }
}
