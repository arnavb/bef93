/* bef93/error.rs - Contains the implementation of the Befunge-93 interpreter
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

use std::io;
use std::error::Error as StdError;
use std::collections::HashMap;
use std::io::Write;

use super::error::Error as BefungeError;

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
enum Mode {
    String,
    Command,
    Bridge,
}

#[derive(Debug)]
pub struct Coord {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug)]
struct Playfield {
    code_map: Vec<Vec<char>>,
    dimensions: Coord,
    
    program_counter_position: Coord,
    program_counter_direction: Direction,
    
    curr_char: char,
}

impl Playfield {
    fn new(code: &str, program_counter_position: Coord, program_counter_direction: Direction) -> Playfield {
        // Get the longest line width as the width of the playfield
        let width = code.lines().max_by_key(|line| line.len()).unwrap_or("").len();
    
        // Create a vector of vector of chars. Each line is right-padded with spaces
        // to the longest line width.
        let code_map = code.lines()
            .map(|line| format!("{:<width$}", line, width = width)
                    .chars().collect::<Vec<_>>())
            .collect::<Vec<Vec<_>>>();
        
        // Get the height of the playfield
        let height = code_map.len();
    
        Playfield {
            code_map,
            dimensions: Coord {
                x: width as i64,
                y: height as i64,
            },
            program_counter_position,
            program_counter_direction,
            curr_char: '@',
        }
    }
    
    fn get_next_character(&self) -> char {
        // program_counter.
        self.code_map[self.program_counter_position.x as usize][self.program_counter_position.y as usize]
    }
    
    fn set_character_at(&mut self, position: Coord, value: char) -> Result<(), BefungeError> {
        if position.x < 0 || position.y < 0
            || position.x > self.dimensions.x || position.y > self.dimensions.y {
            Err(BefungeError(format!("Location ({}, {}) is out of bounds!", position.x, position.y)))
        } else {
            self.code_map[position.x as usize][position.y as usize] = value;
            Ok(())
        }
    }
    
    fn get_character_at(&self, position: Coord) -> Result<char, BefungeError> {
        if position.x < 0 || position.y < 0
            || position.x > self.dimensions.x || position.y > self.dimensions.y {
            Err(BefungeError(format!("Location ({}, {}) is out of bounds!", position.x, position.y)))
        } else {
            Ok(self.code_map[position.x as usize][position.y as usize])
        }
    }
    
    fn update_program_counter(&mut self) {
        self.program_counter_position = match self.program_counter_direction {
            Direction::Up => Coord {
                x: self.program_counter_position.x,
                y: match self.program_counter_position.y {
                    0 => self.dimensions.y - 1,
                    _ => self.program_counter_position.y - 1,
                }
            },
            Direction::Down => Coord {
                x: self.program_counter_position.x,
                y: (self.program_counter_position.y + 1) % self.dimensions.y,
            },
            Direction::Left => Coord {
                x:  match self.program_counter_position.x {
                    0 => self.dimensions.x - 1,
                    _ => self.program_counter_position.x - 1,
                },
                y: self.program_counter_position.y,
            },
            Direction::Right => Coord {
                x: (self.program_counter_position.x + 1) % self.dimensions.x,
                y: self.program_counter_position.y,
            },
        };
    }
}

#[derive(Debug)]
pub struct Interpreter<Writable: Write>
{
    playfield: Playfield,
    stack: Vec<i64>,
    output_handle: Writable,
    mode: Mode,
}

impl<Writable: Write> Interpreter<Writable> {
    pub fn new(code: &str,
        program_counter_position: Option<Coord>,
        program_counter_direction: Option<Direction>,
        output_handle: Writable) -> Interpreter<Writable>
    {
        Interpreter {
            playfield: Playfield::new(code,
                program_counter_position.unwrap_or(Coord { x: 0, y: 0 }),
                program_counter_direction.unwrap_or(Direction::Right)),
            stack: Vec::new(),
            output_handle,
            mode: Mode::Command,
        }
    }
    
    pub fn execute(&mut self) -> Result<(), Box<StdError>> {
        loop {
            let curr_char = self.playfield.get_next_character();
            
            match self.mode {
                Mode::Bridge => self.mode = Mode::Command,
                
                Mode::String => {
                    match curr_char {
                        '"' => self.mode = Mode::Command,
                        _ => self.stack.push(curr_char as i64),
                    }
                }
                
                Mode::Command => {
                    match curr_char {
                        '0' ..= '9' => self.stack.push(curr_char.to_digit(10).unwrap() as i64),
                        
                        '!' | '_' | '|' | ':' | '.' | ',' => self.run_unary_operation(curr_char)?,
                        
                        '+' | '-' | '*' | '/' | '%' | '`' | '\\' | 'g' => self.run_binary_operation(curr_char)?,
                        
                        ' ' | '>' | '<' | '^' | 'v'
                        | '?' | '"' | '#' | 'p' | '&' | '~' => self.run_other_operation(curr_char)?,
                        
                        '@' => break,
                        
                        _ => return Err(BefungeError(format!("{} is not a valid command!", curr_char)).into()),
                    }
                }
            }
            
            self.playfield.update_program_counter();
        }
        Ok(())
    }
    
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
            '.' => {
                write!(self.output_handle, "{} ", value);
                self.output_handle.flush()?;
            }
            _ => {
                write!(self.output_handle, "{}", convert_int_to_char(value)?);
                self.output_handle.flush()?;
            }
        }
        Ok(())
    }
    
    fn run_binary_operation(&mut self, operation: char) -> Result<(), Box<StdError>> {
        let (a, b) = (self.stack.pop().unwrap_or(0), self.stack.pop().unwrap_or(0));
        
        match operation {
            '+' => self.stack.push(b + a),
            '-' => self.stack.push(b - a),
            '*' => self.stack.push(b * a),
            '/' => {
                match a {
                    0 => return Err(BefungeError(format!("Cannot divide {} by 0!", b)).into()),
                    _ => self.stack.push(b / a),
                }
            }
            '%' => {
                match a {
                    0 => return Err(BefungeError(format!("Cannot mod {} by 0!", b)).into()),
                    _ => self.stack.push(b % a),
                }
            }
            '`' => self.stack.push((b > a) as i64),
            
            '\\' => {
                self.stack.push(a);
                self.stack.push(b);
            }
            
            _ => self.stack.push(self.playfield.get_character_at(Coord { y: a, x: b })? as i64),
        }
        Ok(())
    }
    
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

                self.playfield.set_character_at(position, convert_int_to_char(popped_value)?)?;
            }
            '&' => {
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                self.stack.push(input.trim().parse::<i64>()
                    .map_err(|_| BefungeError(format!("{} is not a valid integer!", input)))?);
            }
            _ => {
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                
                self.stack.push(input.trim().parse::<char>()
                    .map_err(|_| BefungeError(format!("{} is not a valid character!", input)))? as i64);
            }
        }
        Ok(())
    }
}

fn convert_int_to_char(value: i64) -> Result<char, Box<StdError>> {
    if value < 0 || value > 127 {
        return Err("ASCII values must be between 0 and 127!".into());
    }
    
    std::char::from_u32(value as u32)
        .ok_or(format!("Unable to convert ASCII value {} to a char", value).into())
}
