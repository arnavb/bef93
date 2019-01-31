/* befunge/interpreter.rs - Contains the implementation of the Befunge-93 interpreter
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

use rand::{thread_rng, Rng};

use std::error::Error as StdError;
use std::io;
use std::io::Write;

// Throughout comments, befunge::Error will be referred to as BefungeError
use super::error::Error as BefungeError;
use super::playfield::{Coord, Direction, Playfield};

// Possible interpreter modes
#[derive(Debug, PartialEq)]
enum Mode {
    String,
    Command,
    Bridge,
}

// This struct handles the execution of the Befunge-93 code. An instance of this
// struct is initialized from the client CLI code.
#[derive(Debug)]
pub struct Interpreter<Writable: Write> {
    playfield: Playfield,
    stack: Vec<i64>,
    output_handle: Writable, // Allow arbitrary output redirection to a struct
    // implementing Write
    mode: Mode,
}

impl<Writable: Write> Interpreter<Writable> {
    // Intializes the interpreter with the program code, an output handle,
    // and optionally an initial program counter position and direction
    pub fn new(
        code: &str,
        output_handle: Writable,
        program_counter_position: Option<Coord>,
        program_counter_direction: Option<Direction>,
    ) -> Result<Interpreter<Writable>, BefungeError> {
        Ok(Interpreter {
            playfield: Playfield::new(
                code,
                program_counter_position.unwrap_or(Coord { x: 0, y: 0 }),
                program_counter_direction.unwrap_or(Direction::Right),
            )?,
            stack: Vec::new(),
            output_handle,
            mode: Mode::Command,
        })
    }

    // Executes the Befunge-93 code. May return the following errors:
    //
    // 1. Any errors propagated from `self.run_unary_operation`, `self.run_binary_operation`,
    //   or `self.run_other_operation`.
    //
    // 2. If an unexpected command is met while parsing in command mode, a BefungeError
    //   will be returned.
    pub fn execute(&mut self) -> Result<(), Box<StdError>> {
        loop {
            // Empty program is an infinite loop
            if self.playfield.dimensions.x == 0 {
                continue;
            }

            let curr_char = self.playfield.get_next_character();

            match self.mode {
                Mode::Bridge => self.mode = Mode::Command,

                Mode::String => match curr_char {
                    '"' => self.mode = Mode::Command,
                    _ => self.stack.push(curr_char as i64),
                },

                Mode::Command => match curr_char {
                    '0'..='9' => self.stack.push(curr_char.to_digit(10).unwrap() as i64),

                    '!' | '_' | '|' | ':' | '$' | '.' | ',' => {
                        self.run_unary_operation(curr_char)?
                    }

                    '+' | '-' | '*' | '/' | '%' | '`' | '\\' | 'g' => {
                        self.run_binary_operation(curr_char)?
                    }

                    ' ' | '>' | '<' | '^' | 'v' | '?' | '"' | '#' | 'p' | '&' | '~' => {
                        self.run_other_operation(curr_char)?
                    }

                    '@' => break,

                    _ => {
                        return Err(
                            BefungeError(format!("{} is not a valid command!", curr_char)).into(),
                        );
                    }
                },
            }

            self.playfield.update_program_counter();
        }
        Ok(())
    }

    // Executes unary operations. May return the following errors:
    //
    // 1. If a conversion from a integer to a character is not possible, a BefungeError
    //   will be returned.
    //
    // 2. TODO: Writing to output handle
    //
    // 3. If the output handle cannot be flushed, the respective io::Error will be
    //   returned.
    fn run_unary_operation(&mut self, operation: char) -> Result<(), Box<StdError>> {
        let value = self.stack.pop().unwrap_or(0);

        match operation {
            '!' => self.stack.push((value == 0) as i64),
            '_' => {
                self.playfield.program_counter_direction = match value {
                    0 => Direction::Right,
                    _ => Direction::Left,
                };
            }
            '|' => {
                self.playfield.program_counter_direction = match value {
                    0 => Direction::Down,
                    _ => Direction::Up,
                };
            }
            ':' => {
                self.stack.push(value);
                self.stack.push(value);
            }
            '$' => (),
            '.' => {
                write!(self.output_handle, "{} ", value)?;
                self.output_handle.flush()?;
            }
            _ => {
                write!(self.output_handle, "{}", convert_int_to_char(value)?)?;
                self.output_handle.flush()?;
            }
        }
        Ok(())
    }

    // Executes binary operations. May return the following errors:
    //
    // 1. If an attempt is made to divide by 0 (usually as a result of an empty stack),
    //   a BefungeError will be returned.
    //
    // 2. If an attempt is made to mod by 0 (usually as a result of an empty stack),
    //   a BefungeError will be returned.
    //
    // 3. Any errors propagated up from `self.playfield.get_character_at`.
    fn run_binary_operation(&mut self, operation: char) -> Result<(), Box<StdError>> {
        let (a, b) = (self.stack.pop().unwrap_or(0), self.stack.pop().unwrap_or(0));

        match operation {
            '+' => self.stack.push(b + a),
            '-' => self.stack.push(b - a),
            '*' => self.stack.push(b * a),
            '/' => match a {
                0 => return Err(BefungeError(format!("Cannot divide {} by 0!", b)).into()),
                _ => self.stack.push(b / a),
            },
            '%' => match a {
                0 => return Err(BefungeError(format!("Cannot mod {} by 0!", b)).into()),
                _ => self.stack.push(b % a),
            },
            '`' => self.stack.push((b > a) as i64),

            '\\' => {
                self.stack.push(a);
                self.stack.push(b);
            }

            _ => self
                .stack
                .push(self.playfield.get_character_at(&Coord { y: a, x: b })? as i64),
        }
        Ok(())
    }

    // Executes other operations (except digits and @). May return the following errors:
    //
    // 1. Any errors propagated up from `self.playfield.set_character_at`.
    //
    // 2. If a conversion from a integer to a character is not possible, a BefungeError
    //   will be returned.
    //
    // 3. For the & command, if a non-integer value is entered, a BefungeError will
    //   be returned.
    //
    // 4. For the ~ command, if a non-char value is entered, a BefungeError will
    //   be returned.
    fn run_other_operation(&mut self, operation: char) -> Result<(), Box<StdError>> {
        match operation {
            ' ' => (),
            '>' => self.playfield.program_counter_direction = Direction::Right,
            '<' => self.playfield.program_counter_direction = Direction::Left,
            '^' => self.playfield.program_counter_direction = Direction::Up,
            'v' => self.playfield.program_counter_direction = Direction::Down,
            '?' => {
                self.playfield.program_counter_direction = match thread_rng().gen_range(0, 4) {
                    0 => Direction::Up,
                    1 => Direction::Down,
                    2 => Direction::Left,
                    _ => Direction::Right,
                }
            }
            '"' => self.mode = Mode::String,
            '#' => self.mode = Mode::Bridge,
            'p' => {
                let position = Coord {
                    y: self.stack.pop().unwrap_or(0),
                    x: self.stack.pop().unwrap_or(0),
                };
                let popped_value = self.stack.pop().unwrap_or(0);

                self.playfield
                    .set_character_at(&position, convert_int_to_char(popped_value)?)?;
            }
            '&' => {
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                self.stack.push(
                    input
                        .trim()
                        .parse::<i64>()
                        .map_err(|_| BefungeError(format!("{} is not a valid integer!", input)))?,
                );
            }
            _ => {
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                self.stack.push(
                    input
                        .trim()
                        .parse::<char>()
                        .map_err(|_| BefungeError(format!("{} is not a valid character!", input)))?
                        as i64,
                );
            }
        }
        Ok(())
    }
}

// TODO: Convert errors to BefungeErrors
fn convert_int_to_char(value: i64) -> Result<char, Box<StdError>> {
    if value < 0 || value > 255 {
        return Err(BefungeError(format!(
            "{} is not a valid ASCII value (between 0 and 255 inclusive)!",
            value
        ))
        .into());
    }

    std::char::from_u32(value as u32)
        .ok_or_else(|| format!("Unable to convert ASCII value {} to a char", value).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod initialization {
        use super::*;

        #[test]
        fn test_basic() {
            let interpreter = Interpreter::new("5:.,@", io::stdout(), None, None).unwrap();

            // Test all fields are properly initialized
            assert!(interpreter.stack.is_empty());
            // TODO: Figure out how to check equality for output handles
            // assert_eq!(interpreter.output_handle, io::stdout());
            assert_eq!(
                interpreter.playfield.code_map,
                vec![['5', ':', '.', ',', '@']]
            );
            assert_eq!(interpreter.mode, Mode::Command);
        }

        #[test]
        fn test_basic_initial_position() {
            let interpreter =
                Interpreter::new("5:.,@", io::stdout(), Some(Coord { x: 1, y: 0 }), None).unwrap();

            assert_eq!(
                interpreter.playfield.program_counter_position,
                Coord { x: 1, y: 0 }
            );
            assert_eq!(interpreter.playfield.get_next_character(), ':');
        }

        #[test]
        fn test_out_of_bounds_initial_position() {
            let interpreter =
                Interpreter::new("5:.,@", io::stdout(), Some(Coord { x: 13333, y: 0 }), None);

            assert!(interpreter.is_err());
        }

        #[test]
        fn test_initial_direction() {
            let interpreter =
                Interpreter::new("5:.,@", io::stdout(), None, Some(Direction::Up)).unwrap();

            assert_eq!(
                interpreter.playfield.program_counter_direction,
                Direction::Up
            );
        }

        #[test]
        fn test_initial_direction_and_position() {
            let mut interpreter = Interpreter::new(
                "5:.,@",
                io::stdout(),
                Some(Coord { x: 1, y: 0 }),
                Some(Direction::Left),
            )
            .unwrap();

            assert_eq!(
                interpreter.playfield.program_counter_position,
                Coord { x: 1, y: 0 }
            );
            assert_eq!(
                interpreter.playfield.program_counter_direction,
                Direction::Left
            );

            interpreter.playfield.update_program_counter();
            assert_eq!(interpreter.playfield.get_next_character(), '5');
        }

        #[test]
        fn test_alternative_output_handle() {
            let mut output: Vec<u8> = Vec::new();
            {
                // Needed for immutable borrow after mutable borrow
                let mut interpreter = Interpreter::new("5:.,@", &mut output, None, None).unwrap();

                interpreter.execute().unwrap();
            }

            assert_eq!(output, vec![53, 32, 5]);
        }
    }

    mod convert_int_to_char {
        use super::*;

        #[test]
        fn test_basic() {
            assert_eq!(convert_int_to_char(57).unwrap(), '9');
            assert_eq!(convert_int_to_char(38).unwrap(), '&');
            assert_eq!(convert_int_to_char(76).unwrap(), 'L');
            assert_eq!(convert_int_to_char(103).unwrap(), 'g');
        }

        #[test]
        fn test_extended_character_set() {
            assert_eq!(convert_int_to_char(233).unwrap(), 'é');
            assert_eq!(convert_int_to_char(247).unwrap(), '÷');
        }

        #[test]
        fn test_upper_bound() {
            assert_eq!(convert_int_to_char(255).unwrap(), 'ÿ');
        }

        #[test]
        fn test_lower_bound() {
            assert_eq!(convert_int_to_char(0).unwrap(), 0_u8 as char);
        }

        #[test]
        fn test_out_of_bounds() {
            assert!(convert_int_to_char(5555).is_err());
            assert!(convert_int_to_char(-333).is_err());
        }
    }
}
