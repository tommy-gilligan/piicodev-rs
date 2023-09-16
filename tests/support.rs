use mp::Remote;
use std::{
    ffi::OsStr,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Child, Stdio},
};

const MICROPYTHON_PATH: &str = "micropython.uf2";

pub struct Example {
    child: Child
}

impl Example {
    pub fn run<S: AsRef<OsStr>>(example: S) -> Self {
        Self {
            child: Command::new("cargo")
                .arg("pe")
                .arg(example)
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap()
        }
    }

    pub fn output<'a>(&'a mut self) -> impl Iterator<Item = String> + 'a {
        let buf_reader = BufReader::new(self.child.stdout.as_mut().unwrap());
        buf_reader.lines().map(|f| f.unwrap())
    }
}

impl Drop for Example {
    fn drop(&mut self) {
        self.child.kill().ok();
        self.child.wait().ok();
    }
}

pub fn run_micropython_example<S: AsRef<Path>, T: AsRef<OsStr>>(
    dependencies: Vec<S>,
    example: T,
) -> impl Iterator<Item = String> {
    mp::install_uf2(MICROPYTHON_PATH);
    let dev_paths = Remote::dev_paths();
    let mut remote = Remote::new(dev_paths.first().unwrap());
    remote.cp(dependencies);
    remote.run(example)
}
