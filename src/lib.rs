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

// Simple Go game struct without WebGPU for now
#[wasm_bindgen]
pub struct GoGame {
    board: [[StoneState; MAX_BOARD_SIZE]; MAX_BOARD_SIZE],
    board_size: usize,
    current_player: StoneState,
    canvas_width: u32,
    canvas_height: u32,
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
        
        GoGame {
            board: [[StoneState::Empty; MAX_BOARD_SIZE]; MAX_BOARD_SIZE],
            board_size: valid_size,
            current_player: StoneState::Black,
            canvas_width: canvas.width(),
            canvas_height: canvas.height(),
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
