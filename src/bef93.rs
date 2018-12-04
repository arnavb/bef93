/* bef93.rs - Contains the actual implementation of the Befunge-93 interpreter.
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

use std::{io, error};
use std::io::Write;

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
enum Mode {
    String,
    Command,
    Bridge,
}

#[derive(Debug)]
struct Coord {
    x: i64,
    y: i64,
}

pub fn interpret(code: &str) -> Result<(), Box<error::Error>> {
    let playfield_width = match code.lines().max_by_key(|line| line.len()) {
        Some(line) => line.len(),
        None => return Ok(()),
    };
    
    // Create a vector of vector of chars. Each line is right-padded with spaces
    // to the longest line width.
    let mut playfield = code.lines()
        .map(|line| format!("{:<width$}", line, width = playfield_width)
                .chars().collect::<Vec<char>>())
        .collect::<Vec<Vec<char>>>();
    
    let playfield_dimensions = Coord {
        x: playfield_width as i64,
        y: playfield.len() as i64,
    };
    
    let mut stack: Vec<i64> = Vec::new();
    let mut mode = Mode::Command;
    let mut direction = Direction::Right;
    let mut position = Coord { x: 0, y: 0 };
    let mut curr_char = playfield[position.y as usize][position.x as usize];
    
    let mut rng = thread_rng();
    
    loop {
        match mode {
            Mode::Bridge => mode = Mode::Command,
            Mode::String => {
                match curr_char {
                    '"' => mode = Mode::Command,
                    _ => stack.push(curr_char as i64),
                }
            }
            Mode::Command => {
                match curr_char {
                    // Digit
                    '0' ... '9' => stack.push(curr_char.to_digit(10).unwrap() as i64),
                    
                    // Space (no-op)
                    ' ' => (),
                    
                    // Binary operations
                    '+' | '-' | '*' | '/' | '%' | '`' | '\\' => do_binary_operation(&mut stack, curr_char)?,
                    
                    // Unary operations
                    '!' | ':' | '$' => do_unary_operation(&mut stack, curr_char),
                    
                    // Playfield directions
                    '>' => direction = Direction::Right,
                    '<' => direction = Direction::Left,
                    '^' => direction = Direction::Up,
                    'v' => direction = Direction::Down,
                    
                    // Random direction
                    '?' => direction = match rng.gen_range(0, 4) {
                        0 => Direction::Up,
                        1 => Direction::Down,
                        2 => Direction::Left,
                        _ => Direction::Right,
                    },
                    
                    // Horizontal if
                    '_' => {
                        direction = match stack.pop().unwrap_or(0) {
                            0 => Direction::Right,
                            _ => Direction::Left,
                        };
                    }
                    
                    // Vertical if
                    '|' => {
                        direction = match stack.pop().unwrap_or(0) {
                            0 => Direction::Down,
                            _ => Direction::Up,
                        };
                    }
                    
                    // Stringmode
                    '"' => mode = Mode::String,
                    
                    // Pop and output as integer with space
                    '.' => {
                        print!("{} ", stack.pop().unwrap_or(0));
                        io::stdout().flush()?;
                    },
                    
                    // Pop and output as char
                    ',' => {
                        print!("{}", convert_int_to_char(stack.pop().unwrap_or(0))?);
                        io::stdout().flush()?;
                    },
                    
                    // Bridge
                    '#' => mode = Mode::Bridge,
                    
                    // Get
                    'g' => {
                        let y = stack.pop().unwrap_or(0);
                        let x = stack.pop().unwrap_or(0);
                        
                        if (x < 0 || y < 0)
                            || ((x > playfield_dimensions.x) || (y > playfield_dimensions.y)) {
                            return Err(format!("Get at ({}, {}) is out of bounds!", x, y).into())
                        }
                        
                        stack.push(playfield[y as usize][x as usize] as i64);
                    }
                    
                    // Put
                    'p' => {
                        let y = stack.pop().unwrap_or(0);
                        let x = stack.pop().unwrap_or(0);
                        let popped_value = stack.pop().unwrap_or(0);
                        
                        if (x < 0 || y < 0)
                            || ((x > playfield_dimensions.x) || (y > playfield_dimensions.y)) {
                            return Err(format!("Put at ({}, {}) is out of bounds!", x, y).into())
                        }

                        playfield[y as usize][x as usize] = convert_int_to_char(popped_value)?;
                    }
                    
                    // Input value
                    '&' => {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        
                        stack.push(input.trim().parse::<i64>()?);
                    }
                    
                    // Input character
                    '~' => {
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        
                        stack.push(input.trim().parse::<char>()? as i64);
                    },
                    
                    // End
                    '@' => break,
                    
                    _ => return Err(format!("An unexpected character '{}' was found!", curr_char).into()),
                }
            }
        }
    
        // println!("position before: {}", (position.x - 1) % playfield_dimensions.x);
        position = match direction {
            Direction::Up => Coord {
                x: position.x,
                // y: (position.y - 1) % playfield_dimensions.y,
                y: match position.y {
                    0 => playfield_dimensions.y - 1,
                    _ => position.y - 1,
                }
            },
            Direction::Down => Coord {
                x: position.x,
                y: (position.y + 1) % playfield_dimensions.y,
            },
            Direction::Left => Coord {
                x:  match position.x {
                    0 => playfield_dimensions.x - 1,
                    _ => position.x - 1,
                },
                y: position.y,
            },
            Direction::Right => Coord {
                x: (position.x + 1) % playfield_dimensions.x,
                y: position.y,
            },
        };
        // println!("curr_char: '{}', stack: {:?}", curr_char, stack);
        curr_char = playfield[position.y as usize][position.x as usize];
    }

    println!();
    Ok(())
}

fn do_unary_operation(stack: &mut Vec<i64>, operation: char) {
    let a = stack.pop().unwrap_or(0);
    
    match operation {
        // Not
        '!' => stack.push((a == 0) as i64),
        
        // Dup
        ':' => {
            stack.push(a);
            stack.push(a);
        }
        
        // Pop ($)
        _ => (),
    }
}

fn do_binary_operation(stack: &mut Vec<i64>, operation: char) -> Result<(), Box<error::Error>> {
    let (a, b) = (stack.pop().unwrap_or(0), stack.pop().unwrap_or(0));
    
    match operation {
        // Add
        '+' => stack.push(b + a),
        
        // Subtract
        '-' => stack.push(b - a),
        
        // Multiply
        '*' => stack.push(b * a),
        
        // Divide
        '/' => {
            if a < 0 {
                return Err(format!("{} / 0 is not a valid operation!", b).into());
            }
            stack.push(b / a);
        }
        
        // Modulo
        '%' => {
            if a < 0 {
                return Err(format!("{} / 0 is not a valid operation!", b).into());
            }
            stack.push(b % a);
        }
        
        // Greater
        '`' => stack.push((b > a) as i64),
        
        // Swap (\)
        _ => {
            stack.push(a);
            stack.push(b);
        }
    }
    Ok(())
} 

fn convert_int_to_char(value: i64) -> Result<char, Box<error::Error>> {
    if value < 0 || value > 127 {
        return Err("ASCII values must be between 0 and 127!".into());
    }
    
    std::char::from_u32(value as u32)
        .ok_or(format!("Unable to convert ASCII value {} to a char", value).into())
}
