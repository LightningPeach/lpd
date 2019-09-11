extern crate chrono;
extern crate rustc_version;

fn main() {
    use std::process::Command;
    use chrono::Utc;

    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let output = Command::new("git").args(&["diff"]).output().unwrap();
    let has_diff = !output.stdout.is_empty();
    let output = Command::new("git").args(&["diff", "--cached"]).output().unwrap();
    let has_diff_cached = !output.stdout.is_empty();
    println!("cargo:rustc-env=GIT_HAS_DIFF={}", has_diff || has_diff_cached);

    let now = Utc::now();
    println!("cargo:rustc-env=BUILD_TIME={}", now);

    let version = rustc_version::version().ok().unwrap();
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);
}
