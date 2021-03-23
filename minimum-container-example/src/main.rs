use nix::sched::*;
use nix::sys::wait::waitpid;
use nix::unistd::execvp;
use std::ffi::CString;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cb = Box::new(&|| {
        if let Ok(cmd) = CString::new("bash") {
            let args = vec![
                CString::new("containered bash").unwrap(),
                CString::new("-l").unwrap(),
            ];
            if execvp(&cmd, &args.as_ref()).is_err() {
                return 127;
            }
        }
        127
    });

    let mut child_stack = [0u8; 8192];
    let flags = CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWUTS
        | CloneFlags::CLONE_NEWIPC
        | CloneFlags::CLONE_NEWPID;
    let sigchld = 17; // x86/arm. ref man 7 signal
    let _pid = clone(cb, &mut child_stack, flags, Some(sigchld))?;

    while let Ok(status) = waitpid(None, None) {
        println!("Exit Status: {:?}", status);
    }

    Ok(())
}
