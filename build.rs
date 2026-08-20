use std::process::Command;

const UNKNOWN: &str = "unknown";

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");

    let sha = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|sha| sha.trim().to_owned())
        .filter(|sha| !sha.is_empty())
        .unwrap_or_else(|| UNKNOWN.to_owned());

    println!("cargo:rustc-env=QUIRPY_GIT_SHA={sha}");
    println!(
        "cargo:rustc-env=QUIRPY_BUILD_DATE={}",
        jiff::Zoned::now().date()
    );
}
