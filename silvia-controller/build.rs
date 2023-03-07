use std::process::Command;
fn main() {
    let status = match Command::new("git").args(&["diff", "--quiet", "HEAD"])
        .status()
        .unwrap()
        .success() {
            true => {""},
            false => {"-?"},
    };

    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();

    println!("cargo:rustc-env=GIT_HASH={}{}", &git_hash[..9], status);
    let version = format!("frnknslv-{}", env!("CARGO_PKG_VERSION"));
    assert!(version.len() <= 16, "{}", version);

    println!("cargo:rustc-env=FRANKENVERSION={}", version);

}
