// use memfd_exec::{MemFdExecutable, Stdio};
// use qemu::QEMU_X86_64_LINUX_USER;

// use std::env::args;

// pub async fn startup_machine() {
//     let qemu = QEMU_X86_64_LINUX_USER;
//     let mut args: Vec<String> = args().collect();
//     args.remove(0);
//     let qemu_exec = MemFdExecutable::new("qemu-x86_64", qemu)
//         .arg("-version")
//         .stdin(Stdio::inherit())
//         .stdout(Stdio::inherit())
//         .stderr(Stdio::inherit())
//         .spawn()
//         .expect("Failed to start qemu process")
//         .wait()
//         .expect("Qemu process failed");
// }
