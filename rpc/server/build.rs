extern crate chrono;
extern crate rustc_version;
extern crate base32;

fn main() {
    use std::process::Command;
    use chrono::Utc;

    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    let output = Command::new("git").args(&["diff", "HEAD"]).output().unwrap();
    let git_diff = base32::encode(base32::Alphabet::Crockford, output.stdout.as_slice());
    println!("cargo:rustc-env=GIT_DIFF={}", git_diff);

    let now = Utc::now();
    println!("cargo:rustc-env=BUILD_TIME={}", now);

    let version = rustc_version::version().ok().unwrap();
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);

    let channel = rustc_version::version_meta().ok().unwrap().channel;
    println!("cargo:rustc-env=RUSTC_CHANNEL={:?}", channel);
}
