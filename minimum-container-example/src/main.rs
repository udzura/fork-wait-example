use nix::mount::*;
use nix::sched::*;
use nix::sys::wait::waitpid;
use nix::unistd::{chroot, execvp};
use std::env::{args, set_current_dir};
use std::ffi::CString;
type MyResult = Result<(), Box<dyn std::error::Error>>;

fn mount_make_private() -> Result<(), nix::Error> {
    mount(
        Some("none"),
        "/",
        None::<&str>,
        MsFlags::MS_REC | MsFlags::MS_PRIVATE,
        None::<&str>,
    )
}

fn mount_proc(source: &str, target: &str) -> Result<(), nix::Error> {
    mount(
        Some(source),
        target,
        Some("proc"),
        MsFlags::empty(),
        None::<&str>,
    )
}

fn container_prelude(root: &str) -> MyResult {
    mount_make_private()?;
    chroot(root)?;
    set_current_dir("/")?;
    mount_proc("proc", "/proc")?;
    Ok(())
}

fn main() -> MyResult {
    let usage = format!("Usage: {} [newroot]", args().nth(0).unwrap());
    let root = args().nth(1).ok_or(usage)?;

    let cb = Box::new(|| {
        if let Err(e) = container_prelude(&root) {
            eprintln!("mount failed: {:?}", e);
            return 127;
        }

        let cmd = CString::new("bash").unwrap();
        let args = vec![
            CString::new("containered bash").unwrap(),
            CString::new("-l").unwrap(),
        ];
        if let Err(e) = execvp(&cmd, &args.as_ref()) {
            eprintln!("execvp failed: {:?}", e);
            return 127;
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
