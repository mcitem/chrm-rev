use tauri::{Manager, Runtime};
pub(super) fn plugins<R: Runtime>(b: tauri::Builder<R>) -> tauri::Builder<R> {
    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    b.plugin(
        tauri_plugin_log::Builder::new()
            .level(level)
            .filter(|metadata| {
                // https://github.com/tauri-apps/tauri/issues/8494
                metadata.target() != "tao::platform_impl::platform::event_loop::runner"
            })
            .build(),
    )
    .plugin(tauri_plugin_single_instance::init(|app, _, _| {
        app.get_webview_window("main").and_then(|v| {
            v.unminimize().ok();
            v.set_focus().ok()
        });
    }))
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_process::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_pinia::init())
}
