use std::collections::HashSet;
use std::time::Instant;

use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;

const CELL_SIZE: usize = 60;
const START_OFFSET: usize = 30;

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

enum Difficulty {
    Easy,
    Medium,
    Hard,
}

type MaybeValue = Option<u8>;

#[derive(Default)]
struct Cell {
    pub is_given: bool,
    pub value: MaybeValue,
    pub candidates: [bool; 9],
}

struct Puzzle {
    pub cells: Vec<Cell>,
}

fn valid_chunk(input: &[MaybeValue]) -> bool {
    let filled: Vec<_> = input.iter().filter(|&v| v.is_some()).copied().collect();
    let unique = filled.iter().collect::<HashSet<_>>().into_iter().len();
    unique == filled.len()
}

fn get_cells<F>(f: F) -> Vec<Vec<MaybeValue>>
where
    F: Fn(usize) -> Vec<MaybeValue>,
{
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
            cells: (0..81).map(|_| Default::default()).collect(),
        }
    }

    fn get_rows(&self) -> Vec<Vec<MaybeValue>> {
        get_cells(|i| {
            let start = i * 9;
            self.cells[start..start + 9]
                .iter()
                .map(|v| v.value)
                .collect()
        })
    }

    fn get_columns(&self) -> Vec<Vec<MaybeValue>> {
        get_cells(|i| {
            let mut cells = vec![];
            for row in 0..9 {
                let idx = i + (row * 9);
                cells.push(self.cells[idx].value);
            }
            cells
        })
    }

    fn get_boxes(&self) -> Vec<Vec<MaybeValue>> {
        get_cells(|i| {
            let start = ((i % 3) * 3) + (i / 3) * 27;
            let mut cells = vec![];
            for row in 0..3 {
                let start = start + (row * 9);
                cells.extend(self.cells[start..start + 3].iter().map(|v| v.value));
            }
            cells
        })
    }

    pub fn is_valid(&self) -> bool {
        !(self.get_rows().iter().any(|row| !valid_chunk(row))
            || self.get_columns().iter().any(|col| !valid_chunk(col))
            || self.get_boxes().iter().any(|_box| !valid_chunk(_box)))
    }

    pub fn is_solved(&self) -> bool {
        let filled = self.cells.iter().filter(|&c| c.value.is_some()).count();
        self.is_valid() && filled == 81
    }

    pub fn random(difficulty: Difficulty) -> Self {
        let mut rng = thread_rng();
        loop {
            let mut puzzle = Self::new();
            for i in 0..81 {
                let mut next_cell_values: Vec<u8> = (1..=9).collect();
                next_cell_values.shuffle(&mut rng);

                for value in next_cell_values {
                    puzzle.cells[i] = Cell {
                        is_given: true,
                        value: Some(value),
                        ..Default::default()
                    };
                    if puzzle.is_valid() {
                        break;
                    }
                }
            }
            if puzzle.is_valid() {
                let cells_to_remove = match difficulty {
                    Difficulty::Easy => 40,
                    Difficulty::Medium => 50,
                    Difficulty::Hard => 60,
                };

                let mut cell_indexes: Vec<u8> = (0..81).collect();
                cell_indexes.shuffle(&mut rng);

                for idx in &cell_indexes[0..cells_to_remove] {
                    let idx = *idx as usize;
                    puzzle.cells[idx] = Default::default();
                }
                return puzzle;
            }
        }
    }
}

fn handle_keyboard(
    selected_cell: Option<usize>,
    increment: i32,
    predicate: fn(usize) -> bool,
) -> Option<usize> {
    let cell = if let Some(cell) = selected_cell {
        if predicate(cell) {
            cell as i32 + increment
        } else {
            cell as i32
        }
    } else {
        0
    };
    Some(cell as usize)
}

