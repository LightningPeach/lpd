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
    let path = std::path::Path::new("target/wasm32-unknown-unknown/debug/examples");
    let result_html_path = {
        let mut temp = std::path::PathBuf::from(path);
        temp.push("wasm-basic.html");
        temp
    };

    let _ = std::process::Command::new("cargo")
        .args(&["build", "--target=wasm32-unknown-unknown", "--example", "wasm-basic"])
        .output().unwrap();
    let _ = std::process::Command::new("wasm-gc")
        .current_dir(path)
        .args(&["wasm-basic.wasm", "stripped.wasm"])
        .output().unwrap();
    let _ = std::fs::copy("examples/wasm-basic.html", result_html_path)
        .unwrap();
    println!("visit http://localhost:8000/wasm-basic.html, press ctrl+c to quit");
    let _ = std::process::Command::new("python3")
        .current_dir(path)
        .args(&["-m", "http.server", "8000"])
        .output().unwrap();
}
