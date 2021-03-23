use nix::sys::wait::waitpid;
use nix::unistd::{close, execvp};
use nix::unistd::{fork, ForkResult};
use std::env::args;
use std::ffi::CString;
use std::net::Shutdown;
use std::os::unix::io::IntoRawFd;
use std::os::unix::net::*;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if &args().nth(0).unwrap() == "child" {
        println!("I'm a new child process");
        sleep(Duration::from_secs(30));
        return Ok(());
    }

    let (sock1, sock2) = UnixStream::pair()?;
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            sock2.shutdown(Shutdown::Both)?;
            close(sock2.into_raw_fd())?;
            println!("Added child. Pid: {}", child);
        }
        ForkResult::Child => {
            let cmd = CString::new("/proc/self/exe").unwrap();
            let args = vec![CString::new("child").unwrap()];
            sock1.shutdown(Shutdown::Both)?;
            close(sock1.into_raw_fd())?;
            execvp(&cmd, &args.as_ref())?;
        }
    }

    while let Ok(status) = waitpid(None, None) {
        println!("Reaped child. Status: {:?}", status);
    }

    Ok(())
}
