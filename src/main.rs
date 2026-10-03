#![windows_subsystem = "windows"]
mod app;
mod server;
mod theme;
mod models;
mod ui;
mod engine;
mod download_manager;
mod persistence;
#[cfg(windows)]
fn trim_memory() {
    type HANDLE = isize;
    extern "system" {
        fn GetCurrentProcess() -> HANDLE;
        fn SetProcessWorkingSetSize(
            hProcess: HANDLE,
            dwMinimumWorkingSetSize: usize,
            dwMaximumWorkingSetSize: usize,
        ) -> i32;
    }
    unsafe {
        // Asking the OS to page out unused memory to disk, dropping the RAM footprint drastically
        SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX);
    }
}

#[cfg(target_os = "linux")]
fn trim_memory() {
    extern "C" {
        fn malloc_trim(pad: usize) -> i32;
    }
    unsafe {
        malloc_trim(0);
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn trim_memory() {}

fn main() -> eframe::Result<()> {
    // Memory trimming has been disabled. Aggressive working set reduction
    // pages out active buffers which severely harms network and disk I/O performance
    // during high-speed downloads.

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Internet Download Manager")
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Internet Download Manager",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
