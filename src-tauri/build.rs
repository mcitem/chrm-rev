fn main() {
    #[cfg(not(target_os = "windows"))]
    {
        panic!("");
    }
    tauri_build::build();
}
