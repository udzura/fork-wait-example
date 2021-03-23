use nix::fcntl::*;
use nix::sys::wait::waitpid;
use nix::unistd::{close, execvp};
use nix::unistd::{fork, ForkResult};
use std::env::args;
use std::ffi::CString;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::os::unix::io::{FromRawFd, IntoRawFd, RawFd};
use std::os::unix::net::*;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if &args().nth(0).unwrap() == "child" {
        println!("I'm a new child process");
        let sock2: RawFd = args().nth(1).unwrap().parse()?;
        let mut sock2 = unsafe { UnixStream::from_raw_fd(sock2) };

        loop {
            let mut buf = [0u8; 1024];
            if sock2.read(&mut buf)? == 0 {
                break;
            }
            print!("{}", String::from_utf8((&buf).to_vec())?.trim_matches('\0'));
        }
        println!("All drained.");
        sleep(Duration::from_secs(3));

        return Ok(());
    }

    let (sock1, sock2) = UnixStream::pair()?;
    let sock = match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            println!("Added child. Pid: {}", child);
            close(sock2.into_raw_fd())?;
            sock1.into_raw_fd()
        }
        ForkResult::Child => {
            let cmd = CString::new("/proc/self/exe").unwrap();
            close(sock1.into_raw_fd())?;

            let newsock = sock2.into_raw_fd();
            let _r = fcntl(newsock, FcntlArg::F_SETFD(FdFlag::empty()))?;

            let args = vec![
                CString::new("child").unwrap(),
                CString::new(format!("{}", newsock)).unwrap(),
            ];
            execvp(&cmd, &args.as_ref())?;
            0
        }
    };

    let mut sock1 = unsafe { UnixStream::from_raw_fd(sock) };
    for i in 0..5 {
        for _ in 0..=i {
            sock1.write(b"byte ")?;
        }
        sock1.write(b"\n")?;
        sleep(Duration::from_secs(1));
    }
    sock1.shutdown(Shutdown::Both)?;

    while let Ok(status) = waitpid(None, None) {
        println!("Reaped child. Status: {:?}", status);
    }

    Ok(())
}
