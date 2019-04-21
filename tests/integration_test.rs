extern crate assert_cmd;

use assert_cmd::prelude::*;

use std::env::current_dir;
use std::process::Command;

#[test]
fn test_long_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--help");

    cmd.assert().success().code(0);
}

#[test]
fn test_short_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-h");

    cmd.assert().success().code(0);
}

#[test]
fn test_long_version() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--version");

    cmd.assert().success().code(0);
}

#[test]
fn test_short_version() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-V");

    cmd.assert().success().code(0);
}

#[test]
fn test_invalid_arguments() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("random").arg("--arguments");

    cmd.assert().failure().code(1);
}

#[test]
fn test_example_file() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(current_dir().unwrap().join("tests").join("hello_world.bf"));

    cmd.assert().success().code(0).stdout("Hello, World!\n");
}

#[test]
fn test_file_not_found() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("non_existent.bf");

    cmd.assert().failure().code(1);
}

#[test]
fn test_invalid_file_extension() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(
        current_dir()
            .unwrap()
            .join("tests")
            .join("integration_test.rs"),
    );

    cmd.assert().failure().code(1);
}
