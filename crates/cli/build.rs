use std::env;
use std::path::PathBuf;

fn main() {
    let minver_version = calculate_minver_version();
    println!(
        "cargo:rustc-env=MINVER_CALCULATED_VERSION={}",
        minver_version
    );

    // shadow-rs generates extended build metadata (git hash, timestamps, rustc version)
    shadow_rs::ShadowBuilder::builder()
        .build()
        .expect("shadow-rs build failed");
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    manifest_dir
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

fn calculate_minver_version() -> String {
    let work_dir = workspace_root();
    let config = minver_rs::Config::default();

    match minver_rs::calculate_version(&work_dir, &config) {
        Ok(result) => result.version.to_string(),
        Err(_) => env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0-dev".to_string()),
    }
}
