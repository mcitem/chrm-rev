use std::sync::atomic::AtomicBool;

use std::sync::atomic::Ordering;
use tauri::Manager;
use tauri::WindowEvent;
use tauri::{AppHandle, RunEvent, Runtime};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_pinia::ManagerExt;

use crate::config;
use crate::config::CONFIG;

/// FnMut(&AppHandle<R>, RunEvent) + 'static
/// tips: run_event 主线程
pub(super) fn run_event<R: Runtime>(app: &AppHandle<R>, event: RunEvent) {
    match event {
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::CloseRequested { api, .. },
            ..
        } => {
            // if label == "sudo" {
            //     unsafe {
            //         let main = app.get_webview_window("main").unwrap();
            //         let main_hwnd = main.hwnd().unwrap().0 as _;
            //         EnableWindow(main_hwnd, 1);
            //     }
            // };

            if label == "main" {
                api.prevent_close();

                static EXIT_DIALOG: AtomicBool = AtomicBool::new(false);
                if EXIT_DIALOG.load(Ordering::SeqCst) {
                    return;
                }
                let handle = app.clone();
                EXIT_DIALOG.store(true, Ordering::SeqCst);
                let main = app.get_webview_window("main");
                if let Some(main) = main {
                    app.dialog()
                        .message("是否退出应用？")
                        .title("Chrm Rev")
                        .buttons(tauri_plugin_dialog::MessageDialogButtons::YesNo)
                        .parent(&main)
                        .show(move |result| {
                            if result {
                                handle.pinia().save_all_now().ok();
                                tauri::async_runtime::block_on(async {
                                    // 仅在release下退出保存配置
                                    if !cfg!(debug_assertions) {
                                        CONFIG.save().await.ok();
                                    }
                                });

                                main.close().ok();
                                // drop(main);

                                handle.cleanup_before_exit();
                                handle.exit(0);
                            }
                            EXIT_DIALOG.store(false, Ordering::SeqCst);
                        });
                }
            }
        }
        _ => {}
    }
}
