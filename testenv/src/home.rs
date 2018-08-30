use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::io;
use std::fs;

/// Abstracts directory containing keys
pub struct Home {
    name: String,
}

impl Home {
    fn sandbox() -> &'static Path {
        Path::new("/tmp/testenv")
    }

    fn append_component(path: &Path, component: &str) -> PathBuf {
        let mut buf = path.to_owned();
        buf.push(component); buf
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn path(&self) -> PathBuf {
        Self::append_component(Self::sandbox(), self.name.as_str())
    }

    pub fn ext_path(&self, component: &str) -> PathBuf {
        Self::append_component(self.path().as_path(), component)
    }

    pub fn public_key_path(&self) -> PathBuf {
        self.ext_path("rpc.cert")
    }

    pub fn private_key_path(&self) -> PathBuf {
        self.ext_path("rpc.key")
    }

    pub fn new(name: &str) -> Result<Self, io::Error> {
        let s = Home {
            name: name.to_owned(),
        };

        fs::create_dir_all(s.path())
            .or_else(|err|
                if err.kind() == io::ErrorKind::NotFound { Ok(()) } else { Err(err) }
            )?;

        let _ = Command::new("gencerts")
            .current_dir(s.path()).output()?;

        Ok(s)
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        fs::remove_dir_all(self.path())
            .unwrap_or(())
    }
}
