use mp::Remote;
use std::{
    ffi::OsStr,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};

const MICROPYTHON_PATH: &str = "micropython.uf2";

pub fn run_example<S: AsRef<OsStr>>(example: S) -> impl Iterator<Item = String> {
    let child = Command::new("cargo")
        .arg("pe")
        .arg(example)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    let buf_reader = BufReader::new(child.stdout.unwrap());
    buf_reader.lines().map(|f| f.unwrap())
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
