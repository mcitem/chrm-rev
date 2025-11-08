// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let runner = std::panic::catch_unwind(|| chrm_rev_lib::run());
    match runner {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            rfd::MessageDialog::new()
                .set_title("启动失败")
                .set_description(format!("chrm_rev 启动失败 \n{e}"))
                .set_buttons(rfd::MessageButtons::OkCustom(String::from("退出")))
                .set_level(rfd::MessageLevel::Error)
                .show();
        }
        Err(panic_info) => {
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "未知 panic原因".to_string()
            };
            rfd::MessageDialog::new()
                .set_title("程序崩溃")
                .set_description(format!(
                    "程序发生 panic \n 这意味着可能遇到了无法自行恢复的错误 \n {msg}\n有关日志的更多信息，请查询数据目录"
                ))
                .set_buttons(rfd::MessageButtons::OkCustom(String::from("退出")))
                .set_level(rfd::MessageLevel::Error)
                .show();

            std::panic::resume_unwind(panic_info)
        }
    }
}
