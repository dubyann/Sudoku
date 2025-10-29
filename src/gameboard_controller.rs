use crate::gameboard::Gameboard;
use piston::input::GenericEvent;
use piston::input::{Button, Key, MouseButton};

pub struct GameboardController {
    pub gameboard: Gameboard,
    pub selected_cell: Option<[usize; 2]>,
    pub cursor_pos: [f64; 2],
    /// 鼠标左键当前是否按下（用于绘制按钮按下效果）
    pub mouse_pressed: bool,
    pub initial_cells: [[u8; 9]; 9],
    pub invalid_cells: Vec<[usize; 2]>,
    /// 操作历史，用于撤销（每项是整个棋盘的快照）
    pub history: Vec<[[u8; 9]; 9]>,
}

impl GameboardController {
    pub fn new(gameboard: Gameboard) -> Self {
        let initial_cells = gameboard.cells;
        Self {
            gameboard,
            selected_cell: None,
            cursor_pos: [0.0; 2],
            mouse_pressed: false,
            initial_cells,
            invalid_cells: Vec::new(),
            history: Vec::new(),
        }
    }

    pub fn event<E: GenericEvent>(&mut self, pos: [f64; 2], size: f64, e: &E) {
        if let Some(p) = e.mouse_cursor_args() {
            self.cursor_pos = p;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            // mark pressed for visual feedback
            self.mouse_pressed = true;

            let mx = self.cursor_pos[0];
            let my = self.cursor_pos[1];

            // First: check if user clicked on one of the bottom buttons (Undo/Reset/Random)
            let btn_w = 96.0_f64;
            let btn_h = (14u32 as f64) + 10.0; // default hud_font_size 14
            let btn_spacing = 12.0_f64;
            let btn_count = 3.0;
            let total_w = btn_count * btn_w + (btn_count - 1.0) * btn_spacing;
            let start_x = pos[0] + (size - total_w) / 2.0;
            let start_y = pos[1] + size + 12.0; // matches view's gap

            for i in 0..3 {
                let bx = start_x + i as f64 * (btn_w + btn_spacing);
                let by = start_y;
                if mx >= bx && mx < bx + btn_w && my >= by && my < by + btn_h {
                    match i {
                        0 => { self.undo(); }
                        1 => { self.reset(); }
                        2 => { self.randomize(40); }
                        _ => {}
                    }
                    return;
                }
            }

            // Otherwise, if inside board, update selected cell
            let x = mx - pos[0];
            let y = my - pos[1];
            if x >= 0.0 && x < size && y >= 0.0 && y < size {
                let cell_x = (x / size * 9.0) as usize;
                let cell_y = (y / size * 9.0) as usize;
                self.selected_cell = Some([cell_x, cell_y]);
            }
        }

        // mouse release updates pressed flag so UI can show active state only while pressed
        if let Some(Button::Mouse(MouseButton::Left)) = e.release_args() {
            self.mouse_pressed = false;
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            // Movement: arrow keys move the selected cell (with boundary protection)
            if let Some(ind) = self.selected_cell {
                let (mut x, mut y) = (ind[0] as isize, ind[1] as isize);
                match key {
                    Key::Up => {
                        y = (y - 1).max(0);
                        self.selected_cell = Some([x as usize, y as usize]);
                        return;
                    }
                    Key::Down => {
                        y = (y + 1).min(8);
                        self.selected_cell = Some([x as usize, y as usize]);
                        return;
                    }
                    Key::Left => {
                        x = (x - 1).max(0);
                        self.selected_cell = Some([x as usize, y as usize]);
                        return;
                    }
                    Key::Right => {
                        x = (x + 1).min(8);
                        self.selected_cell = Some([x as usize, y as usize]);
                        return;
                    }
                    _ => {}
                }
            }

            // For edits (digits/backspace/delete) operate on selected cell
            if let Some(ind) = self.selected_cell {
                let x = ind[0];
                let y = ind[1];
                // protect fixed initial cells
                if self.initial_cells[y][x] != 0 { return; }

                match key {
                    Key::D1|Key::D2|Key::D3|Key::D4|Key::D5|Key::D6|Key::D7|Key::D8|Key::D9 => {
                        let val = match key {
                            Key::D1=>1, Key::D2=>2, Key::D3=>3, Key::D4=>4, Key::D5=>5,
                            Key::D6=>6, Key::D7=>7, Key::D8=>8, Key::D9=>9, _=>0
                        };
                        // save history then set
                        self.push_history();
                        self.gameboard.set([x, y], val);

                        if self.gameboard.is_valid_move(y, x, val) {
                            self.invalid_cells.retain(|&pos| pos != ind);
                        } else if !self.invalid_cells.contains(&ind) {
                            self.invalid_cells.push(ind);
                        }
                    }
                    Key::Backspace|Key::Delete => {
                        self.push_history();
                        self.gameboard.set([x, y], 0);
                        self.invalid_cells.retain(|&pos| pos != ind);
                    }
                    _=>{}
                }
            }
        }
    }

    /// 将当前棋盘状态压入历史（用于撤销）
    fn push_history(&mut self) {
        // cap history size to 100
        if self.history.len() >= 100 {
            self.history.remove(0);
        }
        self.history.push(self.gameboard.cells);
    }

    /// 撤销到上一个棋盘状态（如果有）
    pub fn undo(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.gameboard.cells = prev;
            // recompute invalid cells
            self.invalid_cells.clear();
            for y in 0..9 {
                for x in 0..9 {
                    let v = self.gameboard.cells[y][x];
                    if v != 0 && !self.gameboard.is_valid_move(y, x, v) {
                        self.invalid_cells.push([x, y]);
                    }
                }
            }
        }
    }

    /// 重置为初始题目（initial_cells）
    pub fn reset(&mut self) {
        self.push_history();
        self.gameboard.cells = self.initial_cells;
        self.invalid_cells.clear();
    }

    /// 随机生成新题目（holes = 空格数量）
    pub fn randomize(&mut self, holes: usize) {
        self.push_history();
        self.gameboard = Gameboard::generate_random(holes);
        self.initial_cells = self.gameboard.cells;
        self.invalid_cells.clear();
    }
}
