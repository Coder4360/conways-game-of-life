extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::ButtonArgs;

const ROWS: usize = 20;
const COLS: usize = 20;
const CELL_SIZE: f64 = 25.0;
const CELL_PADDING: f64 = 5.0;
const WINDOW_PADDING: f64 = 20.0;
const FPS: u64 = 60;
const DEBUG_LEVEL: u8 = 0;
const DEAD_CELL_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const ALIVE_CELL_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

// A game of life simulation

pub struct App {
    gl: GlGraphics,
    grid: [[bool; COLS]; ROWS],
    mouse_x: f64,
    mouse_y: f64,
    counter: u64,
    paused: bool,
    speed: i32,
    frames_between_generations: u64,
}

impl App {
    fn new(renderer: GlGraphics) -> Self {
        return App {
            gl: renderer,
            grid: [[false; COLS]; ROWS],
            mouse_x: 0.0,
            mouse_y: 0.0,
            counter: 0,
            paused: false,
            speed: 0,
            frames_between_generations: FPS,
        }
    }
    fn on_render(&mut self, args: &RenderArgs) {
        use graphics::*;

        if DEBUG_LEVEL > 3 {
            println!("Render time={}", self.counter);
        }

        // Clear the screen
        self.gl.draw(args.viewport(), |_c, gl| {
            clear(DEAD_CELL_COLOR, gl);
        });

        // Draw the grid
        for row in 0..ROWS {
            for col in 0..COLS {
                let cell = self.grid[row][col];
                let color = if cell { ALIVE_CELL_COLOR } else { DEAD_CELL_COLOR };
                let x = WINDOW_PADDING + col as f64 * (CELL_SIZE + CELL_PADDING) / 2.0;
                let y = WINDOW_PADDING + row as f64 * (CELL_SIZE + CELL_PADDING) / 2.0;
                let square = rectangle::square(x, y, CELL_SIZE);
                self.gl.draw(args.viewport(), |c, gl| {
                    let transform = c.transform
                        .trans(x, y)
                        .trans(-CELL_SIZE / 2.0, -CELL_SIZE / 2.0);
                    rectangle(color, square, transform, gl);
                });
            }
        }
    }

    fn on_update(&mut self, _args: &UpdateArgs) {
        // Update the grid every second
        self.counter += 1;
        if !self.should_generate() { return; }
        self.grid = self.next_generation();
        if DEBUG_LEVEL > 0 {
            println!("New generation"); // debug
        }
    }

    fn on_press(&mut self, args: ButtonArgs) {
        // Get the mouse position
        let button = args.button;
        match button {
            piston::Button::Mouse(btn) => self.on_mouse_click(btn),
            piston::Button::Keyboard(key) => self.on_keyboard_press(key),
            _ => {}
        }
    }

    fn on_mouse_click(&mut self, button: piston::MouseButton) {
        match button {
            piston::MouseButton::Left => self.on_mouse_left_click(),
            _ => {}
        }
    }

    fn on_mouse_left_click(&mut self) {
        let mouse_x = self.mouse_x;
        let mouse_y = self.mouse_y;
        let column = (mouse_x - WINDOW_PADDING) / (CELL_SIZE + CELL_PADDING);
        let row = (mouse_y - WINDOW_PADDING) / (CELL_SIZE + CELL_PADDING);
        if DEBUG_LEVEL > 1 {
            println!("Left click at x={} y={} row={} column={}", mouse_x, mouse_y, row, column); // debug
        }
        if column >= 0.0 && column < COLS as f64 && row >= 0.0 && row < ROWS as f64 {
            let col = column as usize;
            let row = row as usize;
            if DEBUG_LEVEL > 1 {
                println!("Position rounded to row={} column={}", row, col); // debug
            }
            self.grid[row][col] = !self.grid[row][col];
        }
        if DEBUG_LEVEL > 2 {
            println!("New grid:"); // debug
            for row in 0..ROWS {
                for col in 0..COLS {
                    print!("{}", if self.grid[row][col] { '#' } else { ' ' });
                }
                println!("");
            }
        }
    }

    fn on_keyboard_press(&mut self, button: piston::Key) {
        match button {
            piston::Key::Space => self.on_keyboard_press_space(),
            piston::Key::Up => self.on_keyboard_press_up(),
            piston::Key::Down => self.on_keyboard_press_down(),
            _ => {}
        }
    }

    fn on_keyboard_press_space(&mut self) {
        self.paused = !self.paused;
        if DEBUG_LEVEL > 0 {
            println!("Paused={}", self.paused); // debug
        }
    }

