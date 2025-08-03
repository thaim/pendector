use assert_cmd::Command;
use predicates::prelude::*;
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
    cmd.arg(base_path.to_str().unwrap())
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

#[test]
fn max_depth_flag_works() {
    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg("--max-depth").arg("3").assert().success();
}

#[test]
fn multiple_paths_work() {
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    // Create mock git repositories in both directories
    let repo1_path = temp_dir1.path().join("repo1");
    fs::create_dir_all(&repo1_path).unwrap();
    fs::create_dir_all(repo1_path.join(".git")).unwrap();

    let repo2_path = temp_dir2.path().join("repo2");
    fs::create_dir_all(&repo2_path).unwrap();
    fs::create_dir_all(repo2_path.join(".git")).unwrap();

    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg(temp_dir1.path().to_str().unwrap())
        .arg(temp_dir2.path().to_str().unwrap())
        .assert()
        .success()
        .stdout(predicates::str::contains("repo1"))
        .stdout(predicates::str::contains("repo2"));
}

#[test]
fn json_format_works() {
    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["))
        .stdout(predicate::str::ends_with("]\n"));
}

#[test]
fn json_format_contains_valid_structure() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create a mock git repository
    let repo_path = base_path.join("test_repo");
    fs::create_dir_all(&repo_path).unwrap();
    fs::create_dir_all(repo_path.join(".git")).unwrap();

    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg(base_path.to_str().unwrap())
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\""))
        .stdout(predicate::str::contains("\"path\""))
        .stdout(predicate::str::contains("\"has_changes\""))
        .stdout(predicate::str::contains("test_repo"));
}

#[test]
fn fetch_flag_works() {
    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg("--fetch").assert().success();
}

#[test]
fn fetch_flag_with_specific_directory() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create a mock git repository
    let repo_path = base_path.join("test_repo");
    fs::create_dir_all(&repo_path).unwrap();
    fs::create_dir_all(repo_path.join(".git")).unwrap();

    let mut cmd = Command::cargo_bin("pendector").unwrap();
    cmd.arg(base_path.to_str().unwrap())
        .arg("--fetch")
        .assert()
        .success()
        .stdout(predicate::str::contains("test_repo"));
}
