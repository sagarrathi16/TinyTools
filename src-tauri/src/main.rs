#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
fn attach_console() {
    extern "system" {
        fn AttachConsole(dwProcessId: u32) -> i32;
    }
    unsafe {
        // 0xFFFFFFFF is ATTACH_PARENT_PROCESS (-1)
        AttachConsole(0xFFFFFFFF);
    }
}

fn main() {
    let homelab = std::env::args().any(|a| a == "--serve" || a == "serve" || a == "-s")
        || std::env::var("TINYTOOLS_HOMELAB").map(|v| v == "1").unwrap_or(false);
    if homelab {
        #[cfg(target_os = "windows")]
        attach_console();

        tinytools_lib::run_homelab();
    } else {
        tinytools_lib::run();
    }
}

