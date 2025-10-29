#![allow(missing_docs)]

//! Sudoku Game Main

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

pub use crate::gameboard::Gameboard;
pub use crate::gameboard_controller::GameboardController;
pub use crate::gameboard_view::{GameboardView, GameboardViewSettings};

use glutin_window::GlutinWindow;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::RenderEvent;
use piston::window::Window;
use piston::window::WindowSettings;

mod gameboard;
mod gameboard_controller;
mod gameboard_view;

fn main() {
    let opengl = OpenGL::V3_2;
    // 初始窗口设置为纵向更高，确保棋盘下方的按钮可见
    let setting = WindowSettings::new("Sudoku", [640, 750])
        .graphics_api(opengl)
        .exit_on_esc(true);
    let mut window: GlutinWindow = setting.build().expect("Could not create window");
    let mut events = Events::new(EventSettings::new().lazy(true));
    let mut gl = GlGraphics::new(opengl);

    // 随机生成题目，指定空格数量（传入空格数量）
    let gameboard = Gameboard::generate_random(gameboard::DEFAULT_HOLES);
    let mut gameboard_controller = GameboardController::new(gameboard);

    let gameboard_view_settings = GameboardViewSettings::new();
    let mut gameboard_view = GameboardView::new(gameboard_view_settings);

    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let ref mut glyphs = GlyphCache::new("assets/FiraSans-Regular.ttf", (), texture_settings)
        .expect("Could not load font");

    use piston::input::Button;
    use piston::input::Key;
    use piston::input::PressEvent;

    while let Some(e) = events.next(&mut window) {
        // 处理输入事件（controller 处理移动与数字输入）
        gameboard_controller.event(
            gameboard_view.settings.position,
            gameboard_view.settings.size,
            gameboard_view.settings.window_size,
            &e,
        );

        // 全局快捷键：U=undo, R=reset, G=randomize
        if let Some(Button::Keyboard(k)) = e.press_args() {
            match k {
                Key::U => gameboard_controller.undo(),
                Key::R => gameboard_controller.reset(),
                Key::G => gameboard_controller.randomize(gameboard::DEFAULT_HOLES),
                _ => {}
            }
        }

        // 渲染
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::clear;
                // try to get actual window size from the window object; fallback to viewport
                let (win_w, win_h) = {
                    // GlutinWindow usually provides a `size()` method returning [u32; 2]
                    let s = window.size();
                    (s.width as f64, s.height as f64)
                };
                let size = win_w.min(win_h);
                let pos = [(win_w - size) / 2.0, (win_h - size) / 2.0];
                gameboard_view.settings.position = pos;
                gameboard_view.settings.size = size;
                // inform view about current window size so overlays (buttons) can stay visible
                gameboard_view.settings.window_size = [win_w, win_h];

                clear([1.0; 4], g);
                gameboard_view.draw(&gameboard_controller, glyphs, &c, g);
            });
        }
    }
}