    fn on_keyboard_press_up(&mut self) {
        self.speed += 1;
        if self.speed < 0 {
            self.frames_between_generations = ((FPS as i64) * (1 - self.speed as i64)) as u64;
        }
        if self.speed > 0 {
            self.frames_between_generations = ((FPS as i64) / (1 + self.speed as i64)) as u64;
        }
        if self.speed == 0 {
            self.frames_between_generations = FPS;
        }
        if DEBUG_LEVEL > 0 {
            println!("Speed={} Frames={}", self.speed, self.frames_between_generations); // debug
        }
    }

    fn on_keyboard_press_down(&mut self) {
        self.speed -= 1;
        if self.speed < 0 {
            self.frames_between_generations = ((FPS as i64) * (1 - self.speed as i64)) as u64;
        }
        if self.speed > 0 {
            self.frames_between_generations = ((FPS as i64) / (1 + self.speed as i64)) as u64;
        }
        if self.speed == 0 {
            self.frames_between_generations = FPS;
        }
        if DEBUG_LEVEL > 0 {
            println!("Speed={} Frames={}", self.speed, self.frames_between_generations); // debug
        }
    }

    fn next_generation(&self) -> [[bool; COLS]; ROWS] {
        // Update the grid
        let mut new_grid = [[false; COLS]; ROWS];
        for row in 0..ROWS {
            for col in 0..COLS {
                let mut neighbours_sum: u8 = 0;
                for x in -1..2 {
                    for y in -1..2 {
                        if x == 0 && y == 0 {
                            continue;
                        }
                        let neighbour_row_i = (row as i32 + x) as isize;
                        let neighbour_col_i = (col as i32 + y) as isize;
                        if neighbour_row_i < 0 || neighbour_row_i >= ROWS as isize ||
                           neighbour_col_i < 0 || neighbour_col_i >= COLS as isize {
                            continue;
                        }
                        let neighbour_col = neighbour_col_i as usize;
                        let neighbour_row = neighbour_row_i as usize;
                        let value = self.grid[neighbour_row][neighbour_col];
                        neighbours_sum += value as u8;
                    }
                }
                let cell = self.grid[row][col];
                if DEBUG_LEVEL > 1 {
                    println!("Neightbour sum of cell at row={} column={} ({}) is {}", row, col, if cell {"alive"} else {"dead"}, neighbours_sum); // debug
                }
                if cell == true {
                    if neighbours_sum < 2 || neighbours_sum > 3 {
                        if DEBUG_LEVEL > 2 {
                            println!("Cell at row={} column={} dies", row, col); // debug
                        }
                    }
                    else {
                        new_grid[row][col] = true;
                    }
                } else {
                    if neighbours_sum == 3 {
                        if DEBUG_LEVEL > 2 {
                            println!("Cell at row={} column={} is born", row, col); // debug
                        }
                        new_grid[row][col] = true;
                    }
                }
            }
        }
        if DEBUG_LEVEL > 2 {
            println!("New grid:"); // debug
            for row in 0..ROWS {
                for col in 0..COLS {
                    print!("{}", if self.grid[row][col] { '#' } else { ' ' });
                }
                println!("");
            }
        }
        return new_grid;
    }
    
    fn on_move(&mut self, motion: piston::Motion) {
        match motion {
            piston::Motion::MouseCursor(pos) => self.on_cursor_move(pos),
            _ => {}
        }
    }

    fn on_cursor_move(&mut self, pos: [f64; 2]) {
        self.mouse_x = pos[0];
        self.mouse_y = pos[1];
    }

    fn should_generate(&mut self) -> bool {
        if self.paused { return false; }
        self.counter = self.counter % self.frames_between_generations;
        return self.counter == 0;
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    
    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Conway's game of life", [
        (CELL_SIZE + CELL_PADDING) * ROWS as f64 + 2.0 * WINDOW_PADDING,
        (CELL_SIZE + CELL_PADDING) * COLS as f64 + 2.0 * WINDOW_PADDING])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap_or_else(|e| {panic!("Error creating window: {}", e)});
    
    
    let mut app = App::new(GlGraphics::new(opengl));

    let mut event_settings = EventSettings::new();
    event_settings.ups = FPS;
    event_settings.max_fps = FPS;

    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.on_render(&args)
        }

        if let Some(args) = e.update_args() {
            app.on_update(&args);
        }
        
        // Add a click handler
        if let piston::Event::Input(inp, _time) = e {
            match inp {
                piston::Input::Button(b) => {
                    if b.state == piston::ButtonState::Press {
                        app.on_press(b);
                    }
                }
                piston::Input::Move(motion) => {
                    app.on_move(motion);
                }
                _ => {}
            }
        }
    }
}
