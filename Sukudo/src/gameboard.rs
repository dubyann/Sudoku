//! Game board logic.

/// Size of game board.
const SIZE: usize = 9;

/// Store game board information.
pub struct Gameboard {
    /// Stores the content of the cells.
    /// '0' is an empty cell.
    pub cells: [[u8; SIZE]; SIZE],
}

impl Gameboard {
    /// Create a new game board.
    pub fn new() -> Gameboard {
        Gameboard {
            cells: [[0; SIZE]; SIZE],
        }
    }

    /// Create a game board from an existing 9x9 cell array.
    /// The input uses the same indexing as `set`/`char`: [x, y] where
    /// x is column (0..8) and y is row (0..8).
    pub fn from_cells(cells: [[u8; SIZE]; SIZE]) -> Gameboard {
        Gameboard { cells }
    }

    /// Gets the character at cell location.
    pub fn char(&self, ind: [usize; 2]) -> Option<char> {
        Some(match self.cells[ind[0]][ind[1]] {
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            9 => '9',
            _ => return None,
        })
    }

    /// Set cell value.
    pub fn set(&mut self, ind: [usize; 2], val: u8) {
        self.cells[ind[0]][ind[1]] = val;
    }
}
