use nix::libc::waitpid;
use nix::pty::forkpty;
use nix::unistd::ForkResult;
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::process::{exit, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

static DEFAUL_COMMAND_SHELL: &str = "zsh";

fn main() {
    let pty = unsafe { forkpty(None, None).expect("fork failed") };

    let (master, slave) = (
        pty.master.try_clone().expect("failed to clone master"),
        pty.master.try_clone().expect("failed to clone slave"),
    );

    match pty.fork_result {
        ForkResult::Parent { child: pid, .. } => {
            println!("Welcom to Rust TTY!");
            spawn(move || {
                unsafe { waitpid(pid.as_raw(), &mut 0, 0) };
                println!("process exit!");
                exit(0);
            });
        }
        ForkResult::Child => {
            Command::new(DEFAUL_COMMAND_SHELL)
                .stdin(unsafe { Stdio::from_raw_fd(slave.as_raw_fd()) })
                .stdout(unsafe { Stdio::from_raw_fd(slave.as_raw_fd()) })
                .stderr(unsafe { Stdio::from_raw_fd(slave.as_raw_fd()) })
                .exec();
        }
    }

    let mut pty_out = unsafe { File::from_raw_fd(master.as_raw_fd()) };
    spawn(move || {
        let mut buffer = [0; 10240];
        loop {
            let msg_length = pty_out.read(buffer.as_mut()).expect("read failed");
            if msg_length == 0 {
                continue;
            }
            print!(
                "{}",
                String::from_utf8_lossy(&buffer[..msg_length]).to_string()
            );
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
