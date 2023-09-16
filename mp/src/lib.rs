use probe_rs::{
    flashing::{download_file_with_options, BinOptions, DownloadOptions, Format::Bin},
    Permissions, Probe,
};
use std::{
    ffi::OsStr,
    fs::{canonicalize, File},
    io::{Read, Write},
    process::{Child, Command, Stdio},
    thread, time,
};
use tempfile::NamedTempFile;

pub fn install_uf2<S: AsRef<std::path::Path>>(path: S) {
    let mut buf: Vec<u8> = Vec::new();
    let mut file = File::open(&path).unwrap();
    file.read_to_end(&mut buf).unwrap();
    let mut temp = NamedTempFile::new().unwrap();
    let (bin, family_to_target_addr) = uf2_decode::convert_from_uf2(&buf).unwrap();
    temp.write_all(&bin).unwrap();
    assert_eq!(family_to_target_addr.len(), 1);
    assert_eq!(0xe48b_ff56, *family_to_target_addr.keys().next().unwrap());
    assert_eq!(0x1000_0000, family_to_target_addr[&0xe48b_ff56]);

    let probes = Probe::list_all();
    let probe = probes[0].open().unwrap();
    let mut session = probe.attach("rp2040", Permissions::default()).unwrap();
    let bin_options = BinOptions {
        base_address: Some(0x0000_0000_1000_0000),
        skip: 0,
    };
    temp.flush().unwrap();
    download_file_with_options(
        &mut session,
        temp.into_temp_path(),
        Bin(bin_options),
        DownloadOptions::default(),
    )
    .unwrap();
    session.core(0).unwrap().reset().unwrap();
    thread::sleep(time::Duration::new(5, 0));
}

pub struct Remote {
    dev_path: String,
}

impl Remote {
    pub fn dev_paths() -> Vec<String> {
        let mpremote_devs = Command::new("mpremote")
            .arg("devs")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .unwrap();
        let output = String::from_utf8(mpremote_devs.stdout).unwrap();
        output
            .lines()
            .filter_map(|line| {
                if line.ends_with("MicroPython Board in FS mode") {
                    Some(line.split(|b| b == ' ').next().unwrap().to_owned())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn new(dev_path: &str) -> Self {
        Self {
            dev_path: dev_path.to_owned(),
        }
    }

    fn connect(&self) -> Command {
        let mut command = Command::new("mpremote");
        command.arg("connect").arg(self.dev_path.clone());
        command
    }

    pub fn cp<S: AsRef<std::path::Path>>(&mut self, paths: Vec<S>) {
        let mut connection = self.connect();
        connection.arg("cp");
        for path in paths {
            connection.arg(canonicalize(path).unwrap());
        }
        let status = connection
            .arg(":")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success());
    }

    pub fn run<S: AsRef<OsStr>>(&mut self, path: S) -> Child {
        self.connect()
            .arg("run")
            .arg(path)
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
    }
}
