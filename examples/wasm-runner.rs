#!/usr/bin/env run-cargo-script

// run in order to use this file as executable
// $ cargo install cargo-script

// wasm-gc is required, run
// $ cargo install wasm-gc

// run with
// $ cargo run --example wasm-runner

// python3 is required

// http://localhost:8000/wasm-basic.html should output "6" in console

fn main() {
    let _ = std::process::Command::new("cargo")
        .args(&["build", "--target=wasm32-unknown-unknown", "--example", "wasm-basic"])
        .output().unwrap();
    let _ = std::process::Command::new("wasm-gc")
        .current_dir("target/wasm32-unknown-unknown/debug/examples")
        .args(&["wasm-basic.wasm", "stripped.wasm"])
        .output().unwrap();
    let _ = std::fs::copy("examples/wasm-basic.html", "target/wasm32-unknown-unknown/debug/examples/wasm-basic.html")
        .unwrap();
    let _ = std::process::Command::new("python3")
        .current_dir("target/wasm32-unknown-unknown/debug/examples")
        .args(&["-m", "http.server", "8000"])
        .output().unwrap();
}
