use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::io;
use std::fs;

/// Abstracts directory containing keys
#[derive(Debug)]
pub struct Home {
    name: String,

    // should directory with files be deleted in the end
    // when Home struct is dropped
    cleanup_files: bool
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
        // By default lnd and gencerts use tls.cert file
        self.ext_path("tls.cert")
    }

    pub fn private_key_path(&self) -> PathBuf {
        // By default lnd and gencerts use tls.key file
        self.ext_path("tls.key")
    }

    pub fn lnd_conf_path(&self) -> PathBuf {
        self.ext_path("lnd.conf")
    }

    pub fn new(name: &str, force: bool, cleanup_files: bool) -> Result<Self, io::Error> {
        let s = Home {
            name: name.to_owned(),
            cleanup_files: cleanup_files,
        };

        fs::create_dir_all(s.path())
            .or_else(|err|
                if err.kind() == io::ErrorKind::NotFound {
                    println!("ignoring, path not found during create_all for home dir: {:?}", s.path());
                    Ok(())
                } else {
                    Err(err)
                }
            )
            .map_err(|err| {
                println!("error creating home dir: {:?} {:?}", s.path(), err);
                err
            })?;

        let lock_path = s.ext_path(".lock");
        if force {
            fs::remove_file(&lock_path)
                .or_else(|e| if e.kind() == io::ErrorKind::AlreadyExists {
                    println!("ignoring, cannot create lock file because it already exists: {:?}", &lock_path);
                    Ok(())
                } else {
                    Err(e)
                })
                .map_err(|err|{
                    println!("cannot create lock file: {:?} {:?}", &lock_path, err);
                    err
                })?;
        }
        fs::File::create(&lock_path).map_err( |err| {
            println!("cannot create lock file: {:?} {:?}", lock_path, err);
            err
        })?;

        // lnd tries to open default config file if it is unspecified in its options
        // so we create empty one
        use std::io::Write;
        let lnd_config_path = s.lnd_conf_path();
        let mut lnd_conf_file = fs::File::create(&lnd_config_path)
            .map_err(|err| {
                println!("cannot create lnd config file: {:?} {:?}", &lnd_config_path, err);
                err
            })?;
        lnd_conf_file.write_all(b"[Application Options]\n")
            .map_err(|err| {
                println!("cannot write to lnd config file: {:?} {:?}", &lnd_config_path, err);
                err
            })?;

        // We do not need to generate tls certificates because lnd generates them automatically
        // if they are not present on specified path

        Ok(s)
    }
}

impl Drop for Home {
    fn drop(&mut self) {
        if self.cleanup_files {
            match fs::remove_dir_all(self.path()) {
                Err(err) => println!("error deleting (final cleanup) home directory: {:?} {:?}", self.path(), err),
                Ok(_) => println!("home directory deleted: {:?}", self.path())
            }
        }
    }
}

// creates file and returns stdio object for it,
// so it can be used as redirect for stdout or stderr for command
// https://stackoverflow.com/questions/43949612/redirect-output-of-child-process-spawned-from-rust
// File object is returned because otherwise it became dropped
// which closes the file, making the descriptor invalid
pub fn create_file_for_redirect(p: PathBuf) -> io::Result<(std::process::Stdio, std::fs::File)> {
    use std::process::Stdio;
    use std::os::unix::io::AsRawFd;
    use std::os::unix::io::FromRawFd;
    let f = std::fs::File::create(p).map_err(|err| {
            println!("cannot crate file for redirect: {:?}", err);
            err
    })?;
    let fd = f.as_raw_fd();
    // from_raw_fd is only considered unsafe if the file is used for mmap
    let stdio = unsafe {Stdio::from_raw_fd(fd)};
    Ok((stdio, f))
}
