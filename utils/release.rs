use std::fs;
use std::process::Command;

#[derive(Debug)]
enum Error {}

#[cfg(target_os = "macos")]
fn main() -> Result<(), Error> {
    println!("Copy exe to the dir");
    fs::copy(
        "target/release/kube-switchboard",
        "assets/macos/Kube Switchboard.app/Contents/MacOS/kube-switchboard",
    )
    .expect("Failed to copy the release exe");
    println!("packing kube-switchboard.dmg");
    Command::new("hdiutil")
        .arg("create")
        .arg("assets/kube-switchboard.dmg")
        .arg("-volname")
        .arg("Kube Switchboard")
        .arg("-srcfolder")
        .arg("assets/macos")
        .arg("-ov")
        .spawn()
        .expect("Failed packing dmg");
    Ok(())
}
