use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;

#[test]
fn runs_with_help() {
    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg("--help").assert().success();
}

#[test]
fn scans_current_directory() {
    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.assert().success();
}

#[test]
fn scans_specific_directory() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create a mock git repository
    let repo_path = base_path.join("test_repo");
    fs::create_dir_all(&repo_path).unwrap();
    fs::create_dir_all(repo_path.join(".git")).unwrap();

    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg("--path")
        .arg(base_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicates::str::contains("test_repo"));
}

#[test]
fn verbose_flag_works() {
    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg("--verbose").assert().success();
}

#[test]
fn changes_only_flag_works() {
    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg("--changes-only").assert().success();
}
