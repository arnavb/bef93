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
struct Position {
    x: usize,
    y: usize,
}

pub struct Interpreter {
    stack: Vec<i32>,
    direction: Direction,
}

pub fn interpret(code: &str) -> Result<String, Box<error::Error>> {
    let playfield = code.lines()
        .map(|s| s.chars().collect::<Vec<char>>())
        .collect::<Vec<Vec<char>>>();
    
    let playfield_width = playfield.iter().max_by_key(|r| r.len()).unwrap().len();
    let playfield_height = playfield.len();
    
    if playfield_width > 80 || playfield_height > 25 {
        return Err("Befunge-93 Programs MUST be within 80x25 characters!".into());
    }
    
    let mut stack: Vec<i32> = Vec::new();
    let mut mode = Mode::Command;
    let mut direction = Direction::Right;
    let mut position = Position { x: 0, y: 0 };
    let mut curr_char = playfield[position.y][position.x];
    
    let mut output = String::new();
    while curr_char != '@' {
        println!("The current character is {}", curr_char);
        println!("The current position is ({}, {})", position.x, position.y);
    
        if mode == Mode::Bridge {
            mode = Mode::Command;
            continue;
        } else if mode == Mode::String {
            if curr_char == '"' {
                mode = Mode::Command;
            } else {
                stack.push(curr_char as i32);
            }
            continue;
        }
    
        match curr_char {
            // Mode change from command mode to string mode
            '"' => mode = Mode::String,
            
            // Digit [0-9]
            curr_char if curr_char.is_digit(10) => stack.push(curr_char.to_digit(10).unwrap() as i32),
            
            // Addition
            '+' => {
                let a = stack.pop().unwrap_or(0);
                let b = stack.pop().unwrap_or(0);
                stack.push(a + b);
            }
            
            // Subtraction
            '-' => {
                let a = stack.pop().unwrap_or(0);
                let b = stack.pop().unwrap_or(0);
                stack.push(b - a);
            }
            
            // Multiplication
            '*' => {
                let a = stack.pop().unwrap_or(0);
                let b = stack.pop().unwrap_or(0);
                stack.push(a * b);
            }
            
            // Division
            '/' => {
                let a = stack.pop().unwrap_or(0);
                let b = stack.pop().unwrap_or(0);
                stack.push(b / a);
            }
            
            // Modulo
            '%' => {
                let a = stack.pop().unwrap_or(0);
                let b = stack.pop().unwrap_or(0);
                stack.push(b % a);
            }
            
            // Logical not
            '!' => {
                let value = stack.pop().unwrap_or(0);
                stack.push(!value);
            }
            
            // Greater than
            '`' => {
                let a = stack.pop().unwrap_or(0);
                let b = stack.pop().unwrap_or(0);
                
                stack.push((b > a) as i32);
            }
            
            // Playfield directions
            '^' => direction = Direction::Up,
            'v' => direction = Direction::Down,
            '<' => direction = Direction::Left,
            '>' => direction = Direction::Right,
            
            // TODO: Random direction with rand crate
            '?' => direction = match thread_rng().gen_range(0, 4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                _ => Direction::Right,
            },
            
            // Horizontal conditional
            '_' => {
                let value = stack.pop().unwrap_or(0);
                direction = match value {
                    0 => Direction::Right,
                    _ => Direction::Left,
                };
            }
            
            // Vertical conditional
            '|' => {
                let value = stack.pop().unwrap_or(0);
                direction = match value {
                    0 => Direction::Down,
                    _ => Direction::Up,
                };
            }
            
            // Duplicate top element of stack
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
            
            // Discard top of stack
            '$' => {
                stack.pop();
            }
            
            // Output top of stack as integer
            '.' => output += &stack.pop().unwrap_or(0).to_string(),
            
            // Output top of stack as char
            ',' => {
                let value = stack.pop().unwrap_or(0);
                // TODO: Convert i32 to char and push it to output
            }
            
            // Bridge (skips the next command)
            '#' => mode = Mode::Bridge,
            
            // TODO: Implement variable skipping, etc
            
            _ => return Err(format!("An unexpected character {} was found!", curr_char).into()),
        }
        
        match direction {
            Direction::Up => {
                position.y = position.y - 1 % playfield_height;
            }
            Direction::Down => {
                position.y = position.y + 1 % playfield_height;
            }
            Direction::Left => {
                position.x = position.x - 1 % playfield_width;
            }
            Direction::Right => {
                position.x = position.x + 1 % playfield_width;
            }
        }
        curr_char = playfield[position.y][position.x];
    }

    Ok(output)
}
