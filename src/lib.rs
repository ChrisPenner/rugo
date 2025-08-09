use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

// Import the `console.log` function from the browser console
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to make logging easier
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Go game constants
const MAX_BOARD_SIZE: usize = 19; // Maximum supported board size

// Game state
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum StoneState {
    Empty,
    Black,
    White,
}

// Game history for undo/redo
#[derive(Clone, Debug)]
struct GameState {
    board: [[StoneState; MAX_BOARD_SIZE]; MAX_BOARD_SIZE],
    current_player: StoneState,
}

// Simple Go game struct without WebGPU for now
#[wasm_bindgen]
pub struct GoGame {
    board: [[StoneState; MAX_BOARD_SIZE]; MAX_BOARD_SIZE],
    board_size: usize,
    current_player: StoneState,
    canvas_width: u32,
    canvas_height: u32,
    history: Vec<GameState>,
    history_index: usize,
}

#[wasm_bindgen]
impl GoGame {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> GoGame {
        Self::new_with_size(canvas, 19)
    }
    
    pub fn new_with_size(canvas: HtmlCanvasElement, board_size: usize) -> GoGame {
        console_log!("Initializing Go game with {}x{} board...", board_size, board_size);
        
        // Initialize logging
        console_error_panic_hook::set_once();
        
        let valid_size = match board_size {
            9 | 13 | 19 => board_size,
            _ => {
                console_log!("Invalid board size {}, defaulting to 19x19", board_size);
                19
            }
        };
        
        let initial_board = [[StoneState::Empty; MAX_BOARD_SIZE]; MAX_BOARD_SIZE];
        let initial_state = GameState {
            board: initial_board,
            current_player: StoneState::Black,
        };
        
        GoGame {
            board: initial_board,
            board_size: valid_size,
            current_player: StoneState::Black,
            canvas_width: canvas.width(),
            canvas_height: canvas.height(),
            history: vec![initial_state],
            history_index: 0,
        }
    }
    
    pub fn get_board_state(&self, x: usize, y: usize) -> u8 {
        if x >= self.board_size || y >= self.board_size {
            return 0;
        }
        match self.board[y][x] {
            StoneState::Empty => 0,
            StoneState::Black => 1,
            StoneState::White => 2,
        }
    }
    
    pub fn get_board_size(&self) -> usize {
        self.board_size
    }
    
    pub fn get_current_player(&self) -> u8 {
        match self.current_player {
            StoneState::Black => 1,
            StoneState::White => 2,
            StoneState::Empty => 0,
        }
    }

    pub fn handle_click(&mut self, x: f32, y: f32) {
        console_log!("Click at ({}, {})", x, y);
        // Convert normalized coordinates (-1 to 1) to board coordinates
        // Use rounding instead of truncation to snap to nearest intersection
        let board_x = (((x + 1.0) / 2.0 * (self.board_size - 1) as f32) + 0.5) as usize;
        let board_y = (((y + 1.0) / 2.0 * (self.board_size - 1) as f32) + 0.5) as usize;
        
        if board_x < self.board_size && board_y < self.board_size {
            if self.board[board_y][board_x] == StoneState::Empty {
                self.board[board_y][board_x] = self.current_player;
                self.current_player = match self.current_player {
                    StoneState::Black => StoneState::White,
                    StoneState::White => StoneState::Black,
                    StoneState::Empty => StoneState::Black,
                };
                console_log!("Placed stone at ({}, {})", board_x, board_y);
            }
        }
    }

    pub fn handle_board_click(&mut self, board_x: usize, board_y: usize) {
        console_log!("Board click at ({}, {})", board_x, board_y);
        
        if board_x < self.board_size && board_y < self.board_size {
            if self.board[board_y][board_x] == StoneState::Empty {
                // Remove any future history if we're not at the end
                if self.history_index < self.history.len() - 1 {
                    self.history.truncate(self.history_index + 1);
                }
                
                self.board[board_y][board_x] = self.current_player;
                self.current_player = match self.current_player {
                    StoneState::Black => StoneState::White,
                    StoneState::White => StoneState::Black,
                    StoneState::Empty => StoneState::Black,
                };
                
                // Save the new state after the move is complete
                let new_state = GameState {
                    board: self.board,
                    current_player: self.current_player,
                };
                self.history.push(new_state);
                self.history_index = self.history.len() - 1;
                
                console_log!("Placed stone at ({}, {}), history index: {}", board_x, board_y, self.history_index);
            }
        }
    }
    
    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.history_index -= 1;
            let state = &self.history[self.history_index];
            self.board = state.board;
            self.current_player = state.current_player;
            console_log!("Undo: moved to state {}", self.history_index);
            true
        } else {
            false
        }
    }
    
    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.history_index += 1;
            let state = &self.history[self.history_index];
            self.board = state.board;
            self.current_player = state.current_player;
            console_log!("Redo: moved to state {}", self.history_index);
            true
        } else {
            false
        }
    }
    
    pub fn can_undo(&self) -> bool {
        self.history_index > 0
    }
    
    pub fn can_redo(&self) -> bool {
        self.history_index < self.history.len() - 1
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas_width = width;
        self.canvas_height = height;
    }
}

// Initialize function to be called from JavaScript
#[wasm_bindgen(start)]
pub fn init() {
    console_log!("WASM module loaded successfully!");
}
