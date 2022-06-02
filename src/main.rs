use std::collections::HashSet;

use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

struct Puzzle {
    pub squares: Vec<u8>
}

fn unique(input: &[u8]) -> Vec<&u8> {
    input.iter().collect::<HashSet<_>>().into_iter().collect()
}

fn filled(input: &[u8]) -> Vec<u8> {
    input.iter().filter(|&v| *v != 0).copied().collect()
}

fn valid_chunk(input: &[u8]) -> bool {
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

    fn get_rows(&self) -> Vec<Vec<u8>> {
        let mut rows = vec![];
        for i in 0..9 {
            rows.push(vec![]);
            let start = (i * 9) as usize;
            let squares = &self.squares[start..start+9];
            rows[i] = squares.to_vec();
        }
        rows

    }
    fn get_columns(&self) -> Vec<Vec<u8>> {
        let mut columns = vec![];
        for i in 0..9 {
            columns.push(vec![]);
            let mut squares = vec![];
            for row in 0..9 {
                let idx = i + (row * 9);
                squares.push(self.squares[idx]);
            }
            columns[i] = squares;
        }
        columns
    }

    fn get_boxes(&self) -> Vec<Vec<u8>> {
        let mut boxes = vec![];
        for i in 0..9 {
            boxes.push(vec![]);
            let start = ((i % 3) * 3) + (i / 3) * 27;
            let mut squares = vec![];
            for row in 0..3 {
                let start= start + (row * 9);
                squares.extend(&self.squares[start..start+3]);
            }
            boxes[i] = squares;
        }
        boxes
    }

    // are all rules satisfied when we ignore empty squares?
    pub fn is_valid(&self) -> bool {
        !(self.get_rows().iter().any(|row| !valid_chunk(row)) ||
            self.get_columns().iter().any(|col| !valid_chunk(col)) ||
            self.get_boxes().iter().any(|_box| !valid_chunk(_box)))
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
                buffer.push('\n');
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

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rudokn?", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let _canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())?;

    println!("Generating puzzle...");
    let puzzle = generate_valid_puzzle();
    println!("{}", puzzle.print());

    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // @TODO: render grid with correct values
    }

    Ok(())
}