fn main() -> Result<(), String> {
    /*
     * SDL2 init
     */
    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rudokn?", 600, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;

    /*
     * Load typeface and create usable digit textures
     */
    let font = ttf_context.load_font("./century_gothic.ttf", 48)?;
    let small_font = ttf_context.load_font("./century_gothic.ttf", 16)?;
    let mut digits = vec![];
    let mut small_digits = vec![];
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

        let surface = small_font
            .render(&digit.to_string())
            .blended(Color::RGBA(120, 120, 120, 255))
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;
        let TextureQuery { width, height, .. } = texture.query();
        small_digits.push((texture, width, height));
    }

    let mut puzzle = Puzzle::random(Difficulty::Easy);
    let mut selected_cell: Option<usize> = None;
    let mut candidate_input_mode = false;
    let mut solved_at: Option<Instant> = None;
    let mut is_valid = puzzle.is_valid();

    'running: loop {
        /*
         * Keyboard input
         */
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::LGui),
                    ..
                } => candidate_input_mode = true,
                Event::KeyUp {
                    keycode: Some(Keycode::LGui),
                    ..
                } => candidate_input_mode = false,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    selected_cell = None;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => selected_cell = handle_keyboard(selected_cell, -9, |cell| cell >= 9),
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => selected_cell = handle_keyboard(selected_cell, 9, |cell| cell < 72),
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    selected_cell =
                        handle_keyboard(selected_cell, -1, |cell| cell > 0 && cell % 9 > 0)
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    selected_cell =
                        handle_keyboard(selected_cell, 1, |cell| cell < 80 && cell % 9 < 8)
                }
                Event::KeyDown {
                    keycode: Some(k), ..
                } => {
                    if let Some(idx) = selected_cell {
                        let writable = !puzzle.cells[idx].is_given;
                        if writable {
                            let num = k as i32;
                            if (49..=57).contains(&num) {
                                if candidate_input_mode {
                                    let cidx = (num - 49) as usize;
                                    puzzle.cells[idx].candidates[cidx] =
                                        !puzzle.cells[idx].candidates[cidx];
                                } else {
                                    puzzle.cells[idx].value = Some((num - 48) as u8);
                                }
                            } else if k == Keycode::Backspace {
                                puzzle.cells[idx] = Default::default();
                            }
                            is_valid = puzzle.is_valid();
                            let is_solved = puzzle.is_solved();
                            if is_solved && solved_at.is_none() {
                                selected_cell = None;
                                solved_at = Some(Instant::now());
                            } else if !is_solved {
                                solved_at = None;
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        /*
         * Mouse input
         */
        let mouse_state = event_pump.mouse_state();
        let mx = mouse_state.x() - START_OFFSET as i32;
        let my = mouse_state.y() - START_OFFSET as i32;

        if mouse_state.left() {
            let x = mx / CELL_SIZE as i32;
            let y = my / CELL_SIZE as i32;
            if x >= 0 && y >= 0 && x < 9 && y < 9 {
                let idx = x + (y * 9);
                selected_cell = Some(idx as usize);
            }
        }

        /*
         * Rendering
         */
        canvas.set_draw_color(Color::RGB(100, 100, 100));
        canvas.clear();

        for i in 0..81 {
            let Cell {
                is_given,
                value,
                candidates,
            } = puzzle.cells[i];
            let row = i / 9;
            let column = i % 9;
            let x = START_OFFSET + column * CELL_SIZE;
            let y = START_OFFSET + row * CELL_SIZE;
            let size = CELL_SIZE as i32;

            canvas.set_draw_color(Color::GREY);
            canvas.fill_rect(rect!(x, y, size, size))?;

            let color = if Some(i) == selected_cell {
                Color::YELLOW
            } else if is_given {
                Color::RGBA(240, 240, 240, 255)
            } else if solved_at.is_some() {
                // @TODO: animate based on solved_at value
                Color::RGBA(255, 223, 0, 255)
            } else if !is_valid {
                Color::RGBA(200, 200, 200, 255)
            } else if mx >= 0 && my >= 0 && mx / size == column as i32 && my / size == row as i32 {
                Color::RGBA(255, 255, 200, 255)
            } else {
                Color::WHITE
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(rect!(x, y, size - 2, size - 2))?;

            if let Some(value) = value {
                let (digit, w, h) = &digits[value as usize];
                canvas.copy(digit, None, Some(rect!(x + 16, y - 2, *w, *h)))?;
            } else {
                for (i, is_candidate) in candidates.into_iter().enumerate() {
                    if !is_candidate {
                        continue;
                    }
                    let cx = x + ((i % 3) * 22) + 1;
                    let cy = y + ((i / 3) * 19) - 1;
                    let (digit, w, h) = &small_digits[i + 1];
                    canvas.copy(digit, None, Some(rect!(cx + 2, cy, *w, *h)))?;
                }
            }
        }

        /*
         * Box outlines
         */
        for outlines in 0..4 {
            let x = START_OFFSET + (outlines * CELL_SIZE * 3) - 1;
            let y = START_OFFSET;
            let offset = 9 * CELL_SIZE;
            let size = 2;

            canvas.set_draw_color(Color::BLACK);
            canvas.fill_rect(rect!(x, y, size, offset))?;
            canvas.fill_rect(rect!(y, x, offset, size))?;
        }

        canvas.present();
    }

    Ok(())
}
