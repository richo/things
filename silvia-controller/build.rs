use std::process::Command;
fn main() {
    let status = match Command::new("git").args(&["diff", "--quiet", "HEAD"])
        .status()
        .unwrap()
        .success() {
            true => {""},
            false => {"-?"},
    };

    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output().unwrap();
    let git_hash = String::from_utf8(output.stdout).unwrap();
    let full_hash = format!("{}{}", git_hash.strip_suffix("\n").unwrap(), status);

    assert!(full_hash.len() <= 16, "{}", full_hash);
    println!("cargo:rustc-env=GIT_HASH={}", full_hash);

    let version = format!("frnknslv-{}", env!("CARGO_PKG_VERSION"));
    assert!(version.len() <= 16, "{}", version);
    println!("cargo:rustc-env=FRANKENVERSION={}", version);

}
