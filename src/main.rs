use std::collections::HashSet;

use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;

const SQUARE_SIZE: usize = 60;
const SQUARES_TO_REMOVE: usize = 40;
const START_OFFSET: usize = 30;

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

struct Puzzle {
    pub squares: Vec<u8>
}

fn valid_chunk(input: &[u8]) -> bool {
    let filled: Vec<_> = input.iter().filter(|&v| *v != 0).copied().collect();
    let unique = filled.iter().collect::<HashSet<_>>().into_iter().len();
    unique == filled.len()
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
        let filled = self.squares.iter().filter(|&v| *v != 0).copied().count();
        self.is_valid() && filled == 81
    }
}

fn generate_valid_puzzle() -> Puzzle {
    let mut rng = thread_rng();
    let mut puzzle = Puzzle::new();
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
            break
        }
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

    let mut puzzle = generate_valid_puzzle();

    let mut event_pump = sdl_context.event_pump()?;

    let mut selected_cell: Option<usize> = None;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(k), ..} => {
                    if let Some(cell) = selected_cell {
                        let num = k as i32;
                        if num >= 49 && num <= 57 {
                            puzzle.squares[cell] = (num - 48) as u8;
                        } else if k == Keycode::Backspace {
                            selected_cell = None;
                            puzzle.squares[cell] = 0;
                        }
                        println!("Valid?: {} Solved?: {}", puzzle.is_valid(), puzzle.is_solved());
                    }
                }
                _ => {}
            }
        }

        let mouse_state = event_pump.mouse_state();
        let mx = mouse_state.x() - START_OFFSET as i32;
        let my = mouse_state.y() - START_OFFSET as i32;

        if mouse_state.left() {
            let x = mx/SQUARE_SIZE as i32;
            let y = my/SQUARE_SIZE as i32;
            if x >= 0 && y >= 0 && x < 9 && y < 9 {
                let idx = x + (y * 9);
                selected_cell = Some(idx as usize);
            }
        }

        canvas.set_draw_color(Color::RGB(100, 100, 100));
        canvas.clear();
        for i in 0..81 {
            let square = puzzle.squares[i];
            let row = i/9;
            let column = i % 9;
            let x = START_OFFSET + column * SQUARE_SIZE;
            let y = START_OFFSET + row * SQUARE_SIZE;
            let size = SQUARE_SIZE as u32;
            canvas.set_draw_color(Color::GREY);
            canvas.fill_rect(rect!(x,  y, size, size))?;

            let color = if Some(i) == selected_cell {
                Color::YELLOW
            } else if mx >= 0 && my >= 0 && mx/SQUARE_SIZE as i32 == column as i32 && my/SQUARE_SIZE as i32 == row as i32 {
                Color::RGBA(255, 255, 200, 255)
            } else {
                Color::WHITE
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(rect!(x, y, size - 2, size - 2))?;
            if square != 0 {
                let (digit, w, h) = &digits[square as usize];
                canvas.copy(digit, None, Some(rect!(x + 16, y - 2, *w, *h)))?;
            }
        }
        for vertical_stripes in 0..4 {
            let x = START_OFFSET + (vertical_stripes * SQUARE_SIZE * 3);
            let y = START_OFFSET;
            canvas.set_draw_color(Color::BLACK);
            canvas.fill_rect(rect!(x-1, y, 2, 9 * SQUARE_SIZE))?;

        }
        for horizontal_stripes in 0..4 {
            let x = START_OFFSET;
            let y = START_OFFSET + (horizontal_stripes * SQUARE_SIZE * 3);
            canvas.set_draw_color(Color::BLACK);
            canvas.fill_rect(rect!(x, y-1, 9 * SQUARE_SIZE, 2))?;

        }
        canvas.present();
    }

    Ok(())
}
