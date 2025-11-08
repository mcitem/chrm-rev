use tauri::Manager;
use tauri::Runtime;
use tauri::WebviewWindowBuilder as WWB;
use tauri::window::Color;
/// FnOnce(&mut App<R>) -> std::result::Result<(), Box<dyn std::error::Error>> + Send + 'static,
pub(super) fn setup<R>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>>
where
    R: Runtime,
{
    let index = tauri::WebviewUrl::App("index.html".into());
    let mut main = WWB::new(app, "main", index)
        .title("Chrm Rev")
        .inner_size(900_f64, 600_f64)
        .min_inner_size(800_f64, 600_f64)
        .background_color(Color(24, 24, 24, 0))
        .use_https_scheme(true)
        .auto_resize();

    {
        main = main.visible(false);
    }

    main.build()?;

    #[cfg(debug_assertions)]
    app.get_webview_window("main").unwrap().open_devtools();

    // 看门狗
    let handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        handle
            .get_webview_window("main")
            .ok_or(tauri::Error::WindowNotFound)
            .and_then(|v| v.show())
            .unwrap();
    });

    Ok(())
}
