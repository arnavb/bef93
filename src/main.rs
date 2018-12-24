/* main.rs - Contains the main function and CLI code for bef93
 * Copyright 2018 Arnav Borborah
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[macro_use]
extern crate clap;
extern crate rand;

mod befunge;

use std::env::current_dir;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;
use std::{error, io, process};

fn main() {
    process::exit(if let Err(err) = cli() {
        // Error handling code
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            eprint!("{}", clap_err);
            io::stdout()
                .flush()
                .unwrap_or_else(|_| eprintln!("Unable to flush stdout!"));
        } else if let Some(befunge_err) = err.downcast_ref::<befunge::Error>() {
            eprintln!("Befunge-93 Error: {}", befunge_err);
        } else if let Some(io_err) = err.downcast_ref::<io::Error>() {
            eprintln!("IO Error: {}", io_err);
        }
        1
    } else {
        0
    });
}

fn cli() -> Result<(), Box<error::Error>> {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("A Befunge-93 interpreter supporting an extended grid")
        .arg(
            clap::Arg::with_name("FILE")
                .help("A file with Befunge-93 source code")
                .required(true),
        ).get_matches_safe()?;

    let resolved_filepath = resolve_filepath(matches.value_of("FILE").unwrap())?;

    // Check if the file has a '.bf' or '.b93' extension
    match resolved_filepath.extension() {
        Some(extension) => {
            if !(extension == "bf" || extension == "b93") {
                return Err("The file extension of the passed file was not '.bf' or '.b93'!".into());
            }
        }
        None => return Err("The file extension of the passed file was not found!".into()),
    }

    let file_contents = read_to_string(resolved_filepath)?;

    // TODO: Add support for redirected output to a file
    let mut output_handle = io::stdout();

    // TODO: Add support for user supplied initial direction and position
    let mut interpreter =
        befunge::Interpreter::new(&file_contents, &mut output_handle, None, None)?;

    interpreter.execute()?;

    Ok(())
}

// Resolves a passed filepath to either a relative or absolute location.
// If the file does not exist or refer to a file, a io::Error error will be returned.
fn resolve_filepath(path: &str) -> Result<PathBuf, Box<error::Error>> {
    let mut result = PathBuf::from(path);

    if !result.exists() || !result.is_file() {
        result = current_dir()?;
        result.push(path);

        if !result.exists() || !result.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "The passed path does not exist or does not refer to a file!",
            ).into());
        }
    }

    Ok(result)
}
