use nix::libc::waitpid;
use nix::pty::openpty;
use nix::unistd::{fork, ForkResult};
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::process::{exit, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

fn main() {
    let pty = openpty(None, None).expect("openpty failed");

    let (master, slave) = (pty.master, pty.slave);

    let mut command = Command::new("zsh");
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child: pid, .. }) => {
            spawn(move || {
                unsafe { waitpid(i32::from(pid), &mut 0, 0) };
                println!("process exit!");
                exit(0);
            });
        }
        Ok(ForkResult::Child) => {
            command
                .stdin(unsafe { Stdio::from_raw_fd(slave.as_raw_fd()) })
                .stdout(unsafe { Stdio::from_raw_fd(slave.as_raw_fd()) })
                .stderr(unsafe { Stdio::from_raw_fd(slave.as_raw_fd()) })
                .exec();
        }
        _ => println!("fork failed"),
    }

    let mut pty_out = unsafe { File::from_raw_fd(master.as_raw_fd()) };
    spawn(move || {
        let mut buf = [0; 10240];
        loop {
            let msg_size = pty_out.read(buf.as_mut()).expect("read failed");
            if msg_size == 0 {
                continue;
            }
            print!("{}", String::from_utf8_lossy(&buf[..msg_size]).to_string());
        }
    });

    let pty_in = unsafe { File::from_raw_fd(master.as_raw_fd()) };
    let rc_pty_in = Arc::new(Mutex::new(pty_in));
    loop {
        let mut input_cmd = String::new();
        std::io::stdin()
            .read_line(&mut input_cmd)
            .expect("read failed");

        let mut writer = rc_pty_in.lock().expect("lock failed");
        writer
            .write_all(input_cmd.as_bytes())
            .expect("write failed");
    }
}
