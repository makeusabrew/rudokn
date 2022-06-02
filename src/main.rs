use std::collections::HashSet;

use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;

const SQUARE_SIZE: usize = 60;
const SQUARES_TO_REMOVE: usize = 40;

struct Puzzle {
    pub squares: Vec<u8>
}

fn valid_chunk(input: &[u8]) -> bool {
    let filled: Vec<_> = input.iter().filter(|&v| *v != 0).copied().collect();
    let unique: Vec<_> = filled.iter().collect::<HashSet<_>>().into_iter().collect();
    unique.len() == filled.len()
}

fn get_squares<F>(f: F) -> Vec<Vec<u8>> where
    F: Fn(usize) -> Vec<u8> {
    let mut rows = Vec::with_capacity(9);
    for i in 0..9 {
        rows.push(vec![]);
        rows[i] = f(i);
    }
    rows
}

impl Puzzle {
    pub fn new() -> Self {
        Self {
            squares: vec![]
        }
    }


    fn get_rows(&self) -> Vec<Vec<u8>> {
        get_squares(|i| {
            let start = i * 9;
            self.squares[start..start+9].to_vec()
        })
    }

    fn get_columns(&self) -> Vec<Vec<u8>> {
        get_squares(|i| {
            let mut squares = vec![];
            for row in 0..9 {
                let idx = i + (row * 9);
                squares.push(self.squares[idx]);
            }
            squares
        })
    }

    fn get_boxes(&self) -> Vec<Vec<u8>> {
        get_squares(|i| {
            let start = ((i % 3) * 3) + (i / 3) * 27;
            let mut squares = vec![];
            for row in 0..3 {
                let start= start + (row * 9);
                squares.extend(&self.squares[start..start+3]);
            }
            squares
        })
    }

    // are all rules satisfied when we ignore empty squares?
    pub fn is_valid(&self) -> bool {
        !(self.get_rows().iter().any(|row| !valid_chunk(row)) ||
            self.get_columns().iter().any(|col| !valid_chunk(col)) ||
            self.get_boxes().iter().any(|_box| !valid_chunk(_box)))
    }

    pub fn is_solved(&self) -> bool {
        let filled: Vec<_> = self.squares.iter().filter(|&v| *v != 0).copied().collect();
        self.is_valid() && filled.len() == 81
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
            break
        }
        //failures += 1;
    }

    let mut squares_to_remove: Vec<u8> = (0..81).collect();
    squares_to_remove.shuffle(&mut rng);
    for square in &squares_to_remove[0..SQUARES_TO_REMOVE] {
        let idx = *square as usize;
        puzzle.squares[idx] = 0;
    }
    puzzle
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rudokn?", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let font = ttf_context.load_font("./font.ttf", 48)?;
    let mut digits = vec![];
    for digit in 0..=9 {
        let surface = font
            .render(&digit.to_string())
            .blended(Color::RGBA(0, 0, 0, 255))
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let TextureQuery { width, height, .. } = texture.query();
        digits.push((texture, width, height));
    }

    println!("Generating puzzle...");
    let puzzle = generate_valid_puzzle();

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

        canvas.set_draw_color(Color::RGB(100, 100, 100));
        canvas.clear();
        for i in 0..81 {
            let square = puzzle.squares[i];
            let row = i/9;
            let column = i % 9;
            let x = 30 + column * SQUARE_SIZE;
            let y = 30 + row * SQUARE_SIZE;
            let size = SQUARE_SIZE as u32;
            canvas.set_draw_color(Color::GREY);
            canvas.fill_rect(Rect::new(x as i32,  y as i32, size, size))?;

            canvas.set_draw_color(Color::WHITE);
            canvas
                .fill_rect(Rect::new(x as i32 + 1, y as i32 + 1, size - 2, size - 2))?;
            if square != 0 {
                let (digit, w, h) = &digits[square as usize];
                canvas.copy(digit, None, Some(Rect::new(x as i32 + 16, y as i32 - 2, *w, *h)))?;
            }
        }
        for vertical_delim in 0..4 {
            let x = 30 + (vertical_delim * SQUARE_SIZE * 3);
            let y = 30;
            canvas.set_draw_color(Color::BLACK);
            canvas.fill_rect((Rect::new(x as i32, y, 2, 9* SQUARE_SIZE as u32)))?;

        }
        for horizontal_delim in 0..4 {
            let x = 30;
            let y = 30 + (horizontal_delim * SQUARE_SIZE * 3);
            canvas.set_draw_color(Color::BLACK);
            canvas.fill_rect((Rect::new(x, y as i32, 9 * SQUARE_SIZE as u32, 2)))?;

        }
        canvas.present();
    }

    Ok(())
}
