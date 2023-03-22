mod my_lib;

use std::{
    os::windows::process::CommandExt,
    process::{Child, Command},
};

use anyhow::{anyhow, bail, Context, Ok};
use clap::Parser;
use my_lib::config::Config;

fn main() -> anyhow::Result<()> {
    let cli = crate::my_lib::cli::Cli::parse();
    let config = Config::from_cli(cli).with_context(|| anyhow!("Failed to verify the config."))?;

    if let Some(cwd) = &config.cwd {
        std::env::set_current_dir(cwd).with_context(|| {
            anyhow!("Failed to change working directory to `{}`", cwd.display())
        })?;
    }

    let mut children = Vec::new();

    let mut cmd_vecs = Vec::new();

    for (index, cmd_str) in config.start.iter().enumerate() {
        if cmd_str.is_empty() {
            bail!("The program at index `{index}` in `start` is empty.")
        }
        if let Some(cmd_vec) = shlex::split(cmd_str) {
            cmd_vecs.push(cmd_vec);
        } else {
            bail!("The program at index `{index}` in `start` is erroneous")
        }
    }

    for (index, cmd_vec) in cmd_vecs.iter().enumerate() {
        children.push(spawn_process(cmd_vec).with_context(|| {
            anyhow!(
                "Could not spawn process `{}` at index `{index}` in `start`.",
                cmd_vec[0]
            )
        })?);
    }

    if let Some(exit_on_index) = &config.exit_on {
        wait_on(&mut children, *exit_on_index)?;
        kill_remaining_children(&mut children)?;
    }

    Ok(())
}

fn spawn_process(cmd_vec: &[String]) -> anyhow::Result<Child> {
    let mut cmd: Command;

    if let Some(prog) = cmd_vec.get(0) {
        cmd = Command::new(prog);
    } else {
        bail!("A program in `start` is empty.")
    }

    const DETACHED_PROCESS: u32 = 0x00000008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .args(&cmd_vec[1..]);

    let child = cmd.spawn()?;
    Ok(child)
}

fn wait_on<T: Into<usize>>(children: &mut [Child], index: T) -> anyhow::Result<()> {
    if let Some(child) = children.get_mut(index.into()) {
        child.wait()?;
    }
    Ok(())
}

fn kill_remaining_children(children: &mut [Child]) -> anyhow::Result<()> {
    for child in children.iter_mut() {
        if child.try_wait()?.is_none() {
            if let Err(e) = child.kill() {
                bail!(anyhow!("{e}")
                    .context(anyhow!("Could not kill child with pid `{}`", child.id())))
            }
        }
    }
    Ok(())
}
