use config::{Config, Verified};

use std::{
    os::windows::process::CommandExt,
    process::{Child, Command},
};

use anyhow::{anyhow, bail, Context, Ok};
use sysinfo::{
    Pid, PidExt, Process, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt,
};

pub const DETACHED_PROCESS: u32 = 0x00000008;
pub const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

pub fn run() -> anyhow::Result<()> {
    let config = match cli::get_args() {
        cli::Commands::Start { start, exit_on } => {
            Config::new(None, false, start, exit_on).verify()
        }
        cli::Commands::Config { file_path } => {
            Config::from_existing_config_file(file_path)?.verify()
        }
        cli::Commands::CreateConfig {
            file_path,
            force_overide,
        } => {
            // Creates a new file and wants to exit the program gracefully
            return Config::new_config_to_file(file_path, force_overide);
        }
    }
    .with_context(|| anyhow!("Failed to verify the config."))?;

    change_cwd(&config)?;

    let mut children = spawn_processes(&config)?;

    wait_and_kill(&config, &mut children)?;

    Ok(())
}

fn change_cwd(config: &Config<Verified>) -> anyhow::Result<()> {
    if let Some(cwd) = config.get_cwd() {
        std::env::set_current_dir(cwd).with_context(|| {
            anyhow!("Failed to change working directory to `{}`", cwd.display())
        })?;
    } else if let Some(config_path) = config.get_config_file_path() {
        std::env::set_current_dir(config_path.parent().unwrap()).with_context(|| {
            anyhow!(
                "Failed to change working directory to `{}`",
                config_path.display()
            )
        })?;
    }

    Ok(())
}

fn wait_and_kill(config: &Config<Verified>, children: &mut [Child]) -> anyhow::Result<()> {
    if let Some(exit_on_index) = config.get_exit_on() {
        wait_on(children, exit_on_index)?;
        if config.get_cascade_kill() {
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

    for (index, cmd_str) in config.get_start().iter().enumerate() {
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
        #[cfg(debug_assertions)]
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
        cmd.creation_flags(
            // TODO: swap this out to the windows crate instead of winapi
            winapi::um::winbase::DETACHED_PROCESS | winapi::um::winbase::CREATE_NEW_PROCESS_GROUP,
        )
        .args(&cmd_vec[1..]);
    }

    let child = cmd.spawn()?;

    #[cfg(debug_assertions)]
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
            #[cfg(debug_assertions)]
            println!("Killing child: {}", child.id());

            if let Err(e) = child.kill() {
                bail!(anyhow!("{e}")
                    .context(anyhow!("Could not kill child with pid `{}`", child.id())))
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test_sma {

    use tempdir::TempDir;

    use super::*;

    const TEST_EXE: &str = concat!(env!("OUT_DIR"), "/test.exe");

    #[test]
    fn test_spawn_process() {
        let now = std::time::Instant::now();
        let child = spawn_process(&[TEST_EXE.to_string(), "SLEEP".to_string(), "2".to_string()])
            .unwrap()
            .wait()
            .unwrap();
        assert!(child.success());
        let after = std::time::Instant::now();
        let diff_millis = (after - now).as_millis();
        assert!((2000..2300).contains(&diff_millis))

        // println!("{:?}", stdout);
        // assert!(output.status.success());
    }

    #[test]
    fn test_change_cwd_config_path_none_cwd_none() {
        let cwd = std::env::current_dir().unwrap();
        let config = Config::new(None, false, vec![], None).verify().unwrap();
        change_cwd(&config).unwrap();
        assert_eq!(cwd, std::env::current_dir().unwrap());
    }

    #[test]
    fn test_change_cwd_config_path_none_cwd_some() {
        let cwd = std::env::current_dir().unwrap();
        let dir = TempDir::new("test_change_cwd_config_path_none_cwd_some").unwrap();
        let config_path = dir.path().join("config.json");
        let config = Config::new(
            Some(config_path.parent().unwrap().to_path_buf()),
            false,
            vec![],
            None,
        )
        .verify()
        .unwrap();
        change_cwd(&config).unwrap();
        let new_cwd = std::env::current_dir().unwrap();
        assert_ne!(cwd, new_cwd);
        assert_eq!(config_path.parent().unwrap(), new_cwd);
    }

    #[test]
    fn test_change_cwd_config_path_some_cwd_none() {
        let cwd = std::env::current_dir().unwrap();
        let dir = TempDir::new("test_change_cwd_config_path_some_cwd_none").unwrap();
        let config_path = dir.path().join("config.json");
        Config::default()
            .verify()
            .unwrap()
            .create_file(&config_path, false)
            .unwrap();
        let config = Config::from_existing_config_file(&config_path)
            .unwrap()
            .verify()
            .unwrap();
        change_cwd(&config).unwrap();
        let new_cwd = std::env::current_dir().unwrap();
        assert_ne!(cwd, new_cwd);
        assert_eq!(config_path.parent().unwrap(), new_cwd);
    }

    #[test]
    fn test_change_cwd_config_path_some_cwd_some() {
        let cwd = std::env::current_dir().unwrap();
        let dir_1 = TempDir::new("test_change_cwd_config_path_some_cwd_none_1").unwrap();
        let dir_2 = TempDir::new("test_change_cwd_config_path_some_cwd_none_2").unwrap();
        let config_path = dir_1.path().join("config.json");
        let new_cwd = dir_2.path();
        Config::new(Some(new_cwd.to_path_buf()), false, vec![], None)
            .verify()
            .unwrap()
            .create_file(&config_path, false)
            .unwrap();

        let config = Config::from_existing_config_file(&config_path)
            .unwrap()
            .verify()
            .unwrap();
        change_cwd(&config).unwrap();
        let new_actual_cwd = std::env::current_dir().unwrap();
        assert_ne!(cwd, new_actual_cwd);
        assert_eq!(new_actual_cwd, new_cwd);
    }
}
