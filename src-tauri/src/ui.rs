pub mod invoke_handler;
pub mod plugin;
pub mod run_event;
pub mod setup;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 防止休眠
    unsafe {
        use windows_sys::Win32::System::Power;
        // https://learn.microsoft.com/zh-cn/windows/win32/api/winbase/nf-winbase-setthreadexecutionstate
        Power::SetThreadExecutionState(
            Power::ES_CONTINUOUS | Power::ES_DISPLAY_REQUIRED | Power::ES_SYSTEM_REQUIRED,
        );
    };

    Ok(plugin::plugins(tauri::Builder::default())
        .setup(setup::setup)
        .invoke_handler(invoke_handler::invoke_handler())
        .build(tauri::generate_context!())?
        .run(run_event::run_event))
}
