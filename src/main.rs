use std::collections::HashSet;
use rand::prelude::*;

struct Puzzle {
    pub squares: Vec<u8>
}

fn unique(input: &[u8]) -> Vec<&u8> {
    input.iter().collect::<HashSet<_>>().into_iter().collect()
}

fn filled(input: &[u8]) -> Vec<u8> {
    input.iter().filter(|&v| *v != 0).map(|v| *v).collect()
}

fn all_filled_unique(input: &[u8]) -> bool {
    let filled = filled(input);
    let unique = unique(&filled);
    unique.len() == filled.len()
}

impl Puzzle {
    pub fn new() -> Self {
        Self {
            squares: vec![]
        }
    }

    // are all rules satisfied when we ignore empty squares?
    pub fn is_valid(&self) -> bool {
        // 9x1
        for row in 0..9 {
            let start = (row * 9) as usize;
            let squares = &self.squares[start..start+9];
            if !all_filled_unique(squares) {
                return false
            }
        }

        // 1x9
        for col in 0..9 {
            let mut squares = vec![];
            for row in 0..9 {
                let idx = col + (row * 9);
                squares.push(self.squares[idx]);
            }
            if !all_filled_unique(&squares) {
                return false
            }
        }

        // 3x3
        for grid in 0..9 {
            let start = ((grid % 3) * 3) + (grid / 3) * 27;
            let mut squares = vec![];
            for row in 0..3 {
                let start= start + (row * 9);
                let row_squares = &self.squares[start..start+3];
                squares.extend(row_squares);
            }
            if !all_filled_unique(&squares) {
                return false
            }
        }
        true
    }

    pub fn is_solved(&self) -> bool {
        self.is_valid() && filled(&self.squares).len() == 81
    }

    pub fn print(&self) -> String {
        let divider = "---------------------\n";
        let mut buffer = String::from(divider);
        for square in 0..81 {
            let v = self.squares[square];
            buffer.push_str(format!("{} ", v).as_str());
            if square % 9 == 8 {
                buffer.push_str("\n");
                if square % 27 == 26 {
                    buffer.push_str(divider);
                }
            } else if square % 3 == 2 {
                buffer.push_str("| ");
            }
        }
        buffer
    }
}

fn generate_valid_puzzle() -> Puzzle {
    let mut rng = thread_rng();
    let mut puzzle = Puzzle::new();
    //let mut failures = 0;
    loop {
        puzzle.squares = [0; 81].to_vec();
        for i in 0..81 {
            let mut next_squares: Vec<u8> = (1..10).collect();
            next_squares.shuffle(&mut rng);

            for square in next_squares {
                puzzle.squares[i] = square;
                if puzzle.is_valid() {
                    break
                }
            }
        }
        if puzzle.is_valid() {
            //println!("Generation failures: {}", failures);
            return puzzle
        }
        //failures += 1;
    }
}

fn main() {
    let puzzle = generate_valid_puzzle();
    println!("{}", puzzle.print());
    println!("Solved? {}", puzzle.is_solved())
}
