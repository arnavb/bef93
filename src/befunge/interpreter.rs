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
use std::io::{BufRead, Write};

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
pub struct Interpreter<Writable, Readable>
where
    Writable: Write,
    Readable: BufRead,
{
    playfield: Playfield,
    stack: Vec<i64>,
    output_handle: Writable,
    input_handle: Readable,
    mode: Mode,
}

impl<Writable, Readable> Interpreter<Writable, Readable>
where
    Writable: Write,
    Readable: BufRead,
{
    // Intializes the interpreter with the program code, an output handle,
    // and optionally an initial program counter position and direction
    pub fn new(
        code: &str,
        output_handle: Writable,
        input_handle: Readable,
        program_counter_position: Option<Coord>,
        program_counter_direction: Option<Direction>,
    ) -> Result<Interpreter<Writable, Readable>, BefungeError> {
        Ok(Interpreter {
            playfield: Playfield::new(
                code,
                program_counter_position.unwrap_or(Coord { x: 0, y: 0 }),
                program_counter_direction.unwrap_or(Direction::Right),
            )?,
            stack: Vec::new(),
            output_handle,
            input_handle,
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
                self.input_handle.read_line(&mut input)?;

                self.stack.push(
                    input
                        .trim()
                        .parse::<i64>()
                        .map_err(|_| BefungeError(format!("{} is not a valid integer!", input)))?,
                );
            }
            _ => {
                let mut input = String::new();
                self.input_handle.read_line(&mut input)?;

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
    use std::io;

    // fn setup_interpreter(code: &str) -> Interpreter<io::Stdout, io::StdinLock> {
    //     let input_handle = io::stdin();
    //     let interpreter = Interpreter::new(code, io::stdout(), input_handle.lock(), None, None);
    //     interpreter.unwrap()
    // }

    fn setup_interpreter<'a>(
        code: &str,
        input_data: Option<&'a [u8]>,
    ) -> Interpreter<Vec<u8>, &'a [u8]> {
        let output_handle: Vec<u8> = Vec::new();
        let input_handle = input_data.unwrap_or("".as_bytes());

        let mut interpreter =
            Interpreter::new(code, output_handle, input_handle, None, None).unwrap();
        interpreter.execute().unwrap();
        interpreter
    }

    mod initialization {
        use super::*;

        #[test]
        fn test_basic() {
            let input_handle = io::stdin();
            let interpreter =
                Interpreter::new("5:.,@", io::stdout(), input_handle.lock(), None, None).unwrap();

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
            let input_handle = io::stdin();
            let interpreter = Interpreter::new(
                "5:.,@",
                io::stdout(),
                input_handle.lock(),
                Some(Coord { x: 1, y: 0 }),
                None,
            )
            .unwrap();

            assert_eq!(
                interpreter.playfield.program_counter_position,
                Coord { x: 1, y: 0 }
            );
            assert_eq!(interpreter.playfield.get_next_character(), ':');
        }

        #[test]
        fn test_out_of_bounds_initial_position() {
            let input_handle = io::stdin();
            let interpreter = Interpreter::new(
                "5:.,@",
                io::stdout(),
                input_handle.lock(),
                Some(Coord { x: 13333, y: 0 }),
                None,
            );

            assert!(interpreter.is_err());
        }

        #[test]
        fn test_initial_direction() {
            let input_handle = io::stdin();
            let interpreter = Interpreter::new(
                "5:.,@",
                io::stdout(),
                input_handle.lock(),
                None,
                Some(Direction::Up),
            )
            .unwrap();

            assert_eq!(
                interpreter.playfield.program_counter_direction,
                Direction::Up
            );
        }

        #[test]
        fn test_initial_direction_and_position() {
            let input_handle = io::stdin();
            let mut interpreter = Interpreter::new(
                "5:.,@",
                io::stdout(),
                input_handle.lock(),
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
            let input_handle = io::stdin();
            let mut output_handle: Vec<u8> = Vec::new();
            let interpreter =
                Interpreter::new("5:.,@", &mut output_handle, input_handle.lock(), None, None);
            assert!(interpreter.is_ok());
        }

        #[test]
        fn test_alternative_input_handle() {
            let mut input = "input".as_bytes();
            let interpreter = Interpreter::new("5:.,@", io::stdout(), &mut input, None, None);
            assert!(interpreter.is_ok());
        }
    }

    mod befunge_code {
        use super::*;

        mod example_programs {
            use super::*;
            // Alternative to integration tests (temporarily)
            // These programs are taken from https://esolangs.org/wiki/Befunge#Befunge-93_and_Befunge-98

            #[test]
            fn test_hello_world() {
                let interpreter = setup_interpreter("64+\"!dlroW ,olleH\">:#,_@", None);
                assert_eq!(interpreter.output_handle, "Hello, World!\n".as_bytes());
            }

            #[test]
            fn test_factorial() {
                let interpreter =
                    setup_interpreter("&>:1-:v v *_$.@\n ^    _$>\\:^", Some("5".as_bytes()));
                assert_eq!(interpreter.output_handle, "120 ".as_bytes());
            }

            #[test]
            fn test_sieve_of_eratosthenes() {
                let interpreter = setup_interpreter("2>:3g\" \"-!v\\  g30          <\n |!`\"O\":+1_:.:03p>03g+:\"O\"`|\n @               ^  p3\\\" \":<\n2 234567890123456789012345678901234567890123456789012345678901234567890123456789", None);

                assert_eq!(
                    interpreter.output_handle,
                    "2 3 5 7 11 13 17 19 23 29 31 37 41 43 47 53 59 61 67 71 73 79 ".as_bytes()
                );
            }

            #[test]
            fn test_quine_one() {
                let interpreter =
                    setup_interpreter("01->1# +# :# 0# g# ,# :# 5# 8# *# 4# +# -# _@", None);
                assert_eq!(
                    interpreter.output_handle,
                    "01->1# +# :# 0# g# ,# :# 5# 8# *# 4# +# -# _@".as_bytes()
                );
            }

            #[test]
            fn test_quine_two() {
                let interpreter =
                    setup_interpreter("0 v\n \"<@_ #! #: #,<*2-1*92,*84,*25,+*92*4*55.0", None);
                assert_eq!(
                    interpreter.output_handle,
                    "0 v\n \"<@_ #! #: #,<*2-1*92,*84,*25,+*92*4*55.0 ".as_bytes()
                );
            }

            #[should_panic]
            #[test]
            fn test_quine_three() {
                let interpreter = setup_interpreter(":0g,:\"~\"`#@_1+0\"Quines are Fun\">_", None);
                assert_eq!(
                    interpreter.output_handle,
                    ":0g,:\"~\"`#@_1+0\"Quines are Fun\">_".as_bytes()
                );
            }
        }

        mod individual_commands {
            use super::*;

            mod unary_operators {
                use super::*;

                mod not_operator {
                    use super::*;

                    #[test]
                    fn test_true_value() {
                        let mut interpreter = setup_interpreter("5@", None);
                        let result = interpreter.run_unary_operation('!');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &0);
                    }

                    #[test]
                    fn test_false_value() {
                        let mut interpreter = setup_interpreter("0@", None);
                        let result = interpreter.run_unary_operation('!');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &1);
                    }
                }

                mod horizontal_if {
                    use super::*;

                    #[test]
                    fn test_evaluates_to_true() {
                        let mut interpreter = setup_interpreter("0@", None);
                        let result = interpreter.run_unary_operation('_');
                        assert!(result.is_ok());
                        assert_eq!(
                            interpreter.playfield.program_counter_direction,
                            Direction::Right
                        );
                    }

                    #[test]
                    fn test_evaluates_to_false() {
                        let mut interpreter = setup_interpreter("9@", None);
                        let result = interpreter.run_unary_operation('_');
                        assert!(result.is_ok());
                        assert_eq!(
                            interpreter.playfield.program_counter_direction,
                            Direction::Left
                        );
                    }
                }

                mod vertical_if {
                    use super::*;

                    #[test]
                    fn test_evaluates_to_true() {
                        let mut interpreter = setup_interpreter("0@", None);
                        let result = interpreter.run_unary_operation('|');
                        assert!(result.is_ok());
                        assert_eq!(
                            interpreter.playfield.program_counter_direction,
                            Direction::Down
                        );
                    }

                    #[test]
                    fn test_evaluates_to_false() {
                        let mut interpreter = setup_interpreter("9@", None);
                        let result = interpreter.run_unary_operation('|');
                        assert!(result.is_ok());
                        assert_eq!(
                            interpreter.playfield.program_counter_direction,
                            Direction::Up
                        );
                    }
                }

                #[test]
                fn test_duplicate() {
                    let mut interpreter = setup_interpreter("5@", None);
                    let result = interpreter.run_unary_operation(':');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.stack, vec![5, 5]);
                }

                #[test]
                fn test_pop() {
                    let mut interpreter = setup_interpreter("5@", None);
                    let result = interpreter.run_unary_operation('$');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.stack, vec![]);
                }

                #[test]
                fn test_write_integer() {
                    let mut interpreter = setup_interpreter("5@", None);
                    let result = interpreter.run_unary_operation('.');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.output_handle, "5 ".as_bytes());
                }

                mod write_character {
                    use super::*;

                    #[test]
                    fn test_basic() {
                        let mut interpreter = setup_interpreter("\"a\"@", None);
                        let result = interpreter.run_unary_operation(',');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.output_handle, "a".as_bytes());
                    }

                    #[test]
                    fn test_non_printable_character() {
                        let mut interpreter = setup_interpreter("55+@", None);
                        let result = interpreter.run_unary_operation(',');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.output_handle, "\n".as_bytes());
                    }

                    #[test]
                    fn test_special_characters() {
                        let mut interpreter = setup_interpreter("\"á\"@", None);
                        let result = interpreter.run_unary_operation(',');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.output_handle, "á".as_bytes());
                    }
                }
            }

            mod binary_operators {
                use super::*;

                mod addition {
                    use super::*;

                    #[test]
                    fn test_basic() {
                        let mut interpreter = setup_interpreter("55@", None);
                        let result = interpreter.run_binary_operation('+');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &10);
                    }

                    #[test]
                    fn test_only_one_operand() {
                        let mut interpreter = setup_interpreter("5@", None);
                        let result = interpreter.run_binary_operation('+');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &5);
                    }

                    #[test]
                    fn test_only_no_operands() {
                        let mut interpreter = setup_interpreter("@", None);
                        let result = interpreter.run_binary_operation('+');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &0);
                    }
                }

                mod subtraction {
                    use super::*;

                    #[test]
                    fn test_basic() {
                        let mut interpreter = setup_interpreter("55@", None);
                        let result = interpreter.run_binary_operation('-');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &0);
                    }

                    #[test]
                    fn test_negative_result() {
                        let mut interpreter = setup_interpreter("57@", None);
                        let result = interpreter.run_binary_operation('-');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &-2);
                    }
                }

                #[test]
                fn test_multiplication() {
                    let mut interpreter = setup_interpreter("56@", None);
                    let result = interpreter.run_binary_operation('*');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.stack.last().unwrap(), &30);
                }

                mod division {
                    use super::*;

                    #[test]
                    fn test_basic() {
                        let mut interpreter = setup_interpreter("62@", None);
                        let result = interpreter.run_binary_operation('/');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &3);
                    }

                    #[test]
                    fn test_integer_division() {
                        let mut interpreter = setup_interpreter("72@", None);
                        let result = interpreter.run_binary_operation('/');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &3);
                    }

                    #[test]
                    fn test_division_by_zero() {
                        let mut interpreter = setup_interpreter("60@", None);
                        let result = interpreter.run_binary_operation('/');
                        assert!(result.is_err());
                    }
                }

                mod modulo {
                    use super::*;

                    #[test]
                    fn test_basic() {
                        let mut interpreter = setup_interpreter("64@", None);
                        let result = interpreter.run_binary_operation('%');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &2);
                    }

                    #[test]
                    fn test_modulo_by_zero() {
                        let mut interpreter = setup_interpreter("60@", None);
                        let result = interpreter.run_binary_operation('%');
                        assert!(result.is_err());
                    }
                }

                mod greater_than {
                    use super::*;

                    #[test]
                    fn test_basic() {
                        let mut interpreter = setup_interpreter("65@", None);
                        let result = interpreter.run_binary_operation('`');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &1);
                    }

                    #[test]
                    fn test_false_value() {
                        let mut interpreter = setup_interpreter("56@", None);
                        let result = interpreter.run_binary_operation('`');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &0);
                    }

                    #[test]
                    fn test_equal_values() {
                        let mut interpreter = setup_interpreter("66@", None);
                        let result = interpreter.run_binary_operation('`');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &0);
                    }
                }

                #[test]
                fn test_swap() {
                    let mut interpreter = setup_interpreter("65@", None);
                    let result = interpreter.run_binary_operation('\\');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.stack, vec![5, 6]);
                }

                mod get {
                    use super::*;

                    #[test]
                    fn test_basic() {
                        let mut interpreter = setup_interpreter("49v\n  >10@", None);
                        let result = interpreter.run_binary_operation('g');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.stack.last().unwrap(), &57);
                    }

                    #[test]
                    fn test_out_of_bounds() {
                        let mut interpreter = setup_interpreter("49v\n  >15@", None);
                        let result = interpreter.run_binary_operation('g');
                        assert!(result.is_err());
                    }
                }
            }

            mod other_operators {
                use super::*;

                #[test]
                fn test_noop() {
                    let mut interpreter = setup_interpreter("5@", None);
                    let result = interpreter.run_other_operation(' ');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.stack.last().unwrap(), &5);
                }

                #[test]
                fn test_direction_right() {
                    let mut interpreter = setup_interpreter("@", None);
                    let result = interpreter.run_other_operation('>');
                    assert!(result.is_ok());
                    assert_eq!(
                        interpreter.playfield.program_counter_direction,
                        Direction::Right
                    );
                }

                #[test]
                fn test_direction_left() {
                    let mut interpreter = setup_interpreter("@", None);
                    let result = interpreter.run_other_operation('<');
                    assert!(result.is_ok());
                    assert_eq!(
                        interpreter.playfield.program_counter_direction,
                        Direction::Left
                    );
                }

                #[test]
                fn test_direction_down() {
                    let mut interpreter = setup_interpreter("@", None);
                    let result = interpreter.run_other_operation('v');
                    assert!(result.is_ok());
                    assert_eq!(
                        interpreter.playfield.program_counter_direction,
                        Direction::Down
                    );
                }

                #[test]
                fn test_direction_up() {
                    let mut interpreter = setup_interpreter("@", None);
                    let result = interpreter.run_other_operation('^');
                    assert!(result.is_ok());
                    assert_eq!(
                        interpreter.playfield.program_counter_direction,
                        Direction::Up
                    );
                }

                #[test]
                fn test_random_direction() {
                    let mut interpreter = setup_interpreter("@", None);
                    let result = interpreter.run_other_operation('?');
                    assert!(result.is_ok());
                }

                #[test]
                fn test_string_mode_change() {
                    let mut interpreter = setup_interpreter("5@", None);
                    let result = interpreter.run_other_operation('"');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.mode, Mode::String);
                }

                #[test]
                fn test_bridge_mode_change() {
                    let mut interpreter = setup_interpreter("5@", None);
                    let result = interpreter.run_other_operation('#');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.mode, Mode::Bridge);
                }

                mod put {
                    use super::*;

                    #[test]
                    fn test_basic() {
                        let mut interpreter = setup_interpreter("49v\n  >510@", None);
                        let result = interpreter.run_other_operation('p');
                        assert!(result.is_ok());
                        assert_eq!(interpreter.playfield.code_map[0][1], '\u{5}');
                    }

                    #[test]
                    fn test_out_of_bounds() {
                        let mut interpreter = setup_interpreter("49v\n  >515@", None);
                        let result = interpreter.run_other_operation('p');
                        assert!(result.is_err());
                    }
                }

                #[test]
                fn test_read_integer() {
                    let mut interpreter = setup_interpreter("@", Some("5".as_bytes()));
                    let result = interpreter.run_other_operation('&');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.stack.last().unwrap(), &5);
                }

                #[test]
                fn test_read_character() {
                    let mut interpreter = setup_interpreter("@", Some("5".as_bytes()));
                    let result = interpreter.run_other_operation('~');
                    assert!(result.is_ok());
                    assert_eq!(interpreter.stack.last().unwrap(), &53);
                }
            }
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
