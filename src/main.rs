mod code;

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{OpenGL, GlGraphics};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston::window::WindowSettings;

// A game of life simulation

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;
    
    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Conway's game of life", [
        (code::CELL_SIZE + code::CELL_PADDING) * code::ROWS as f64 + 2.0 * code::WINDOW_PADDING,
        (code::CELL_SIZE + code::CELL_PADDING) * code::COLS as f64 + 2.0 * code::WINDOW_PADDING])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap_or_else(|e| {panic!("Error creating window: {}", e)});
    
    
    let mut app = code::App::new(GlGraphics::new(opengl));

    let mut event_settings = EventSettings::new();
    event_settings.ups = code::FPS;
    event_settings.max_fps = code::FPS;

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
                    if b.state == piston::ButtonState::Release {
                        app.on_release(b);
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
