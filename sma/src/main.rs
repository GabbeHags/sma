use std::{os::windows::process::CommandExt, process::Command};
use sysinfo::{Pid, PidExt, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};
use winapi::um::{
    wincon::GetConsoleWindow,
    winuser::{ShowWindow, SW_HIDE},
};

fn hide_console_window() {
    // TODO: swap this out to the windows crate instead of winapi
    let window = unsafe { GetConsoleWindow() };
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if !window.is_null() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}

fn is_started_by_double_click() -> bool {
    let me = Pid::from_u32(std::process::id());
    let sys =
        System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));

    let parent_process_name = sys
        .process(sys.process(me).unwrap().parent().unwrap())
        .unwrap()
        .name();

    parent_process_name == "explorer.exe"
}

fn start_gui_if_exist() -> anyhow::Result<bool> {
    let gui_path = std::env::current_exe()?
        .parent()
        .unwrap()
        .to_path_buf()
        .join("sma-gui.exe");

    if gui_path.try_exists().is_ok_and(|x| x) && std::env::args().len() == 1 {
        Command::new(&gui_path)
            .creation_flags(
                // TODO: swap this out to the windows crate instead of winapi
                winapi::um::winbase::DETACHED_PROCESS
                    | winapi::um::winbase::CREATE_NEW_PROCESS_GROUP,
            )
            .spawn()?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn main() -> anyhow::Result<()> {
    if !is_started_by_double_click() {
        hide_console_window();
        if !start_gui_if_exist()? {
            sma::run()?;
        }
    } else {
        sma::run()?;
    }
    Ok(())
}
