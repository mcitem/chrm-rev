use crate::cmd;
use tauri::Runtime;

pub(super) fn invoke_handler<R: Runtime>()
-> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        cmd::open_web_url,
        cmd::open_core_dir,
        cmd::open_data_dir,
        cmd::open_devtools,
        cmd::open_logs_dir,
        cmd::open_timedate_cpl,
        cmd::sudo,
        cmd::backup,
    ]
}
