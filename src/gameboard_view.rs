//! Gameboard view: render the Gameboard to the screen.

use graphics::character::CharacterCache;
use graphics::types::Color;
use graphics::{Context, Graphics};
use crate::gameboard_controller::GameboardController;

/// Rendering settings for the board view.
pub struct GameboardViewSettings {
    /// Top-left position (x, y)
    pub position: [f64; 2],
    /// Board size in pixels (width == height)
    pub size: f64,
    /// Background color
    pub background_color: Color,
    /// Outer board edge color
    pub board_edge_color: Color,
    /// 3x3 section edge color
    pub section_edge_color: Color,
    /// Cell edge color
    pub cell_edge_color: Color,
    /// Outer board edge radius (line width)
    pub board_edge_radius: f64,
    /// Section edge radius
    pub section_edge_radius: f64,
    /// Cell edge radius
    pub cell_edge_radius: f64,
    /// Selected cell background color
    pub selected_cell_background_color: Color,
    /// Text color for numbers
    pub text_color: Color,
    /// Padding inside the view (pixels) between board edge and cells
    pub padding: f64,
    /// Current window size — updated each frame by `main.rs` so view can layout overlays
    pub window_size: [f64; 2],
    // Button appearance / layout
    pub btn_width: f64,
    pub btn_height: f64,
    pub btn_spacing: f64,
    pub btn_bg_color: Color,
    pub btn_hover_color: Color,
    pub btn_active_color: Color,
    pub btn_border_color: Color,
    pub btn_text_color: Color,
    /// HUD anchor position
    pub hud_anchor: HudAnchor,
    /// HUD font size
    pub hud_font_size: u32,
    /// HUD background color (RGBA)
    pub hud_bg_color: Color,
    /// HUD text color
    pub hud_text_color: Color,
}

impl GameboardViewSettings {
    /// Create default settings
    pub fn new() -> Self {
        Self {
            position: [10.0; 2],
            size: 400.0,
            background_color: [0.8, 0.8, 1.0, 1.0],
            board_edge_color: [0.0, 0.0, 0.2, 1.0],
            section_edge_color: [0.0, 0.0, 0.2, 1.0],
            cell_edge_color: [0.0, 0.0, 0.2, 1.0],
            board_edge_radius: 3.0,
            section_edge_radius: 2.0,
            cell_edge_radius: 1.0,
            selected_cell_background_color: [0.9, 0.9, 1.0, 1.0],
            text_color: [0.0, 0.0, 0.1, 1.0],
            padding: 10.0,
            hud_anchor: HudAnchor::TopLeft,
            hud_font_size: 14,
            hud_bg_color: [1.0, 1.0, 1.0, 0.85],
            hud_text_color: [0.0, 0.0, 0.0, 0.85],
            window_size: [512.0, 512.0],
            btn_width: 96.0,
            btn_height: 14.0 + 10.0,
            btn_spacing: 12.0,
            btn_bg_color: [0.96, 0.96, 0.96, 1.0],
            btn_hover_color: [0.88, 0.9, 1.0, 1.0],
            btn_active_color: [0.75, 0.85, 1.0, 1.0],
            btn_border_color: [0.2, 0.2, 0.25, 1.0],
            btn_text_color: [0.05, 0.05, 0.08, 1.0],
        }
    }
}

/// HUD anchor positions for the help overlay
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

/// View for the sudoku gameboard.
pub struct GameboardView {
    /// View settings
    pub settings: GameboardViewSettings,
}

impl GameboardView {
    /// Create a new view with given settings.
    pub fn new(settings: GameboardViewSettings) -> Self {
        GameboardView { settings }
    }

