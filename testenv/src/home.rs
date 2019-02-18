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

    pub fn lnd_conf_path(&self) -> PathBuf {
        self.ext_path("lnd.conf")
    }

    pub fn new(name: &str, force: bool) -> Result<Self, io::Error> {
        let s = Home {
            name: name.to_owned(),
        };

        fs::create_dir_all(s.path())
            .or_else(|err|
                if err.kind() == io::ErrorKind::NotFound { Ok(()) } else { Err(err) }
            )?;

        let lock_path = s.ext_path(".lock");
        if force {
            fs::remove_file(&lock_path)
                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(e)
                })?;
        }
        fs::File::create(lock_path)?;

        // lnd tries to open default config file if it is unspecified in its options
        // so we create empty one
        use std::io::Write;
        let lnd_config_path = s.lnd_conf_path();
        let mut lnd_conf_file = fs::File::create(lnd_config_path)?;
        lnd_conf_file.write_all(b"[Application Options]\n")?;

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
