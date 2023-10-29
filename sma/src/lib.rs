use config::{Config, Verified};

use std::{
    ffi::OsStr,
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Child, Command},
};

use anyhow::{anyhow, bail, Context, Ok};
use sysinfo::{
    Pid, PidExt, Process, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt,
};

pub fn run() -> anyhow::Result<()> {
    let config = match cli::parse_args(std::env::args())? {
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
        let config_dir = config_path.parent().unwrap();
        std::env::set_current_dir(config_dir).with_context(|| {
            anyhow!(
                "Failed to change working directory to `{}`",
                config_dir.display()
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
            if cmd_str.contains('\\') {
                if let Some(exe) = cmd_vec.get(0) {
                    if !PathBuf::from(exe).exists() {
                        bail!("There seems to be a problem with the finding the executable for the program at index `{index}`. The executable we looked for was {exe} which does not exist. This might be because \"\\\\\" was used instead of \"/\".")
                    }
                }
            }
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

fn spawn_process<S: AsRef<OsStr>>(cmd_vec: &[S]) -> anyhow::Result<Child> {
    let mut cmd: Command;

    if let Some(prog) = cmd_vec.get(0) {
        cmd = Command::new::<_>(prog.as_ref());
        #[cfg(debug_assertions)]
        dbg!(&cmd);
    } else {
        bail!("A program in `start` is empty.")
    }

    // This cfg makes it possible to see what spawned processes print to the console.
    // If false they will have there own std(in/out).
    if cfg!(debug_assertions) {
        cmd.args(&cmd_vec[1..]);
    } else {
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(
            // TODO: swap this out to the windows crate instead of winapi
            // winapi::um::winbase::DETACHED_PROCESS | winapi::um::winbase::CREATE_NEW_PROCESS_GROUP,
            DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
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

    use std::time::Duration;

    use tempdir::TempDir;

    use super::*;

    mod testbin {
        #![allow(non_upper_case_globals)]
        use test_binary::build_test_binary_once;

        build_test_binary_once!(test, "testbins");
    }

    #[test]
    fn test_spawn_processes_good() {
        let test_bin_path = testbin::path_to_test()
            .into_string()
            .unwrap()
            .replace('\\', "/");

        let config = Config::new(
            None,
            false,
            vec![format!("{} SLEEP 1", test_bin_path.clone())],
            None,
        )
        .verify()
        .unwrap();
        let now = std::time::Instant::now();
        let success = spawn_processes(&config)
            .unwrap()
            .iter_mut()
            .map(|child| child.wait().unwrap().success())
            .all(|child| child);
        let after = std::time::Instant::now();
        assert!(success);
        let diff_millis = (after - now).as_millis();
        assert!((1000..1200).contains(&diff_millis))
    }

    #[test]
    fn test_spawn_processes_fail_bad_executable_path() {
        let test_bin_path = testbin::path_to_test().into_string().unwrap();

        let config = Config::new(
            None,
            false,
            vec![format!("{} SLEEP 1", test_bin_path.clone())],
            None,
        )
        .verify()
        .unwrap();
        let err = spawn_processes(&config).unwrap_err().to_string();
        let wrong_path = test_bin_path.replace('\\', "");
        assert_eq!(
            err,
            format!("There seems to be a problem with the finding the executable for the program at index `0`. The executable we looked for was {wrong_path} which does not exist. This might be because \"\\\\\" was used instead of \"/\".")
        )
    }

    #[test]
    fn test_spawn_processes_empty() {
        let config = Config::default().verify().unwrap();
        let children = spawn_processes(&config).unwrap();
        assert_eq!(config.get_start().len(), children.len());
        assert_eq!(0, children.len());
    }

    #[test]
    fn test_spawn_process_empty() {
        let cmd: Vec<String> = vec![];
        let child_exit_status = spawn_process(&cmd);

        assert_eq!(
            "A program in `start` is empty.",
            child_exit_status.unwrap_err().to_string().as_str()
        )
    }

    #[test]
    fn test_spawn_process_one_sleep() {
        let test_bin_path = testbin::path_to_test().into_string().unwrap();
        let now = std::time::Instant::now();
        let child_exit_status = spawn_process(&[test_bin_path.as_str(), "SLEEP", "1"])
            .unwrap()
            .wait()
            .unwrap();
        let after = std::time::Instant::now();
        assert!(child_exit_status.success());
        let diff_millis = (after - now).as_millis();
        assert!((1000..1200).contains(&diff_millis))
    }

    #[test]
    fn test_spawn_process_write() {
        let test_bin_path = testbin::path_to_test();
        let file_name = "test_file_name";
        let temp_dir = TempDir::new("test_spawn_process_write").unwrap();
        let file_path = temp_dir.path().join(file_name);

        let write_content = "test";
        let child_exit_status = spawn_process(&[
            test_bin_path.as_os_str().to_str().unwrap(),
            "WRITE",
            file_path.to_str().unwrap(),
            write_content,
        ])
        .unwrap()
        .wait()
        .unwrap();
        assert!(child_exit_status.success());

        let read_content = std::fs::read_to_string(file_path.as_path()).unwrap();
        assert!(!read_content.is_empty());
        assert_eq!(write_content, &read_content);

        // cleanup
        drop(temp_dir)
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

        // cleanup
        std::env::set_current_dir(cwd).unwrap();
        drop(dir)
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

        // cleanup
        std::env::set_current_dir(cwd).unwrap();
        drop(dir)
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

        // cleanup
        std::env::set_current_dir(cwd).unwrap();
        drop(dir_1);
        drop(dir_2);
    }

    #[test]
    fn test_change_cwd_to_none_existing() {
        let cwd = std::env::current_dir().unwrap();
        let temp_dir = TempDir::new("test_change_cwd_to_none_existing").unwrap();
        let config = Config::new(Some(temp_dir.path().to_path_buf()), false, vec![], None)
            .verify()
            .unwrap();

        let removed_temp_dir = temp_dir.path().to_path_buf();

        // remove temp_dir
        drop(temp_dir);

        std::thread::sleep(Duration::from_secs(1));

        let err = change_cwd(&config).unwrap_err();

        let new_actual_cwd = std::env::current_dir().unwrap();
        assert_eq!(cwd, new_actual_cwd);

        assert_eq!(
            format!(
                "Failed to change working directory to `{}`",
                removed_temp_dir.display()
            ),
            err.to_string()
        );
    }

    #[test]
    fn test_change_cwd_none_cwd_to_none_existing() {
        let cwd = std::env::current_dir().unwrap();
        let temp_dir = TempDir::new("test_change_cwd_to_none_existing").unwrap();
        let file_name = "config.json";
        Config::new(None, false, vec![], None)
            .verify()
            .unwrap()
            .create_file(temp_dir.path().join(file_name), false)
            .unwrap();

        let config = Config::from_existing_config_file(temp_dir.path().join(file_name))
            .unwrap()
            .verify()
            .unwrap();

        let removed_temp_dir = temp_dir.path().to_path_buf();

        // remove temp_dir
        drop(temp_dir);

        std::thread::sleep(Duration::from_secs(1));

        let err = change_cwd(&config).unwrap_err();

        let new_actual_cwd = std::env::current_dir().unwrap();
        assert_eq!(cwd, new_actual_cwd);

        assert_eq!(
            format!(
                "Failed to change working directory to `{}`",
                removed_temp_dir.display()
            ),
            err.to_string()
        );
    }
}
