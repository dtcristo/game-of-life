extern crate ggez;

use std::time::{Duration, Instant};

use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{Keycode, Mod, MouseButton, MouseState};
use ggez::graphics::{DrawMode, Rect};
use ggez::{event, graphics, mouse, Context, ContextBuilder, GameResult};

const GRID_SIZE: (usize, usize) = (32, 24);
const GRID_CELL_SIZE: (usize, usize) = (25, 25);

// 800 x 600
const SCREEN_SIZE: (u32, u32) = (
    GRID_SIZE.0 as u32 * GRID_CELL_SIZE.0 as u32,
    GRID_SIZE.1 as u32 * GRID_CELL_SIZE.1 as u32,
);

const TICKS_PER_SECOND: f32 = 6.0;
const MILLIS_PER_TICK: u64 = (1.0 / TICKS_PER_SECOND * 1000.0) as u64;

fn pos_from_screen_coords(x: i32, y: i32) -> (usize, usize) {
    (
        (x as usize / GRID_CELL_SIZE.0),
        (y as usize / GRID_CELL_SIZE.1),
    )
}

fn pos_to_rect(x: usize, y: usize) -> Rect {
    Rect::new_i32(
        x as i32 * GRID_CELL_SIZE.0 as i32,
        y as i32 * GRID_CELL_SIZE.1 as i32,
        GRID_CELL_SIZE.0 as i32,
        GRID_CELL_SIZE.1 as i32,
    )
}

struct Board {
    grid: [[bool; GRID_SIZE.1]; GRID_SIZE.0],
}

impl Board {
    fn new() -> Self {
        Board {
            grid: [[false; GRID_SIZE.1]; GRID_SIZE.0],
        }
    }

    fn update(&mut self) -> GameResult<()> {
        let mut new_grid = [[false; GRID_SIZE.1]; GRID_SIZE.0];
        for x in 0..GRID_SIZE.0 {
            for y in 0..GRID_SIZE.1 {
                let alive = self.grid[x][y];
                let neighbours = self.neighbours(x, y);
                new_grid[x][y] = if alive {
                    neighbours == 2 || neighbours == 3
                } else {
                    neighbours == 3
                }
            }
        }
        self.grid = new_grid;
        Ok(())
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for x in 0..GRID_SIZE.0 {
            for y in 0..GRID_SIZE.1 {
                let alive = self.cell(x, y);
                if alive {
                    graphics::set_color(ctx, [0.0, 0.0, 1.0, 1.0].into())?;
                    graphics::rectangle(ctx, DrawMode::Fill, pos_to_rect(x, y))?;
                }
            }
        }
        Ok(())
    }

    fn cell(&self, x: usize, y: usize) -> bool {
        x < GRID_SIZE.0 && y < GRID_SIZE.1 && self.grid[x][y]
    }

    fn set_cell(&mut self, x: usize, y: usize, alive: bool) {
        self.grid[x][y] = alive;
    }

    fn neighbours(&self, x: usize, y: usize) -> u8 {
        let mut result = 0;
        let cells = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];
        for cell in cells.iter() {
            if self.cell((x as i32 + cell.0) as usize, (y as i32 + cell.1) as usize) {
                result += 1;
            }
        }
        result
    }
}

struct GameState {
    board: Board,
    last_tick: Instant,
    paused: bool,
    focused: bool,
    hover_pos: Option<(usize, usize)>,
}

impl GameState {
    fn new() -> Self {
        GameState {
            board: Board::new(),
            last_tick: Instant::now(),
            paused: false,
            focused: false,
            hover_pos: None,
        }
    }

    fn handle_click(&mut self, left_down: bool, right_down: bool, x: i32, y: i32) {
        if left_down || right_down {
            let (x_pos, y_pos) = pos_from_screen_coords(x, y);
            self.board.set_cell(x_pos, y_pos, left_down);
        }
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Update board when not paused and ready for new tick
        if !self.paused && Instant::now() - self.last_tick >= Duration::from_millis(MILLIS_PER_TICK)
        {
            self.board.update()?;
            self.last_tick = Instant::now();
        }
        // Record mouse hover position when window is focused
        if self.focused {
            mouse::get_position(ctx).and_then(|point2| {
                self.hover_pos = Some(pos_from_screen_coords(point2[0] as i32, point2[1] as i32));
                Ok(())
            })?
        } else {
            self.hover_pos = None;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Background is white when paused, black otherwise
        if self.paused {
            graphics::set_background_color(ctx, [1.0, 1.0, 1.0, 1.0].into());
        } else {
            graphics::set_background_color(ctx, [0.0, 0.0, 0.0, 1.0].into());
        }
        graphics::clear(ctx);
        // Draw board and cells
        self.board.draw(ctx)?;
        // Draw hover position
        if let Some(hover_pos) = self.hover_pos {
            graphics::set_color(ctx, [0.5, 0.5, 0.5, 0.25].into())?;
            graphics::rectangle(ctx, DrawMode::Fill, pos_to_rect(hover_pos.0, hover_pos.1))?;
        }
        graphics::present(ctx);
        ggez::timer::yield_now();
        Ok(())
    }

    fn focus_event(&mut self, _ctx: &mut Context, gained: bool) {
        self.focused = gained;
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: Keycode, _keymod: Mod, _repeat: bool) {
        match keycode {
            Keycode::Escape => ctx.quit().expect("Should never fail"),
            Keycode::Space => self.paused = !self.paused,
            _ => {}
        }
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: i32, y: i32) {
        self.handle_click(
            button == MouseButton::Left,
            button == MouseButton::Right,
            x,
            y,
        );
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        state: MouseState,
        x: i32,
        y: i32,
        _xrel: i32,
        _yrel: i32,
    ) {
        self.handle_click(state.left(), state.right(), x, y);
    }
}

fn main() {
    let ctx = &mut ContextBuilder::new("game_of_life", "dtcristo")
        .window_setup(WindowSetup::default().title("game_of_life"))
        .window_mode(WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()
        .expect("Failed to build ggez context");
    let state = &mut GameState::new();
    match event::run(ctx, state) {
        Ok(_) => println!("Game exited cleanly!"),
        Err(e) => println!("Error encountered running game: {}", e),
    }
}
