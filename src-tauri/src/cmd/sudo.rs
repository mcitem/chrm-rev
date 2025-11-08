use tauri::{AppHandle, Emitter, Listener, Manager, Runtime};
use tokio::sync::oneshot;
use ts_rs::TS;

/// bool: 是否记住授权结果
pub struct SUDO;

#[derive(serde::Deserialize, TS)]
#[ts(export, export_to = "sudo.ts")]
struct SudoResponse<'a> {
    remember: bool,
    password: &'a str,
}

#[tauri::command]
pub async fn sudo<R: Runtime>(app: AppHandle<R>) -> bool {
    if let None = app.try_state::<SUDO>() {
        let Ok(()) = app.emit("mcitem://sudo/request", ()) else {
            return false;
        };

        let app_clone = app.clone();
        let (tx, rx) = oneshot::channel::<bool>();
        app.once("mcitem://sudo/response", move |event| {
            if let Ok(res) = serde_json::from_str::<SudoResponse>(event.payload()) {
                if res.password == "111" {
                    tx.send(true).ok();
                    if res.remember {
                        app_clone.manage(SUDO);
                    }
                }
            } else {
                tx.send(false).ok();
            }
        });

        tokio::select! {
            res = rx => {
                if let Ok(result) = res {
                    return result;
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {
            }
        }

        return false;
    };
    true
}
