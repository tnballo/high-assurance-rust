use std::{
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

fn run_cli(test_manifest: impl AsRef<Path>) -> ExitStatus {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--")
        .arg(format!("{}", test_manifest.as_ref().display()))
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{stdout}");

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("{stderr}");

    output.status
}

#[test]
fn test_check_nonce_typing() {
    let test_manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("nonce_typing")
        .join("Cargo.toml");

    assert!(test_manifest.is_file());
    assert!(run_cli(test_manifest).success());
}

#[test]
fn test_check_test_crypto_duplicates() {
    let test_manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tt_test_crates")
        .join("test_crypto_duplicates")
        .join("Cargo.toml");

    assert!(test_manifest.is_file());
    assert!(!run_cli(test_manifest).success());
}

#[test]
fn test_check_test_disallowed_crypto_duplicates() {
    let test_manifest = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tt_test_crates")
        .join("test_disallowed_crypto_publishers")
        .join("Cargo.toml");

    assert!(test_manifest.is_file());
    assert!(!run_cli(test_manifest).success());
}
