use std::{error::Error, fmt};
use chrono::{DateTime, Utc, NaiveDateTime};
use rustc_version::{VersionMeta as VersionMetaRust, Channel as ChannelRust};
use semver::{Identifier as IdentifierRust, Version as VersionRust};
use serde::{Serialize, Deserialize};

/// An identifier in the pre-release or build metadata.
///
/// See sections 9 and 10 of the spec for more about pre-release identifers and
/// build metadata.
/// It is a copy-paste of semver::Identifier` to add `Serialize` and `Deserialize`
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum Identifier {
    /// An identifier that's solely numbers.
    Numeric(u64),
    /// An identifier with letters and numbers.
    AlphaNumeric(String),
}

impl From<Identifier> for IdentifierRust {
    fn from(a: Identifier) -> IdentifierRust {
        match a {
            Identifier::Numeric(x) => IdentifierRust::Numeric(x),
            Identifier::AlphaNumeric(s) => IdentifierRust::AlphaNumeric(s),
        }
    }
}

impl From<IdentifierRust> for Identifier {
    fn from(a: IdentifierRust) -> Identifier {
        match a {
            IdentifierRust::Numeric(x) => Identifier::Numeric(x),
            IdentifierRust::AlphaNumeric(s) => Identifier::AlphaNumeric(s),
        }
    }
}

/// Represents a version number conforming to the semantic versioning scheme.
/// It is a copy paste from `semver::Version` to add `Serialize` and `Deserialize`
#[derive(Clone, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub struct Version {
    /// The major version, to be incremented on incompatible changes.
    pub major: u64,
    /// The minor version, to be incremented when functionality is added in a
    /// backwards-compatible manner.
    pub minor: u64,
    /// The patch version, to be incremented when backwards-compatible bug
    /// fixes are made.
    pub patch: u64,
    /// The pre-release version identifier, if one exists.
    pub pre: Vec<Identifier>,
    /// The build metadata, ignored when determining version precedence.
    pub build: Vec<Identifier>,
}

impl From<VersionRust> for Version {
    fn from(v: VersionRust) -> Version {
        Version {
            major: v.major,
            minor: v.minor,
            patch: v.patch,
            pre: v.pre.into_iter().map(|x| x.into()).collect(),
            build: v.build.into_iter().map(|x| x.into()).collect()
        }
    }
}

impl From<Version> for VersionRust {
    fn from(v: Version) -> VersionRust {
        VersionRust {
            major: v.major,
            minor: v.minor,
            patch: v.patch,
            pre: v.pre.into_iter().map(|x| x.into()).collect(),
            build: v.build.into_iter().map(|x| x.into()).collect()
        }
    }
}

/// Release channel of the compiler.
/// It is a copy-paste from `rustc_version::Channel` to add `Serialize` and `Deserialize`
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum Channel {
    /// Development release channel
    Dev,
    /// Nightly release channel
    Nightly,
    /// Beta release channel
    Beta,
    /// Stable release channel
    Stable,
}

impl From<ChannelRust> for Channel {
    fn from(c: ChannelRust) -> Channel {
        match c {
            ChannelRust::Dev => Channel::Dev,
            ChannelRust::Nightly => Channel::Nightly,
            ChannelRust::Beta => Channel::Beta,
            ChannelRust::Stable => Channel::Stable
        }
    }
}

impl From<Channel> for ChannelRust {
    fn from(c: Channel) -> ChannelRust {
        match c {
            Channel::Dev => ChannelRust::Dev,
            Channel::Nightly => ChannelRust::Nightly,
            Channel::Beta => ChannelRust::Beta,
            Channel::Stable => ChannelRust::Stable
        }
    }
}

/// Rustc version plus metada like git short hash and build date.
/// It is a copy-paste from `rustc_version::Version` to add `Serialize` and `Deserialize`
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct VersionMeta {
    /// Version of the compiler
    pub semver: Version,

    /// Git short hash of the build of the compiler
    pub commit_hash: Option<String>,

    /// Commit date of the compiler
    pub commit_date: Option<String>,

    /// Build date of the compiler; this was removed between Rust 1.0.0 and 1.1.0.
    pub build_date: Option<String>,

    /// Release channel of the compiler
    pub channel: Channel,

    /// Host target triple of the compiler
    pub host: String,

    /// Short version string of the compiler
    pub short_version_string: String,
}

impl From<VersionMetaRust> for VersionMeta {
    fn from(v: VersionMetaRust) -> VersionMeta {
        VersionMeta {
            semver: v.semver.into(),
            commit_hash: v.commit_hash,
            commit_date: v.commit_date,
            build_date: v.build_date,
            channel: v.channel.into(),
            host: v.host,
            short_version_string: v.short_version_string,
        }
    }
}

