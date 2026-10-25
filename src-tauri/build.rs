#[path = "src/config.rs"]
#[allow(dead_code)]
mod config;

fn main() {
    println!("cargo:rerun-if-changed=../config.json");
    println!("cargo:rerun-if-changed=src/config.rs");

    let raw = std::fs::read_to_string("../config.json")
        .expect("[CONFIG][FATAL] config.json is missing from the repository root.");
    if let Err(err) = serde_json::from_str::<config::AppConfig>(&raw) {
        panic!("[CONFIG][FATAL] config.json is invalid: {err}");
    }

    tauri_build::build()
}
