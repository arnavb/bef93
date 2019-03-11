extern crate assert_cmd;

use assert_cmd::prelude::*;
use std::process::Command;

const EXPECTED_HELP_MESSAGE: &str = "bef93 0.1.0
Arnav Borborah <arnavborborah11@gmail.com>
A Befunge-93 interpreter supporting an extended grid

USAGE:
    bef93 <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <FILE>    A file with Befunge-93 source code
";

const EXPECTED_VERSION_MESSAGE: &str = "bef93 0.1.0\n";

#[test]
fn test_long_help_message() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--help");

    cmd.assert().success().code(0).stdout(EXPECTED_HELP_MESSAGE);
}

#[test]
fn test_short_help_message() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-h");

    cmd.assert().success().code(0).stdout(EXPECTED_HELP_MESSAGE);
}

#[test]
fn test_long_version_message() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .code(0)
        .stdout(EXPECTED_VERSION_MESSAGE);
}

#[test]
fn test_short_version_message() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("-V");

    cmd.assert()
        .success()
        .code(0)
        .stdout(EXPECTED_VERSION_MESSAGE);
}

#[test]
fn test_invalid_arguments() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("random").arg("--arguments");

    cmd.assert().failure().code(1);
}
