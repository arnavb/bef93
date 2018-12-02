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

use std::error;

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
    x: usize,
    y: usize,
}

pub fn interpret(code: &str) -> Result<String, Box<error::Error>> {
    let playfield_width = match code.lines().max_by_key(|line| line.len()) {
        Some(line) => line.len(),
        None => return Ok("".to_string()),
    };
    
    let mut playfield = code.lines()
        .map(|line| {
            if line.len() < playfield_width {
                (&format!("{}{}", line, " ".repeat(playfield_width - line.len())))
                    .chars().collect::<Vec<char>>()
            } else {
                line.chars().collect::<Vec<char>>()
            }
        })
        .collect::<Vec<Vec<char>>>();
    
    let playfield_dimensions = Coord {
        x: playfield_width,
        y: playfield.len(),
    };
    
    let mut stack: Vec<i64> = Vec::new();
    let mut mode = Mode::Command;
    let mut direction = Direction::Right;
    let mut position = Coord { x: 0, y: 0 };
    let mut curr_char = playfield[position.y][position.x];
    
    let mut output = String::new();
    loop {
        println!("The current character is {}", curr_char);
        println!("The current position is ({}, {})", position.x, position.y);
    
        match mode {
            Mode::Bridge => mode = Mode::Command,
            Mode::String => {
                if curr_char == '"' {
                    mode = Mode::Command;
                } else {
                    stack.push(curr_char as i64);
                }
            }
            Mode::Command => {
                match curr_char {
                    // Digit [0-9]
                    curr_char if curr_char.is_digit(10) => stack.push(curr_char.to_digit(10).unwrap() as i64),
                    
                    // Space (no-op)
                    ' ' => (),
                    
                    // Add
                    '+' => {
                        let a = stack.pop().unwrap_or(0);
                        let b = stack.pop().unwrap_or(0);
                        stack.push(a + b);
                    }
                    
                    // Subtract
                    '-' => {
                        let a = stack.pop().unwrap_or(0);
                        let b = stack.pop().unwrap_or(0);
                        stack.push(b - a);
                    }
                    
                    // Multiply
                    '*' => {
                        let a = stack.pop().unwrap_or(0);
                        let b = stack.pop().unwrap_or(0);
                        stack.push(a * b);
                    }
                    
                    // Divide
                    '/' => {
                        let a = stack.pop().unwrap_or(0);
                        let b = stack.pop().unwrap_or(0);
                        
                        if a == 0 {
                            return Err(format!("{} / 0 is not a valid operation!", b).into());
                        }
                        
                        stack.push(b / a);
                    }
                    
                    // Modulo
                    '%' => {
                        let a = stack.pop().unwrap_or(0);
                        let b = stack.pop().unwrap_or(0);
                        
                        if a == 0 {
                            return Err(format!("{} % 0 is not a valid operation!", b).into());
                        }
                        
                        stack.push(b % a);
                    }
                    
                    // Not
                    '!' => {
                        let value = stack.pop().unwrap_or(0);
                        stack.push(!value);
                    }
                    
                    // Greater
                    '`' => {
                        let a = stack.pop().unwrap_or(0);
                        let b = stack.pop().unwrap_or(0);
                        
                        stack.push((b > a) as i64);
                    }
                    
                    // Playfield directions
                    '>' => direction = Direction::Right,
                    '<' => direction = Direction::Left,
                    '^' => direction = Direction::Up,
                    'v' => direction = Direction::Down,
                    
                    // Random direction
                    '?' => direction = match thread_rng().gen_range(0, 4) {
                        0 => Direction::Up,
                        1 => Direction::Down,
                        2 => Direction::Left,
                        _ => Direction::Right,
                    },
                    
                    // Horizontal if
                    '_' => {
                        let value = stack.pop().unwrap_or(0);
                        direction = match value {
                            0 => Direction::Right,
                            _ => Direction::Left,
                        };
                    }
                    
                    // Vertical if
                    '|' => {
                        let value = stack.pop().unwrap_or(0);
                        direction = match value {
                            0 => Direction::Down,
                            _ => Direction::Up,
                        };
                    }
                    
                    // Stringmode
                    '"' => mode = Mode::String,
                    
                    // Dup
                    ':' => {
                        let value = stack.last().cloned().unwrap_or(0);
                        stack.push(value);
                    }
                    
                    // Swap
                    '\\' => {
                        let a = stack.pop().unwrap_or(0);
                        let b = stack.pop().unwrap_or(0);
                        
                        stack.push(a);
                        stack.push(b);
                    }
                    
                    // Pop
                    '$' => {
                        stack.pop();
                    }
                    
                    // Pop and output as integer with space
                    '.' => output += &format!("{} ", stack.pop().unwrap_or(0).to_string()),
                    
                    // Pop and output as char
                    ',' => output.push(convert_int_to_char(stack.pop().unwrap_or(0))?),
                    
                    // Bridge
                    '#' => mode = Mode::Bridge,
                    
                    // Get
                    'g' => {
                        let y = stack.pop().unwrap_or(0);
                        let x = stack.pop().unwrap_or(0);
                        
                        if y < 0 || x < 0 {
                            return Err(format!("Get at ({}, {}) is out of bounds!", x, y).into())
                        }
                        
                        let value = match playfield.get(y as usize) {
                            Some(line) => {
                                match line.get(x as usize) {
                                    Some(value) => value,
                                    None => return Err(format!("Get at ({}, {}) is out of bounds!", x, y).into()),
                                }
                            }
                            None => return Err(format!("Get at ({}, {}) is out of bounds!", x, y).into()),
                        };
                        stack.push(*value as i64);
                    }
                    
                    // Put
                    'p' => {
                        let y = stack.pop().unwrap_or(0);
                        let x = stack.pop().unwrap_or(0);
                        let popped_value = stack.pop().unwrap_or(0);
                        
                        if y < 0 || x < 0 {
                            return Err(format!("Put at ({}, {}) is out of bounds!", x, y).into())
                        }
                        
                        match playfield.get_mut(y as usize) {
                            Some(line) => {
                                match line.get_mut(x as usize) {
                                    Some(value) => *value = convert_int_to_char(popped_value)?,
                                    None => return Err(format!("Put at ({}, {}) is out of bounds!", x, y).into()),
                                }
                            }
                            None => return Err(format!("Put at ({}, {}) is out of bounds!", x, y).into()),
                        }
                    }
                    
                    // Input value
                    '&' => (),
                    
                    // Input character
                    '~' => (),
                    
                    // End
                    '@' => break,
                    
                    _ => return Err(format!("An unexpected character {} was found!", curr_char).into()),
                }
            }
        }

        println!("The stack contains: {:?}", stack);
        position = match direction {
            Direction::Up => Coord {
                x: position.x,
                y: (position.y - 1) % playfield_dimensions.y,
            },
            Direction::Down => Coord {
                x: position.x,
                y: (position.y + 1) % playfield_dimensions.y,
            },
            Direction::Left => Coord {
                x: (position.x - 1) % playfield_dimensions.x,
                y: position.y,
            },
            Direction::Right => Coord {
                x: (position.x + 1) % playfield_dimensions.x,
                y: position.y,
            },
        };
        
        curr_char = playfield[position.y][position.x];
    }

    Ok(output)
}

fn convert_int_to_char(value: i64) -> Result<char, Box<error::Error>> {
    if value < 0 || value > 127 {
        return Err("ASCII values must be between 0 and 127!".into());
    }
    
    // let value = value as u32;
    
    match std::char::from_u32(value as u32) {
        Some(value) => Ok(value),
        None => Err(format!("Unable to convert ASCII value {} to a char", value).into())
    }
}
