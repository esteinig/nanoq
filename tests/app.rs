use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn input_file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.args(vec!["-i", "file/doesnt/exist.fq", "-s"]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));

    Ok(())
}

#[test]
fn output_file_in_nonexistant_dir() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.args(vec![
        "-i",
        "tests/cases/test_ok.fq",
        "-o",
        "dir/doesnt/exists/out.fq",
    ]);
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file"));

    Ok(())
}

#[test]
fn valid_inputs_raise_no_errors() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.args(vec![
        "-i",
        "tests/cases/test_ok.fq",
        "-O",
        "g",
        "-c",
        "9",
        "-l",
        "5000",
    ]);

    cmd.assert().success();

    Ok(())
}

#[test]
fn valid_input_output_stdout_ok() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    cmd.args(vec!["-i", "tests/cases/test_ok.fq"]);

    cmd.assert().success();

    Ok(())
}

#[test]
fn valid_length_file_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    let dir = tempdir()?;
    let test_file = dir.path().join("test.txt");
    let test_file_path_str = test_file.to_str().unwrap();

    cmd.args(vec![
        "-i",
        "tests/cases/test_ok.fq",
        "-L",
        test_file_path_str,
    ]);
    cmd.assert().success();

    let contents = fs::read_to_string(test_file)?;

    dir.close()?;

    assert_eq!(contents.trim(), "4");

    Ok(())
}
#[test]
fn valid_quality_file_output() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;

    let dir = tempdir()?;
    let test_file = dir.path().join("test.txt");
    let test_file_path_str = test_file.to_str().unwrap();

    cmd.args(vec![
        "-i",
        "tests/cases/test_ok.fq",
        "-Q",
        test_file_path_str,
    ]);
    cmd.assert().success();

    let contents = fs::read_to_string(test_file)?;

    dir.close()?;

    assert_eq!(contents.trim(), "40.0");

    Ok(())
}
