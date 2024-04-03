use nix::fcntl::{open, OFlag};
use nix::pty::{grantpt, posix_openpt, ptsname, unlockpt};
use nix::sys::stat::Mode;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::path::Path;
use std::thread::spawn;

fn main() {
    let mut master_fd = posix_openpt(OFlag::O_RDWR).unwrap();

    let slave_name = unsafe { ptsname(&master_fd) }.unwrap();

    grantpt(&master_fd).unwrap();
    unlockpt(&master_fd).unwrap();

    let slave_fd = open(Path::new(&slave_name), OFlag::O_RDWR, Mode::empty()).unwrap();

    spawn(move || loop {
        let mut buffer = [0u8; 1024];
        master_fd.read(buffer.as_mut_slice()).unwrap();

        println!("received data: {}", String::from_utf8_lossy(&buffer));
    });

    let mut slave_file = unsafe { File::from_raw_fd(slave_fd) };
    loop {
        slave_file
            .write("ls -l\n".as_bytes())
            .expect("Failed to write to slave");

        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
