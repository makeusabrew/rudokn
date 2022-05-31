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
        for column in 0..9 {
            let mut squares = vec![];
            for row in 0..9 {
                let idx = column + (row * 9);
                squares.push(self.squares[idx]);
            }
            if !all_filled_unique(&squares) {
                return false
            }
        }

        // 3x3
        for _box in 0..9 {
            let start = ((_box % 3) * 3) + (_box / 3) * 27;
            let mut squares = vec![];
            for row in 0..3 {
                let start= start + (row * 9);
                squares.extend(&self.squares[start..start+3]);
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

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rudokn?", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
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
