use rand::seq::SliceRandom;
use rand::thread_rng;

pub const SIZE: usize = 9;
// Default number of holes (tweak to adjust difficulty)
pub const DEFAULT_HOLES: usize = 40;

#[derive(Clone)]
pub struct Gameboard {
    pub cells: [[u8; SIZE]; SIZE],
}

impl Gameboard {
    pub fn new() -> Self {
        Self {
            cells: [[0; SIZE]; SIZE],
        }
    }

    pub fn from_cells(cells: [[u8; SIZE]; SIZE]) -> Self {
        Self { cells }
    }

    pub fn char(&self, ind: [usize; 2]) -> Option<char> {
        // `ind` is [x, y] (column, row) in the rest of the codebase.
        match self.cells[ind[1]][ind[0]] {
            1..=9 => Some((self.cells[ind[1]][ind[0]] + b'0') as char),
            _ => None,
        }
    }

    pub fn set(&mut self, ind: [usize; 2], val: u8) {
        // interpret ind as [x, y]
        self.cells[ind[1]][ind[0]] = val;
    }

    pub fn is_valid_move(&self, row: usize, col: usize, num: u8) -> bool {
        // Ignore the value at (row, col) itself when validating
        for i in 0..SIZE {
            if i != col && self.cells[row][i] == num {
                return false;
            }
            if i != row && self.cells[i][col] == num {
                return false;
            }
        }
        let box_row = row / 3 * 3;
        let box_col = col / 3 * 3;
        for r in box_row..box_row + 3 {
            for c in box_col..box_col + 3 {
                if !(r == row && c == col) && self.cells[r][c] == num {
                    return false;
                }
            }
        }
        true
    }

    pub fn solve(&mut self) -> bool {
        for row in 0..SIZE {
            for col in 0..SIZE {
                if self.cells[row][col] == 0 {
                    for num in 1..=9 {
                        if self.is_valid_move(row, col, num) {
                            self.cells[row][col] = num;
                            if self.solve() {
                                return true;
                            }
                            self.cells[row][col] = 0;
                        }
                    }
                    return false;
                }
            }
        }
        true
    }

    pub fn generate_random(holes: usize) -> Self {
        let mut board = Self::generate_full_solution();
        let mut positions: Vec<(usize, usize)> = (0..SIZE)
            .flat_map(|r| (0..SIZE).map(move |c| (r, c)))
            .collect();
        positions.shuffle(&mut thread_rng());
        for (r, c) in positions.into_iter().take(holes) {
            board.cells[r][c] = 0;
        }
        board
    }

    fn generate_full_solution() -> Self {
        let mut board = [[0u8; SIZE]; SIZE];
        Self::fill_board(&mut board);
        Self { cells: board }
    }

    fn fill_board(board: &mut [[u8; SIZE]; SIZE]) -> bool {
        let mut rng = thread_rng();
        for row in 0..SIZE {
            for col in 0..SIZE {
                if board[row][col] == 0 {
                    let mut nums: Vec<u8> = (1..=9).collect();
                    nums.shuffle(&mut rng);
                    for &num in &nums {
                        if Self::is_valid_static(board, row, col, num) {
                            board[row][col] = num;
                            if Self::fill_board(board) {
                                return true;
                            }
                            board[row][col] = 0;
                        }
                    }
                    return false;
                }
            }
        }
        true
    }

    fn is_valid_static(board: &[[u8; SIZE]; SIZE], row: usize, col: usize, num: u8) -> bool {
        for i in 0..SIZE {
            if board[row][i] == num || board[i][col] == num {
                return false;
            }
        }
        let box_row = row / 3 * 3;
        let box_col = col / 3 * 3;
        for r in box_row..box_row + 3 {
            for c in box_col..box_col + 3 {
                if board[r][c] == num {
                    return false;
                }
            }
        }
        true
    }
}
