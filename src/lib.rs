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
const BOARD_SIZE: usize = 19; // Standard Go board is 19x19

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
    board: [[StoneState; BOARD_SIZE]; BOARD_SIZE],
    current_player: StoneState,
    canvas_width: u32,
    canvas_height: u32,
}

#[wasm_bindgen]
impl GoGame {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas: HtmlCanvasElement) -> GoGame {
        console_log!("Initializing Go game...");
        
        // Initialize logging
        console_error_panic_hook::set_once();
        
        GoGame {
            board: [[StoneState::Empty; BOARD_SIZE]; BOARD_SIZE],
            current_player: StoneState::Black,
            canvas_width: canvas.width(),
            canvas_height: canvas.height(),
        }
    }
    
    pub fn get_board_state(&self, x: usize, y: usize) -> u8 {
        if x >= BOARD_SIZE || y >= BOARD_SIZE {
            return 0;
        }
        match self.board[y][x] {
            StoneState::Empty => 0,
            StoneState::Black => 1,
            StoneState::White => 2,
        }
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
        // Convert normalized coordinates (-1 to 1) to board coordinates (0 to 18)
        // Use rounding instead of truncation to snap to nearest intersection
        let board_x = (((x + 1.0) / 2.0 * (BOARD_SIZE - 1) as f32) + 0.5) as usize;
        let board_y = (((y + 1.0) / 2.0 * (BOARD_SIZE - 1) as f32) + 0.5) as usize;
        
        if board_x < BOARD_SIZE && board_y < BOARD_SIZE {
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
        
        if board_x < BOARD_SIZE && board_y < BOARD_SIZE {
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
