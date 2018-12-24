/* befunge/playfield.rs - Contains the struct definition of the Befunge-93 playfield
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


use super::error::Error as BefungeError;

#[derive(Debug, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
pub struct Coord {
    pub x: i64,
    pub y: i64,
}

// Represents the Befunge-93 playfield
#[derive(Debug)]
pub struct Playfield {
    code_map: Vec<Vec<char>>,
    pub dimensions: Coord,
    
    pub program_counter_position: Coord,
    pub program_counter_direction: Direction,
}

impl Playfield {
    // Initializes the playfield with the program code code,
    // an initial program counter position, and direction
    pub fn new(code: &str, program_counter_position: Coord, program_counter_direction: Direction) -> Result<Playfield, BefungeError> {
        // Get the longest line width as the width of the playfield
        let width = code.lines().max_by_key(|line| line.len()).unwrap_or("").len();
    
        // Create a vector of vector of chars. Each line is right-padded with spaces
        // to the longest line width.
        let code_map = code.lines()
            .map(|line| format!("{:<width$}", line, width = width)
                    .chars().collect::<Vec<_>>())
            .collect::<Vec<Vec<_>>>();
    
        let width = width as i64;
        let height = code_map.len() as i64;
    
        if (program_counter_position.x > width || program_counter_position.y > height)
            || (program_counter_position.x < 0 || program_counter_position.y < 0) {
            return Err(BefungeError(format!("Initial program counter position ({}, {}) is out of bounds!",
                program_counter_position.x,
                program_counter_position.y)))
        }
        
        Ok(Playfield {
            code_map,
            dimensions: Coord {
                x: width,
                y: height,
            },
            program_counter_position,
            program_counter_direction,
        })
    }
    
    // Returns the character at the current program counter position
    pub fn get_next_character(&self) -> char {
        self.code_map[self.program_counter_position.y as usize][self.program_counter_position.x as usize]
    }
    
    // Modifies the playfield at a specific position. This is needed for put (p)
    // calls.
    // If the passed position is out of bounds, a BefungeError will be returned.
    pub fn set_character_at(&mut self, position: &Coord, value: char) -> Result<(), BefungeError> {
        if (position.x < 0 || position.y < 0)
            || (position.x > self.dimensions.x || position.y > self.dimensions.y) {
            Err(BefungeError(format!("Location ({}, {}) is out of bounds!", position.x, position.y)))
        } else {
            self.code_map[position.y as usize][position.x as usize] = value;
            Ok(())
        }
    }
    
    // Gets the character on the playfield at a specific position.
    // This is needed for get (g) calls.
    // If the passed position is out of bounds, a BefungeError will be returned.
    pub fn get_character_at(&self, position: &Coord) -> Result<char, BefungeError> {
        if (position.x < 0 || position.y < 0)
            || (position.x > self.dimensions.x || position.y > self.dimensions.y) {
            Err(BefungeError(format!("Location ({}, {}) is out of bounds!", position.x, position.y)))
        } else {
            Ok(self.code_map[position.y as usize][position.x as usize])
        }
    }
    
    // Updates the position of the program counter based on it's direction
    // and position. This method handles position wraparound (assuming
    // the width/height of the playfield is less than std::i64::MAX).
    pub fn update_program_counter(&mut self) {
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

#[cfg(test)]
mod tests {
    use super::*;
        
    mod initialization {
        use super::*;
    
        #[test]
        fn test_basic() {
            let playfield = Playfield::new("lwkwkl\ndhdhde\n333ddd",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            // Check if code_map is properly initialized
            assert_eq!(playfield.code_map,vec![
                vec!['l', 'w', 'k', 'w', 'k', 'l'],
                vec!['d', 'h', 'd', 'h', 'd', 'e'],
                vec!['3', '3', '3', 'd', 'd', 'd'],
            ]);
            
            // Check if dimensions are properly initialized
            assert_eq!(playfield.dimensions, Coord { x: 6, y: 3 });
        }
        
        #[test]
        fn test_empty() {
            let playfield = Playfield::new("",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            // Check if code_map is properly initialized
            assert!(playfield.code_map.is_empty());
            
            // Check if dimensions are properly initialized
            assert_eq!(playfield.dimensions, Coord { x: 0, y: 0 });
        }
        
        #[test]
        fn test_single_row() {
            let playfield = Playfield::new("lwkwkl",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            // Check if code_map is properly initialized
            assert_eq!(playfield.code_map,vec![
                vec!['l', 'w', 'k', 'w', 'k', 'l'],
            ]);
            
            // Check if dimensions are properly initialized
            assert_eq!(playfield.dimensions, Coord { x: 6, y: 1 });
        }
        
        #[test]
        fn test_single_column() {
            let playfield = Playfield::new("l\nw\nk\nw\nk\nl",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            // Check if code_map is properly initialized
            assert_eq!(playfield.code_map,vec![
                vec!['l'],
                vec!['w'],
                vec!['k'],
                vec!['w'],
                vec!['k'],
                vec!['l'],
            ]);
            
            // Check if dimensions are properly initialized
            assert_eq!(playfield.dimensions, Coord { x: 1, y: 6 });
        }
        
        #[test]
        fn one_longer_row() {
            let playfield = Playfield::new("l\nww\nk",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
                
            // Check if code_map is properly initialized
            assert_eq!(playfield.code_map,vec![
                vec!['l', ' '],
                vec!['w', 'w'],
                vec!['k', ' '],
            ]);
            
            assert_eq!(playfield.dimensions, Coord { x: 2, y: 3 });
        }
        
        #[test]
        fn one_longer_column() {
            let playfield = Playfield::new("ldd\nwwe\ng",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
                
            // Check if code_map is properly initialized
            assert_eq!(playfield.code_map,vec![
                vec!['l', 'd', 'd'],
                vec!['w', 'w', 'e'],
                vec!['g', ' ', ' '],
            ]);
            
            assert_eq!(playfield.dimensions, Coord { x: 3, y: 3 });
        }
        
        #[test]
        fn other_attributes() {
            let playfield = Playfield::new("ldd\nwwe\ng",
                Coord{ x: 0, y: 1 }, Direction::Left).unwrap();
            
            assert_eq!(playfield.program_counter_position, Coord{ x: 0, y: 1 });
            assert_eq!(playfield.program_counter_direction, Direction::Left);
        }
    }
    
    mod get_next_character {
        use super::*;
        
        #[test]
        fn test_top_left_position() {
            let playfield = Playfield::new("l\nd",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            assert_eq!(playfield.get_next_character(), 'l');
        }
        
        #[test]
        fn test_non_top_left_position() {
            let playfield = Playfield::new("l\nd",
                Coord { x: 0, y: 1 }, Direction::Right).unwrap();
            
            assert_eq!(playfield.get_next_character(), 'd');
        }
        
        #[test]
        fn test_out_of_bounds_initial_position() {
            let playfield = Playfield::new("l\nd",
                Coord { x: 33, y: 43783 }, Direction::Right);
            
            assert!(playfield.is_err());
        }
        
        #[test]
        fn test_direction_does_not_affect_next_character() {
            let playfield = Playfield::new("l\nd",
                Coord { x: 0, y: 1 }, Direction::Up).unwrap();
            
            assert_eq!(playfield.get_next_character(), 'd');
        }
    }
    
    mod set_character_at {
        use super::*;
        
        #[test]
        fn test_basic() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            playfield.set_character_at(Coord { x: 1, y: 1 }, '#').unwrap();
            
            assert_eq!(playfield.code_map, vec![
                ['l', 'w'],
                ['g', '#'],
            ]);
        }
        
        #[test]
        fn test_out_of_bounds_access() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            let return_value = playfield.set_character_at(Coord { x: 10, y: 1 }, '#');
            
            assert!(return_value.is_err());
        }
    }
    
    mod get_character_at {
        use super::*;
        
        #[test]
        fn test_basic() {
            let playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            let character = playfield.get_character_at(Coord { x: 1, y: 1 }).unwrap();
            
            assert_eq!(character, 'g');
        }
        
        #[test]
        fn test_out_of_bounds_access() {
            let playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            let return_value = playfield.get_character_at(Coord { x: 10, y: 1 });
            
            assert!(return_value.is_err());
        }
    }
    
    mod update_program_counter {
        use super::*;
        
        #[test]
        fn test_basic_up() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 1 }, Direction::Up).unwrap();
            
            playfield.update_program_counter();
            
            assert_eq!(playfield.program_counter_position, Coord { x: 0, y: 0 });
        }
        
        #[test]
        fn test_wraparound_up() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 0 }, Direction::Up).unwrap();
            
            playfield.update_program_counter();
            
            assert_eq!(playfield.program_counter_position, Coord { x: 0, y: 1 });
        }
        
        #[test]
        fn test_basic_down() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 0 }, Direction::Down).unwrap();
            
            playfield.update_program_counter();
            
            assert_eq!(playfield.program_counter_position, Coord { x: 0, y: 1 });
        }
        
        #[test]
        fn test_wraparound_down() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 1 }, Direction::Down).unwrap();
            
            playfield.update_program_counter();
            
            assert_eq!(playfield.program_counter_position, Coord { x: 0, y: 0 });
        }
        
        #[test]
        fn test_basic_left() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 1, y: 0 }, Direction::Left).unwrap();
            
            playfield.update_program_counter();
            
            assert_eq!(playfield.program_counter_position, Coord { x: 0, y: 0 });
        }
        
        #[test]
        fn test_wraparound_left() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 0 }, Direction::Left).unwrap();
            
            playfield.update_program_counter();
            
            assert_eq!(playfield.program_counter_position, Coord { x: 1, y: 0 });
        }
        
        #[test]
        fn test_basic_right() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 0, y: 0 }, Direction::Right).unwrap();
            
            playfield.update_program_counter();
            
            assert_eq!(playfield.program_counter_position, Coord { x: 1, y: 0 });
        }
        
        #[test]
        fn test_wraparound_right() {
            let mut playfield = Playfield::new("lw\ngg",
                Coord { x: 1, y: 0 }, Direction::Right).unwrap();
            
            playfield.update_program_counter();
            
            assert_eq!(playfield.program_counter_position, Coord { x: 0, y: 0 });
        }
    }
}
