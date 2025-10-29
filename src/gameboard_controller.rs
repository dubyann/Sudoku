use crate::gameboard::{Gameboard, DEFAULT_HOLES};
use piston::input::GenericEvent;
use piston::input::{Button, Key, MouseButton};

#[derive(Clone, Copy)]
pub struct Change {
    pub x: usize,
    pub y: usize,
    pub prev: u8,
}

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
    /// 逐步变更历史：记录每次用户对单个格子的修改（用于精细撤销）
    pub changes: Vec<Change>,
    /// 当前提示（蓝色显示）：(x,y, 正确值)
    pub hint: Option<([usize; 2], u8)>,
    /// 是否显示全部答案（仅显示，不写入）
    pub show_all: bool,
    /// 显示全部答案的求解缓存
    pub solved_cache: Option<[[u8; 9]; 9]>,
    /// 是否已提交（提交后锁定，无法编辑/撤销/重置/提示）
    pub submitted: bool,
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
            changes: Vec::new(),
            hint: None,
            show_all: false,
            solved_cache: None,
            submitted: false,
        }
    }

    // 单格变更记录类型见文件顶部 `Change`

    /// 是否存在玩家输入（与初始题面不同的格子）
    fn has_user_input(&self) -> bool {
        for y in 0..9 {
            for x in 0..9 {
                if self.gameboard.cells[y][x] != self.initial_cells[y][x] {
                    return true;
                }
            }
        }
        false
    }

    pub fn event<E: GenericEvent>(
        &mut self,
        pos: [f64; 2],
        size: f64,
        window_size: [f64; 2],
        e: &E,
    ) {
        if let Some(p) = e.mouse_cursor_args() {
            self.cursor_pos = p;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            // mark pressed for visual feedback
            self.mouse_pressed = true;

            let mx = self.cursor_pos[0];
            let my = self.cursor_pos[1];

            // First: check if user clicked on one of the bottom buttons (Undo/Reset/Random)
            // Use same layout math as view (with clamping), to keep hit-test aligned with drawing
            let btn_w = 96.0_f64; // matches GameboardViewSettings defaults
            let btn_h = (14u32 as f64) + 10.0; // hud_font_size 14 + padding
            let btn_spacing = 12.0_f64; // spacing between buttons
            let btn_count = 6.0;
            let total_w = btn_count * btn_w + (btn_count - 1.0) * btn_spacing;
            let preferred_start_x = pos[0] + (size - total_w) / 2.0;
            let preferred_start_y = pos[1] + size + 12.0; // 固定在棋盘正下方
            let margin = 8.0;
            let start_x = preferred_start_x
                .max(margin)
                .min(window_size[0] - margin - total_w);
            let start_y = preferred_start_y;

            for i in 0..6 {
                let bx = start_x + i as f64 * (btn_w + btn_spacing);
                let by = start_y;
                if mx >= bx && mx < bx + btn_w && my >= by && my < by + btn_h {
                    match i {
                        0 => {
                            self.undo();
                        }
                        1 => {
                            self.reset();
                        }
                        2 => {
                            self.randomize(DEFAULT_HOLES);
                        }
                        3 => {
                            self.show_hint();
                        }
                        4 => {
                            self.toggle_show_all();
                        }
                        5 => {
                            self.submit();
                        }
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
                // 如果点击的是提示格子，则确认该提示为玩家输入
                if let Some((pos, val)) = self.hint {
                    if pos == [cell_x, cell_y] {
                        // 仅当该格可编辑且当前为空时写入
                        if self.initial_cells[cell_y][cell_x] == 0
                            && self.gameboard.cells[cell_y][cell_x] == 0
                        {
                            let prev = 0;
                            self.push_change(cell_x, cell_y, prev);
                            self.gameboard.set([cell_x, cell_y], val);
                            self.hint = None;
                            self.invalid_cells.retain(|&p| p != [cell_x, cell_y]);
                            if self.show_all {
                                self.recompute_solution_cache();
                            }
                            // 若该值仍然非法，则加入 invalid（一般不会，因为来自解）
                            if !self.gameboard.is_valid_move(cell_y, cell_x, val) {
                                self.invalid_cells.push([cell_x, cell_y]);
                            }
                            return;
                        }
                    }
                }
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
                // protect fixed initial cells and submitted state
                if self.initial_cells[y][x] != 0 || self.submitted {
                    return;
                }

                match key {
                    Key::D1
                    | Key::D2
                    | Key::D3
                    | Key::D4
                    | Key::D5
                    | Key::D6
                    | Key::D7
                    | Key::D8
                    | Key::D9 => {
                        let val = match key {
                            Key::D1 => 1,
                            Key::D2 => 2,
                            Key::D3 => 3,
                            Key::D4 => 4,
                            Key::D5 => 5,
                            Key::D6 => 6,
                            Key::D7 => 7,
                            Key::D8 => 8,
                            Key::D9 => 9,
                            _ => 0,
                        };
                        // only act if the value actually changes
                        if self.gameboard.cells[y][x] != val {
                            let prev = self.gameboard.cells[y][x];
                            self.push_change(x, y, prev);
                            self.gameboard.set([x, y], val);
                            if self.show_all {
                                self.recompute_solution_cache();
                            }
                        } else {
                            return;
                        }

                        if self.gameboard.is_valid_move(y, x, val) {
                            self.invalid_cells.retain(|&pos| pos != ind);
                        } else if !self.invalid_cells.contains(&ind) {
                            self.invalid_cells.push(ind);
                        }
                    }
                    Key::Backspace | Key::Delete => {
                        // only act if there is something to delete
                        if self.gameboard.cells[y][x] != 0 {
                            let prev = self.gameboard.cells[y][x];
                            self.push_change(x, y, prev);
                            self.gameboard.set([x, y], 0);
                            self.invalid_cells.retain(|&pos| pos != ind);
                            if self.show_all {
                                self.recompute_solution_cache();
                            }
                        }
                    }
                    _ => {}
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

    /// 记录一次对单个格子的修改（变更为新值之前的旧值）
    fn push_change(&mut self, x: usize, y: usize, prev: u8) {
        if self.changes.len() >= 200 {
            self.changes.remove(0);
        }
        self.changes.push(Change { x, y, prev });
    }

    /// 全量重新计算无效格集合（仅对玩家输入的格子做标记，初始题面不标红）
    fn recompute_invalid_cells(&mut self) {
        self.invalid_cells.clear();
        for y in 0..9 {
            for x in 0..9 {
                let v = self.gameboard.cells[y][x];
                // 仅标记玩家输入（初始为 0 的格子）
                if self.initial_cells[y][x] == 0 && v != 0 && !self.gameboard.is_valid_move(y, x, v)
                {
                    self.invalid_cells.push([x, y]);
                }
            }
        }
    }

    /// 重新计算"显示全部答案"的解缓存
    fn recompute_solution_cache(&mut self) {
        if !self.show_all {
            self.solved_cache = None;
            return;
        }
        // 基于初始题面求解（忽略玩家输入，无论对错都能求解）
        let mut clone = Gameboard::from_cells(self.initial_cells);
        if clone.solve() {
            self.solved_cache = Some(clone.cells);
        } else {
            self.solved_cache = None;
        }
    }

    /// 切换显示全部答案（只显示，不落子）
    pub fn toggle_show_all(&mut self) {
        if self.show_all {
            self.show_all = false;
            self.solved_cache = None;
        } else {
            self.show_all = true;
            self.recompute_solution_cache();
        }
    }

    /// 撤销：
    /// 1) 未选择格子：撤销最近一次用户输入（全局最近）
    /// 2) 已选择格子：只撤销该格子的最近一次输入


    /// 重置为初始题目（initial_cells）
    pub fn reset(&mut self) {
        // do nothing if there is no user input or already submitted
        if !self.has_user_input() || self.submitted {
            return;
        }
        self.push_history();
        self.gameboard.cells = self.initial_cells;
        self.invalid_cells.clear();
        self.hint = None;
        self.show_all = false;
        self.solved_cache = None;
    }

    /// 随机生成新题目（holes = 空格数量）
    pub fn randomize(&mut self, holes: usize) {
        self.push_history();
        self.gameboard = Gameboard::generate_random(holes);
        self.initial_cells = self.gameboard.cells;
        self.invalid_cells.clear();
        self.hint = None;
        self.show_all = false;
        self.solved_cache = None;
        self.submitted = false;
    }

    /// 生成一个提示：选择"最容易想到"的空格（候选数最少的可编辑空格），
    /// 基于求解结果给出正确值，蓝色显示，不直接写入棋盘。
    pub fn show_hint(&mut self) {
        // 提交后禁用 Hint
        if self.submitted {
            return;
        }
        // 若已有提示，则本次点击视为取消提示
        if self.hint.is_some() {
            self.hint = None;
            return;
        }
        // 1) 选择候选数最少的可编辑空格
        let mut best_pos: Option<[usize; 2]> = None;
        let mut best_count: usize = usize::MAX;
        for y in 0..9 {
            for x in 0..9 {
                if self.initial_cells[y][x] != 0 {
                    continue;
                } // 不提示初始题面
                if self.gameboard.cells[y][x] != 0 {
                    continue;
                } // 仅空格
                let mut cnt = 0usize;
                for num in 1..=9u8 {
                    if self.gameboard.is_valid_move(y, x, num) {
                        cnt += 1;
                    }
                }
                if cnt > 0 && cnt < best_count {
                    best_count = cnt;
                    best_pos = Some([x, y]);
                    if best_count == 1 {
                        break;
                    }
                }
            }
            if best_count == 1 {
                break;
            }
        }

        // 2) 若无合适空格，放弃提示
        let Some([tx, ty]) = best_pos else {
            self.hint = None;
            return;
        };

        // 3) 基于求解结果得到该格正确值
        let mut clone = self.gameboard.clone();
        if !clone.solve() {
            self.hint = None;
            return;
        }
        let val = clone.cells[ty][tx];
        if (1..=9).contains(&val) {
            self.hint = Some(([tx, ty], val));
        } else {
            self.hint = None;
        }
    }

    /// 提交答案：锁定棋盘，将玩家输入与正确答案对比标记颜色
    pub fn submit(&mut self) {
        if self.submitted {
            return;
        }
        // 计算正确答案（基于初始题面求解）
        let mut solution = Gameboard::from_cells(self.initial_cells);
        if !solution.solve() {
            return; // 无解则不提交
        }
        // 标记提交状态
        self.submitted = true;
        // 清除 Hint 和无效格标记（提交后用绿色/红分）
        self.hint = None;
        self.invalid_cells.clear();
        // 重新计算无效格：玩家输入与正确答案不符的标红
        for y in 0..9 {
            for x in 0..9 {
                if self.initial_cells[y][x] != 0 {
                    continue;
                } // 只检查可编辑格
                let player_val = self.gameboard.cells[y][x];
                if player_val == 0 {
                    continue;
                } // 空格不标记
                let correct_val = solution.cells[y][x];
                if player_val != correct_val {
                    self.invalid_cells.push([x, y]); // 错误的加入 invalid
                }
            }
        }
    }
}
