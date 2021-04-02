#![feature(array_map)]

extern crate rand;

use glam::*;

use ggez;
use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Color, DrawParam};
use ggez::{Context, GameResult};

use std::time::{Duration, Instant};

const UPDATES_PER_SECOND: f32 = 4.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

const FIELD_WIDTH: i16 = 12;
const FIELD_HEIGHT: i16 = 18;
const FIELD_AREA: usize = (FIELD_WIDTH * FIELD_HEIGHT) as usize;

const BLOCK_SIZE: i16 = 16;

#[derive(Clone, Copy, PartialEq)]
enum Block {
    Empty,
    Tetro(usize),
    Garbage,
    Wall,
}

const PALETTE: [u32; 9] = [
    0x103242,
    0xf94144,
    0xf3722c,
    0xf8961e,
    0xf9c74f,
    0x90be6d,
    0x43aa8b,
    0x577590,
    0x081921,
];

fn create_block_shapes<const A: usize, const B: usize>(shapes: [[usize; A]; B]) -> [[Block; A]; B] {
    shapes.map(|shape| {
        shape.map(|b| {
           if b == 0 {
               Block::Empty
           } else {
               Block::Tetro(b)
           } 
        }) 
    })
}

const SHAPES: [[usize; 16]; 7] = [
    [
        0, 0, 1, 0,
        0, 0, 1, 0,
        0, 0, 1, 0,
        0, 0, 1, 0,
    ],
    [
        0, 0, 2, 0,
        0, 2, 2, 0,
        0, 2, 0, 0, 
        0, 0, 0, 0,
    ],
    [
        0, 3, 0, 0,
        0, 3, 3, 0,
        0, 0, 3, 0,
        0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0,
        0, 4, 4, 0,
        0, 0, 4, 0,
        0, 0, 4, 0,
    ],
    [
        0, 0, 0, 0,
        0, 5, 5, 0,
        0, 5, 0, 0,
        0, 5, 0, 0,
    ],
    [
        0, 0, 6, 0,
        0, 6, 6, 0,
        0, 0, 6, 0,
        0, 0, 0, 0,
    ],
    [
        0, 0, 0, 0,
        0, 7, 7, 0,
        0, 7, 7, 0,
        0, 0, 0, 0,
    ],
];

enum State {
    Dropping,
    Clearing, 
    GameOver,
}

struct Game {
    last_update: Instant,
    shapes: [[Block; 16]; 7],
    field: [Block; FIELD_AREA],
    current_piece: usize,
    current_rotation: usize,
    current_x: i16,
    current_y: i16,    
}

