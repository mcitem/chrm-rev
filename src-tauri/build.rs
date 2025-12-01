fn main() {
    println!(
        "cargo:rustc-env={}={}",
        "UNSAFE_INVOKE_SECRET",
        hex::encode(rand::random::<[u8; 32]>())
    );

    tauri_build::build();
}