    /// Draw the board using the provided graphics context and glyph cache.
    pub fn draw<G: Graphics, C>(
        &self,
        controller: &GameboardController,
        glyphs: &mut C,
        c: &Context,
        g: &mut G,
    ) where
        C: CharacterCache<Texture = G::Texture>,
    {
        use graphics::{Image, Line, Rectangle, Transformed};

        let ref settings = self.settings;
        let board_rect = [
            settings.position[0],
            settings.position[1],
            settings.size,
            settings.size,
        ];

        // Draw board background.
        Rectangle::new(settings.background_color).draw(board_rect, &c.draw_state, c.transform, g);

        // Compute inner board area (respect padding) so board doesn't touch window edges
        let inner_left = settings.position[0] + settings.padding;
        let inner_top = settings.position[1] + settings.padding;
        let inner_size = (settings.size - 2.0 * settings.padding).max(16.0);
        let cell_size = inner_size / 9.0;

        // Draw selected cell background (selected_cell stored as [x, y]).
        if let Some(ind) = controller.selected_cell {
            let pos = [inner_left + ind[0] as f64 * cell_size, inner_top + ind[1] as f64 * cell_size];
            let cell_rect = [pos[0], pos[1], cell_size, cell_size];
            // subtle semi-transparent highlight (no thick border)
            Rectangle::new([0.9, 0.95, 1.0, 0.6]).draw(cell_rect, &c.draw_state, c.transform, g);
        }

        // Draw characters with styling: fixed cells darker, invalid cells red
        // Choose font size relative to cell size for responsiveness
        let font_size = ((cell_size * 0.65) as u32).max(12);

        for row in 0..9 {
            for col in 0..9 {
                let val = controller.gameboard.cells[row][col];
                if val == 0 { continue; }

                // choose color: invalid -> red, fixed -> darker, else text_color
                let mut text_color = settings.text_color;
                if controller.invalid_cells.contains(&[col, row]) {
                    text_color = [1.0, 0.2, 0.2, 1.0];
                } else if controller.initial_cells[row][col] != 0 {
                    text_color = [0.0, 0.0, 0.0, 1.0];
                }

                if let Some(ch) = std::char::from_digit(val as u32, 10) {
                    let cell_left = inner_left + col as f64 * cell_size;
                    let cell_top = inner_top + row as f64 * cell_size;
                    if let Ok(character) = glyphs.character(font_size, ch) {
                        // center the glyph using atlas_size and character metrics
                        let glyph_w = character.atlas_size[0] as f64;
                        let glyph_h = character.atlas_size[1] as f64;
                        let ch_x = cell_left + (cell_size - glyph_w) / 2.0 + character.left();
                        let ch_y = cell_top + (cell_size + glyph_h) / 2.0 - character.top();

                        let img = Image::new_color(text_color);
                        img.src_rect([
                            character.atlas_offset[0],
                            character.atlas_offset[1],
                            character.atlas_size[0],
                            character.atlas_size[1],
                        ]).draw(
                            character.texture,
                            &c.draw_state,
                            c.transform.trans(ch_x, ch_y),
                            g,
                        );
                    }
                }
            }
        }

        // Declare the format for cell and section lines.
        let cell_edge = Line::new(settings.cell_edge_color, settings.cell_edge_radius);
        let section_edge = Line::new(settings.section_edge_color, settings.section_edge_radius);
        // Generate and draw the lines for the Sudoku Grid using inner area
        for i in 0..=9 {
            let x = inner_left + i as f64 * cell_size;
            let y = inner_top + i as f64 * cell_size;
            let x2 = inner_left + inner_size;
            let y2 = inner_top + inner_size;

            let vline = [x, inner_top, x, y2];
            let hline = [inner_left, y, x2, y];

            if (i % 3) == 0 {
                section_edge.draw(vline, &c.draw_state, c.transform, g);
                section_edge.draw(hline, &c.draw_state, c.transform, g);
            } else {
                cell_edge.draw(vline, &c.draw_state, c.transform, g);
                cell_edge.draw(hline, &c.draw_state, c.transform, g);
            }
        }

        // Draw board edge around outer rect
        Rectangle::new_border(settings.board_edge_color, settings.board_edge_radius).draw(
            board_rect,
            &c.draw_state,
            c.transform,
            g,
        );

        // Draw a subtle padding border to indicate inner area
        let pad_rect = [
            settings.position[0] + settings.padding,
            settings.position[1] + settings.padding,
            inner_size,
            inner_size,
        ];
        Rectangle::new_border([0.0, 0.0, 0.0, 0.08], 1.0).draw(pad_rect, &c.draw_state, c.transform, g);

        // Draw bottom-centered buttons (Undo / Reset / Random) as an overlay that stays inside window
        let btn_labels = ["Undo", "Reset", "Random"];
        let btn_font = settings.hud_font_size;
        let btn_w = settings.btn_width;
        let btn_h = settings.btn_height;
        let btn_spacing = settings.btn_spacing;
        let total_w = btn_labels.len() as f64 * btn_w + (btn_labels.len() as f64 - 1.0) * btn_spacing;
        // Prefer placing below the board, but clamp so buttons remain visible within the window
        let preferred_start_x = settings.position[0] + (settings.size - total_w) / 2.0;
        let preferred_start_y = settings.position[1] + settings.size + 12.0; // gap below board
        let margin = 8.0;
        let start_x = preferred_start_x.max(margin).min(settings.window_size[0] - margin - total_w);
        // clamp vertical: don't go beyond bottom of window
        let bottom_limit_y = settings.window_size[1] - margin - btn_h;
        let start_y = preferred_start_y.min(bottom_limit_y).max(margin);

        for (i, &label) in btn_labels.iter().enumerate() {
            let bx = start_x + i as f64 * (btn_w + btn_spacing);
            let by = start_y;
            let rect = [bx, by, btn_w, btn_h];

            // hover/active detection using controller.cursor_pos and controller.mouse_pressed
            let mx = controller.cursor_pos[0];
            let my = controller.cursor_pos[1];
            let is_hover = mx >= bx && mx < bx + btn_w && my >= by && my < by + btn_h;
            let is_active = is_hover && controller.mouse_pressed;

            // choose background color based on state
            let bg = if is_active {
                settings.btn_active_color
            } else if is_hover {
                settings.btn_hover_color
            } else {
                settings.btn_bg_color
            };

            Rectangle::new(bg).draw(rect, &c.draw_state, c.transform, g);
            Rectangle::new_border(settings.btn_border_color, 1.0).draw(rect, &c.draw_state, c.transform, g);

            // draw label centered
            let mut text_w = 0.0;
            for ch in label.chars() {
                if let Ok(g) = glyphs.character(btn_font, ch) {
                    text_w += g.advance_width();
                }
            }
            let mut tx = bx + (btn_w - text_w) / 2.0;
            let ty = by + (btn_h + settings.hud_font_size as f64) / 2.0 - 2.0;
            for ch in label.chars() {
                if let Ok(glyph) = glyphs.character(btn_font, ch) {
                    let gx = tx + glyph.left();
                    let gy = ty - glyph.top();
                    let img = Image::new_color(settings.btn_text_color);
                    img.src_rect([
                        glyph.atlas_offset[0],
                        glyph.atlas_offset[1],
                        glyph.atlas_size[0],
                        glyph.atlas_size[1],
                    ]).draw(glyph.texture, &c.draw_state, c.transform.trans(gx, gy), g);
                    tx += glyph.advance_width();
                }
            }
        }
    }
}