impl Game {
    fn new(_ctx: &mut Context) -> GameResult<Game> {
        // create playing field
        let mut field = [Block::Empty; FIELD_AREA];
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                // fill in walls on edges
                if x == 0 || x == FIELD_WIDTH - 1 || y == FIELD_HEIGHT - 1 {
                    let idx = (y * FIELD_WIDTH + x) as usize;
                    field[idx] = Block::Wall;
                }
            }
        }

        let s = Game {
            shapes: create_block_shapes(SHAPES),
            field,
            current_piece: 0,
            current_rotation: 0,
            current_x: FIELD_WIDTH / 2 - 2,
            current_y: -1,
            last_update: Instant::now(),
        };

        Ok(s)
    }

    fn can_move(&self, x: i16, y: i16, rot: usize) -> bool {
        let new_x = self.current_x + x;
        let new_y = self.current_y + y;
        let new_rot = self.current_rotation + rot;

        for px in 0..4 as i16 {
            for py in 0..4 as i16 {
                if new_x + px >= 0 && new_x + px < FIELD_WIDTH {
                    if new_y + py >= 0 && new_y + py < FIELD_HEIGHT {
                        // get index into piece
                        let pi = rotate(px, py, new_rot) as usize;
                        // get index into field
                        let fi = ((new_y + py) * FIELD_WIDTH + (new_x + px)) as usize;
                        // check collision
                        match self.shapes[self.current_piece][pi] {
                            Block::Tetro(c) => {
                                if self.field[fi] != Block::Empty {
                                    return false;
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        }

        true
    }
}

fn draw_block(ctx: &mut Context, x: i16, y: i16, color: Color) -> GameResult {
    let rect = graphics::Rect::new((x*BLOCK_SIZE) as f32, (y*BLOCK_SIZE) as f32, BLOCK_SIZE as f32, BLOCK_SIZE as f32);
    let rect_mesh = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, color)?;
    
    graphics::draw(ctx, &rect_mesh, DrawParam::default())
}

fn rotate(px: i16, py: i16, dir: usize) -> i16 {
    match dir % 4 {
        0 => py * 4 + px,
        1 => 12 + py - (px * 4),
        2 => 15 - (py * 4) - px,
        3 => 3 - py + (px * 4),
        _ => 0,
    }
}


impl event::EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if !(Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE)) {
            return Ok(());
        }
        
        // move piece down each tick
        if self.can_move(0, 1, 0) {
            self.current_y += 1;
        } else {
            // save current piece to field
            for x in 0..4 {
                for y in 0..4 {
                    let block = self.shapes[self.current_piece][rotate(x, y, self.current_rotation) as usize];
                    match block {
                        Block::Tetro(_c) => {
                            let idx = ((self.current_y + y) * FIELD_WIDTH + (self.current_x + x)) as usize;
                            self.field[idx] = block;
                        },
                        _ => (), 
                    }
                }
            }

            // clear lines
            for line_y in self.current_y..(self.current_y + 4) {
                if line_y < FIELD_HEIGHT - 1 {
                    let mut is_line = true;
                    for x in 1..(FIELD_WIDTH-1) {
                        let idx = (line_y * FIELD_WIDTH + x) as usize;
                        if self.field[idx] == Block::Empty {
                            is_line = false;
                            break;
                        }
                    }
                    if is_line {
                        // shift all blocks down 1
                        for px in 1..(FIELD_WIDTH-1) {
                            for py in (1..(line_y+1)).rev() {
                                self.field[(py * FIELD_WIDTH + px) as usize] = self.field[((py-1) * FIELD_WIDTH + px) as usize];
                                self.field[px as usize] = Block::Empty;
                            }
                        }    
                    }
                }
            }

            // choose next piece
            self.current_x = FIELD_WIDTH / 2 - 2;
            self.current_y = -1;
            self.current_rotation = 0;
            let rng: usize = rand::random();
            self.current_piece = rng % 7;

            // TODO: check for game over
            if !self.can_move(0, 0, 0) {

            }
        }
        
        self.last_update = Instant::now();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::from_rgb_u32(PALETTE[0]));
        
        // draw field
        for x in 0..FIELD_WIDTH {
            for y in 0..FIELD_HEIGHT {
                let idx = (y * FIELD_WIDTH + x) as usize;
                let block = self.field[idx];
                let color = match block {
                    Block::Wall => Color::from_rgb_u32(PALETTE[8]),
                    Block::Tetro(c) => Color::from_rgb_u32(PALETTE[c]),
                    Block::Garbage => Color::from_rgb(200, 200, 200),
                    Block::Empty => Color::from_rgb_u32(PALETTE[0]),
                };
                if block != Block::Empty {
                    draw_block(ctx, x, y, color)?;
                } 
            }
        }
        
        // draw current piece
        for x in 0..4 {
            for y in 0..4 {
                let idx = rotate(x, y, self.current_rotation) as usize;
                match self.shapes[self.current_piece][idx] {
                    Block::Tetro(c) => {
                        draw_block(ctx, self.current_x + x, self.current_y + y, Color::from_rgb_u32(PALETTE[c]))?;
                    }
                    _ => ()
                }
            }
        }
        
        graphics::present(ctx)?;

        Ok(())
    }
    
    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::H | KeyCode::A => {
                if self.can_move(-1, 0, 0) {
                    self.current_x -= 1;
                }
            }
            KeyCode::L | KeyCode::D => {
                if self.can_move(1, 0, 0) {
                    self.current_x += 1;
                }
            }
            KeyCode::J | KeyCode::S => {
                if self.can_move(0, 1, 0) {
                    self.current_y += 1;
                }
            }
            KeyCode::I | KeyCode::E => {
                if self.can_move(0, 0, 1) {
                    self.current_rotation += 1;
                }
            }
            _ => (),
        };
    }
}


pub fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("hypertetris", "ggez");
    let (mut ctx, events_loop) = cb.build()?;

    let state = Game::new(&mut ctx)?;
    
    event::run(ctx, events_loop, state)
}