impl From<VersionMeta> for VersionMetaRust {
    fn from(v: VersionMeta) -> VersionMetaRust {
        VersionMetaRust {
            semver: v.semver.into(),
            commit_hash: v.commit_hash,
            commit_date: v.commit_date,
            build_date: v.build_date,
            channel: v.channel.into(),
            host: v.host,
            short_version_string: v.short_version_string,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Contains information about build process like git commit, etc...
pub struct BuildInfo {
    /// Hash of git commit
    pub git_hash: String,

    /// Difference between commit and working tree
    pub git_diff: String,

    /// Name of the git branch
    pub git_branch_name: String,

    /// Build time "UNIX timestamp" in seconds
    pub build_time: i64,

    /// Information about compiler
    pub rustc_version_meta: VersionMeta,
}

impl BuildInfo {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        use std::process::Command;

        let git_hash = {
            let output = Command::new("git").args(&["rev-parse", "HEAD"]).output()
                .map_err(|err| format!("Cannot run `git rev-parse HEAD` : {:?}", err))?;
            String::from_utf8_lossy(&output.stdout).trim_end_matches('\n').to_owned()
        };

        let git_diff = {
            let output = Command::new("git").args(&["diff", "HEAD"]).output()
                .map_err(|err| format!("Cannot run `git diff HEAD` : {:?}", err))?;
            String::from_utf8_lossy(&output.stdout).to_string()
        };


        let git_branch_name = {
            let output = Command::new("git").args(&["rev-parse", "--abbrev-ref", "HEAD"]).output()
                .map_err(|err| format!("Cannot run `git rev-parse --abbrev-ref HEAD` : {:?}", err))?;
            String::from_utf8_lossy(&output.stdout).trim_end_matches('\n').to_owned()
        };

        let now = Utc::now();

        let version_meta: VersionMeta = rustc_version::version_meta()?.into();

        Ok(BuildInfo {
            git_hash,
            git_diff,
            git_branch_name,
            build_time: now.timestamp(),
            rustc_version_meta: version_meta,
        })
    }

    /// Serialize structure to binary format and then encode using base32
    pub fn to_base32(&self) -> String {
        let encoded = bincode::serialize(self).unwrap();
        base32::encode(base32::Alphabet::Crockford, &encoded)
    }

    /// Deserialize structure from base32 encoded binary format
    pub fn from_base32(s: &str) -> Result<Self, Box<dyn Error>> {
        let b = base32::decode(base32::Alphabet::Crockford, s)
            .ok_or(format!("cannot decode from base32"))?;
        let build_info: BuildInfo = bincode::deserialize(&b)
            .map_err(|err| format!("cannot deserialize build info using bincode: {:?}", err))?;
        Ok(build_info)
    }
}

impl fmt::Debug for BuildInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let build_time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.build_time, 0), Utc);
        let mut r = f.debug_struct("BuildInfo");
        r
            .field("git_hash", &self.git_hash)
            .field("git_has_diff", &!self.git_diff.is_empty())
            .field("git_branch_name", &self.git_branch_name)
            .field("build_time", &build_time.to_rfc2822())
            .field("rustc_version", &self.rustc_version_meta.semver)
            .field("rustc_channel", &self.rustc_version_meta.channel);
        if !self.git_diff.is_empty() {
            r.field("git_diff", &self.git_diff);
        }
        r.finish()
    }
}

/// Call this function in `build.rs` to include build info.
/// Then you can use `get_build_info!()`
pub fn include_build_info() {
    let build_info = BuildInfo::new().unwrap();
    println!("cargo:rustc-env=BUILD_INFO={}", build_info.to_base32());
}

/// Returns `BuildInfo`
#[macro_export]
macro_rules! get_build_info {
    () => {{
        use $crate::BuildInfo;
        BuildInfo::from_base32(env!("BUILD_INFO")).unwrap()
    }};
}

#[cfg(test)]
mod test {
    use rustc_version::{VersionMeta as VersionMetaRust};
    use crate::{BuildInfo, VersionMeta, Version, Identifier, Channel};

    fn new_build_info() -> BuildInfo {
        BuildInfo {
            git_hash: "some hash".to_owned(),
            git_diff: "some diff".to_owned(),
            git_branch_name: "some branch name".to_owned(),
            build_time: 12312,
            rustc_version_meta: VersionMeta {
                semver: Version {
                    major: 1,
                    minor: 2,
                    patch: 3,
                    pre: vec![Identifier::Numeric(1), Identifier::AlphaNumeric("b2".to_owned())],
                    build: vec![Identifier::AlphaNumeric("c3".to_owned())]
                },
                commit_hash: Some("some rustc commit hash".to_owned()),
                commit_date: Some("some rustc commit date".to_owned()),
                build_date: Some("some rustc build date".to_owned()),
                channel: Channel::Nightly,
                host: "wasm32-unknown-unknown".to_owned(),
                short_version_string: "some version string".to_owned(),
            }
        }
    }

    #[test]
    fn test_encode_decode() {
        let info = new_build_info();
        let s = info.to_base32();
        let info2 = BuildInfo::from_base32(&s).unwrap();
        assert_eq!(info, info2);
    }

    #[test]
    fn test_version_meta_convert() {
        let info = new_build_info();
        let version_meta_rust: VersionMetaRust = info.rustc_version_meta.clone().into();
        let version_meta: VersionMeta = version_meta_rust.into();
        assert_eq!(info.rustc_version_meta, version_meta)
    }
}