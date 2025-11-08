// use std::sync::atomic::AtomicBool;

use tauri::Builder;
use tauri::Runtime;

mod invoke_handler;
mod plugin;
mod run_event;
mod setup;

// pub(self) static UI_READY: AtomicBool = AtomicBool::new(false);

/// tauri Builder 会导致RA响应变慢，故放到独立mod中
pub(super) fn run_inner<R: Runtime>(b: Builder<R>) -> Result<(), tauri::Error> {
    plugin::plugins(b)
        .setup(setup::setup)
        .invoke_handler(invoke_handler::invoke_handler())
        .build(tauri::generate_context!())?
        .run(run_event::run_event);

    Ok(())
}
