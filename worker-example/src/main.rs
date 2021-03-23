use nix::sys::wait::waitpid;
use nix::unistd::{fork, ForkResult};
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    for _ in 0..5 {
        match unsafe { fork()? } {
            ForkResult::Parent { child } => {
                println!("Added child. Pid: {}", child);
            }
            ForkResult::Child => {
                println!("I'm a new child process");
                sleep(Duration::from_secs(30));
                println!("Exit!!1");
                std::process::exit(0)
            }
        }
        sleep(Duration::from_secs(3))
    }

    while let Ok(status) = waitpid(None, None) {
        println!("Reaped child. Status: {:?}", status);
    }

    Ok(())
}
