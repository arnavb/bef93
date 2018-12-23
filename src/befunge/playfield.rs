
use super::interpreter::{Coord, Direction};
use super::error::Error as BefungeError;

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
    pub fn new(code: &str, program_counter_position: Coord, program_counter_direction: Direction) -> Playfield {
        // Get the longest line width as the width of the playfield
        let width = code.lines().max_by_key(|line| line.len()).unwrap_or("").len();
    
        // Create a vector of vector of chars. Each line is right-padded with spaces
        // to the longest line width.
        let code_map = code.lines()
            .map(|line| format!("{:<width$}", line, width = width)
                    .chars().collect::<Vec<_>>())
            .collect::<Vec<Vec<_>>>();
        
        let height = code_map.len();
    
        Playfield {
            code_map,
            dimensions: Coord {
                x: width as i64,
                y: height as i64,
            },
            program_counter_position,
            program_counter_direction,
        }
    }
    
    // Returns the character at the current program counter position
    pub fn get_next_character(&self) -> char {
        self.code_map[self.program_counter_position.y as usize][self.program_counter_position.x as usize]
    }
    
    // Modifies the playfield at a specific position. This is needed for put (p)
    // calls.
    // If the passed position is out of bounds, a BefungeError will be returned.
    pub fn set_character_at(&mut self, position: Coord, value: char) -> Result<(), BefungeError> {
        if position.x < 0 || position.y < 0
            || position.x > self.dimensions.x || position.y > self.dimensions.y {
            Err(BefungeError(format!("Location ({}, {}) is out of bounds!", position.x, position.y)))
        } else {
            self.code_map[position.y as usize][position.x as usize] = value;
            Ok(())
        }
    }
    
    // Gets the character on the playfield at a specific position.
    // This is needed for get (g) calls.
    // If the passed position is out of bounds, a BefungeError will be returned.
    pub fn get_character_at(&self, position: Coord) -> Result<char, BefungeError> {
        if position.x < 0 || position.y < 0
            || position.x > self.dimensions.x || position.y > self.dimensions.y {
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
    
    mod playfield {
        use super::*;
        
        mod initialization {
            use super::*;
        
            #[test]
            fn test_basic() {
                let playfield = Playfield::new("lwkwkl\ndhdhde\n333ddd",
                    Coord{ x: 0, y: 0}, Direction::Right);
                
                // Check if code_map is properly initialized
                assert_eq!(playfield.code_map,vec![
                    vec!['l', 'w', 'k', 'w', 'k', 'l'],
                    vec!['d', 'h', 'd', 'h', 'd', 'e'],
                    vec!['3', '3', '3', 'd', 'd', 'd'],
                ]);
                
                // Check if dimensions are properly initialized
                assert_eq!(playfield.dimensions, Coord { x: 6, y: 3 });
                
                // Check if program counter is initialized properly
                assert_eq!(playfield.program_counter_position, Coord { x: 0, y: 0 });
                assert_eq!(playfield.program_counter_direction, Direction::Right);
            }
            
            #[test]
            fn test_empty() {
                let playfield = Playfield::new("",
                    Coord{ x: 0, y: 0}, Direction::Right);
                
                // Check if code_map is properly initialized
                assert!(playfield.code_map.is_empty());
                
                // Check if dimensions are properly initialized
                assert_eq!(playfield.dimensions, Coord { x: 0, y: 0 });
            }
            
            #[test]
            fn test_single_row() {
                let playfield = Playfield::new("lwkwkl",
                    Coord{ x: 0, y: 0}, Direction::Right);
                
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
                    Coord{ x: 0, y: 0}, Direction::Right);
                
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
                    Coord{ x: 0, y: 0}, Direction::Right);
                    
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
                    Coord{ x: 0, y: 0}, Direction::Right);
                    
                // Check if code_map is properly initialized
                assert_eq!(playfield.code_map,vec![
                    vec!['l', 'd', 'd'],
                    vec!['w', 'w', 'e'],
                    vec!['g', ' ', ' '],
                ]);
                
                assert_eq!(playfield.dimensions, Coord { x: 3, y: 3 });
            }
        }
    }
}
