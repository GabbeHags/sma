use crate::my_lib::{
    cli::Cli,
    config::{Config, Verified},
};

use std::{
    os::windows::process::CommandExt,
    process::{Child, Command},
};

use anyhow::{anyhow, bail, Context, Ok};
use clap::Parser;
use sysinfo::{
    Pid, PidExt, Process, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt,
};

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::from_cli(cli).with_context(|| anyhow!("Failed to verify the config."))?;
    change_cwd(&config)?;

    let mut children = spawn_processes(&config)?;

    wait_and_kill(&config, &mut children)?;

    Ok(())
}

fn change_cwd(config: &Config<Verified>) -> anyhow::Result<()> {
    if let Some(cwd) = &config.cwd {
        std::env::set_current_dir(cwd).with_context(|| {
            anyhow!("Failed to change working directory to `{}`", cwd.display())
        })?;
    }
    Ok(())
}

fn wait_and_kill(config: &Config<Verified>, children: &mut [Child]) -> anyhow::Result<()> {
    if let Some(exit_on_index) = &config.exit_on {
        wait_on(children, *exit_on_index)?;
        if config.cascade_kill {
            kill_remaining_children_cascade(children)?;
        } else {
            kill_remaining_children(children)?;
        }
    }

    Ok(())
}

fn spawn_processes(config: &Config<Verified>) -> anyhow::Result<Vec<Child>> {
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

    Ok(children)
}

fn kill_remaining_children_cascade(children: &mut [Child]) -> anyhow::Result<()> {
    let mut sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );

    sys.refresh_processes();
    let me = Pid::from_u32(std::process::id());

    let mut cascaded_pids = vec![vec![]];
    let mut updated = false;
    let mut layer = 0;
    let processes: Vec<(&Pid, &Process)> = sys
        .processes()
        .iter()
        .filter(|(_, proc)| sys.process(me).unwrap().start_time() <= proc.start_time())
        .collect();

    for (pid, proc) in &processes {
        for child in children.iter() {
            if let Some(parent) = proc.parent().map(|parent| parent.as_u32()) {
                if parent == child.id() && me.as_u32() != parent {
                    cascaded_pids[layer].push(pid.as_u32());
                    updated = true;
                }
            }
        }
    }
    while updated {
        updated = false;
        layer += 1;
        let mut new_pids_found = vec![];

        for cascade_pid in &cascaded_pids[layer - 1] {
            for (pid, proc) in &processes {
                if let Some(parent) = proc.parent().map(|parent| parent.as_u32()) {
                    if parent == *cascade_pid {
                        new_pids_found.push(pid.as_u32());
                        updated = true;
                    }
                }
            }
        }
        cascaded_pids.push(new_pids_found);
    }

    kill_remaining_children(children)?;

    sys.refresh_processes();
    for proc in cascaded_pids
        .iter()
        .flatten()
        .filter_map(|pid| sys.process(Pid::from_u32(*pid)))
    {
        println!("Killing sub child: [{}] {}", proc.pid(), proc.name());
        proc.kill();
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

    if cfg!(debug_assertions) {
        cmd.args(&cmd_vec[1..]);
    } else {
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
            .args(&cmd_vec[1..]);
    }

    let child = cmd.spawn()?;
    println!("Spawning child: {}", child.id());
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
            println!("Killing child: {}", child.id());
            if let Err(e) = child.kill() {
                bail!(anyhow!("{e}")
                    .context(anyhow!("Could not kill child with pid `{}`", child.id())))
            }
        }
    }
    Ok(())
}
