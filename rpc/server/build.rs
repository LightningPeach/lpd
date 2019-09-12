extern crate chrono;
extern crate rustc_version;
extern crate base32;

use std::{error::Error, fmt};
use chrono::{DateTime, Utc};
use rustc_version::VersionMeta;

struct BuildInfo {
    git_hash: String,
    git_diff_base32: String,
    git_branch_name: String,
    build_time: DateTime<Utc>,
    rustc_version_meta: VersionMeta,
}

impl BuildInfo {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        use std::process::Command;

        let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()?;
        let git_hash = String::from_utf8(output.stdout)?.trim_end_matches('\n').to_owned();

        let output = Command::new("git").args(&["diff", "HEAD"]).output()?;
        let git_diff_base32 = base32::encode(base32::Alphabet::Crockford, output.stdout.as_slice());

        let output = Command::new("git").args(&["rev-parse", "--abbrev-ref", "HEAD"]).output()?;
        let git_branch_name = String::from_utf8(output.stdout)?.trim_end_matches('\n').to_owned();

        let now = Utc::now();

        let version_meta = rustc_version::version_meta()?;

        Ok(BuildInfo {
            git_hash: git_hash,
            git_diff_base32: git_diff_base32,
            git_branch_name: git_branch_name,
            build_time: now,
            rustc_version_meta: version_meta,
        })
    }

    pub fn diff_base32(&self) -> String {
        self.git_diff_base32.clone()
    }
}

impl fmt::Debug for BuildInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuildInfo")
            .field("git_hash", &self.git_hash)
            .field("git_has_diff", &!self.git_diff_base32.is_empty())
            .field("git_branch_name", &self.git_branch_name)
            .field("build_time", &self.build_time)
            .field("rustc_version", &self.rustc_version_meta.semver)
            .field("rustc_channel", &self.rustc_version_meta.channel)
            .finish()
    }
}

fn main() {
    let build_info = BuildInfo::new().unwrap();
    println!("cargo:rustc-env=GIT_DIFF={}", build_info.diff_base32());
    println!("cargo:rustc-env=BUILD_INFO={:?}", build_info);
}
