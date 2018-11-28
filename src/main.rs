#[macro_use] extern crate clap;

mod interpreter;

use std::{error, io, process};
use std::env::current_dir;
use std::fs::read_to_string;
use std::path::PathBuf;

fn main() {
    process::exit(if let Err(err) = cli() {
        // The string is trimmed because clap errors have an extra newline
        // at the end.
        eprintln!("{}", format!("{}", err).trim_right());
        1
    } else {
        0
    });
}

fn cli() -> Result<(), Box<error::Error>> {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("A Befunge-93 interpreter")
        .arg(
            clap::Arg::with_name("FILE")
                .help("A file with Befunge-93 source code")
                .required(true)
        )
        .get_matches_safe()?;
    
    let file_contents = get_file_contents(matches.value_of("FILE").unwrap())?;
    
    Ok(())
}

fn get_file_contents(passed_argument: &str) -> Result<String, Box<error::Error>> {
    let mut resolved_path = PathBuf::from(passed_argument);
    
    if !resolved_path.exists() || !resolved_path.is_file() {
        resolved_path = current_dir()?;
        resolved_path.push(passed_argument);
        
        if !resolved_path.exists() || !resolved_path.is_file() {
            return Err(io::Error::new(io::ErrorKind::NotFound,
                "The passed file is either not a file or does not exist!").into());
        }
    }

    Ok(read_to_string(resolved_path)?)
}
