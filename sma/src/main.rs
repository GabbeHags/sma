use std::{os::windows::process::CommandExt, process::Command};

fn hide_console_window() {
    use winapi::um::{
        wincon::GetConsoleWindow,
        winuser::{ShowWindow, SW_HIDE},
    };

    let window = unsafe { GetConsoleWindow() };
    // https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
    if !window.is_null() {
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
    }
}

fn is_started_by_double_click() -> bool {
    use sysinfo::{Pid, PidExt, ProcessExt, ProcessRefreshKind, RefreshKind, System, SystemExt};
    let me = Pid::from_u32(std::process::id());
    let sys =
        System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));

    let parent_process_name = sys
        .process(sys.process(me).unwrap().parent().unwrap())
        .unwrap()
        .name();

    parent_process_name == "explorer.exe"
}

fn main() -> anyhow::Result<()> {
    if is_started_by_double_click() {
        hide_console_window();
        let gui_path = std::env::current_exe()?
            .parent()
            .unwrap()
            .to_path_buf()
            .join("sma-gui.exe");

        if gui_path.exists() && std::env::args().len() == 1 {
            _ = Command::new(&gui_path)
                .creation_flags(sma::DETACHED_PROCESS | sma::CREATE_NEW_PROCESS_GROUP)
                .spawn()?;
        } else {
            sma::run()?;
        }
    } else {
        sma::run()?;
    }

    Ok(())
}
