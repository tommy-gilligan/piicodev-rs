#![cfg(feature = "acceptance")]
use mp::Remote;
use std::{
    ffi::OsStr,
    io::{BufRead, BufReader},
    path::Path,
    process::{Child, Command, Stdio},
};

const MICROPYTHON_PATH: &str = "micropython.uf2";

pub struct Example {
    child: Child,
}

impl Example {
    pub fn run_micropython<T: AsRef<Path>, S: AsRef<OsStr>>(
        example: S,
        dependencies: Vec<T>,
    ) -> Self {
        mp::install_uf2(MICROPYTHON_PATH);
        let dev_paths = Remote::dev_paths();
        let mut remote = Remote::new(dev_paths.first().unwrap());
        remote.cp(dependencies);
        Self {
            child: remote.run(example),
        }
    }

    pub fn run<S: AsRef<OsStr>>(example: S) -> Self {
        Self {
            child: Command::new("cargo")
                .arg("pe")
                .arg(example)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        }
    }

    pub fn output(&mut self) -> impl Iterator<Item = String> + '_ {
        let buf_reader = BufReader::new(self.child.stdout.as_mut().unwrap());
        buf_reader.lines().map(Result::unwrap)
    }
}

impl Drop for Example {
    fn drop(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }
}
