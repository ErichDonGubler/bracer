use anyhow::{bail, Context};
use brace_expand::brace_expand;
use clap::Parser;
use std::{
    ffi::OsString,
    process::{exit, Command},
};

/// A utility to expand brace notation arguments to a binary (i.e., `{a,b,c}`).
#[derive(Debug, Parser)]
#[clap(about, author)]
struct Cli {
    /// The command to run and its arguments, if any.
    #[clap(raw(true))]
    command_and_args: Vec<OsString>,
}

fn main() -> anyhow::Result<()> {
    let Cli { command_and_args } = Cli::parse();

    let mut args = command_and_args.into_iter();

    let Some(cmd) = args.next() else {
        bail!("no command or args specified");
    };
    let mut cmd = Command::new(cmd);

    for arg in args {
        if let Some(s) = arg.to_str() {
            for expansion in brace_expand(s) {
                cmd.arg(expansion);
            }
        } else {
            cmd.arg(arg);
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
