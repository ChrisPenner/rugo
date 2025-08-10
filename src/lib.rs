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
    black_captures: u32,
    white_captures: u32,
    last_move: Option<(usize, usize)>, // Track the last move position
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
    black_captures: u32,
    white_captures: u32,
    last_move: Option<(usize, usize)>, // Track the last move position
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
            black_captures: 0,
            white_captures: 0,
            last_move: None,
        };

        GoGame {
            board: initial_board,
            board_size: valid_size,
            current_player: StoneState::Black,
            canvas_width: canvas.width(),
            canvas_height: canvas.height(),
            history: vec![initial_state],
            history_index: 0,
            black_captures: 0,
            white_captures: 0,
            last_move: None,
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

    pub fn handle_board_click(&mut self, board_x: usize, board_y: usize) -> String {
        console_log!("Board click at ({}, {})", board_x, board_y);

        if board_x >= self.board_size || board_y >= self.board_size {
            return "Invalid move: Outside board bounds".to_string();
        }

        if self.board[board_y][board_x] != StoneState::Empty {
            return "Invalid move: Position already occupied".to_string();
        }

        let placed_stone = self.current_player;
        let opponent = match placed_stone {
            StoneState::Black => StoneState::White,
            StoneState::White => StoneState::Black,
            StoneState::Empty => StoneState::Empty,
        };

        // Check if this move would be suicidal
        if self.is_suicidal_move(board_x, board_y, placed_stone) {
            return "Invalid move: Cannot place stone that would be immediately captured (suicide rule)".to_string();
        }

        // Remove any future history if we're not at the end
        if self.history_index < self.history.len() - 1 {
            self.history.truncate(self.history_index + 1);
        }

        // Place the stone
        self.board[board_y][board_x] = placed_stone;

        // Update last move position
        self.last_move = Some((board_x, board_y));

        let mut total_captured = 0;
        // Check all four adjacent positions for opponent groups to capture
        let adjacent_positions = [
            (board_x.wrapping_sub(1), board_y), // Left
            (board_x + 1, board_y),             // Right
            (board_x, board_y.wrapping_sub(1)), // Up
            (board_x, board_y + 1),             // Down
        ];

        for (adj_x, adj_y) in adjacent_positions {
            if adj_x < self.board_size && adj_y < self.board_size {
                if self.board[adj_y][adj_x] == opponent {
                    let captured = self.capture_group_if_no_liberties(adj_x, adj_y, opponent);
                    total_captured += captured;
                }
            }
        }

        // Update capture count
        match placed_stone {
            StoneState::Black => self.black_captures += total_captured,
            StoneState::White => self.white_captures += total_captured,
            StoneState::Empty => {},
        }

        if total_captured > 0 {
            console_log!("Captured {} stones", total_captured);
        }

        // Switch players
        self.current_player = match self.current_player {
            StoneState::Black => StoneState::White,
            StoneState::White => StoneState::Black,
            StoneState::Empty => StoneState::Black,
        };

        // Save the new state after the move is complete
        let new_state = GameState {
            board: self.board,
            current_player: self.current_player,
            black_captures: self.black_captures,
            white_captures: self.white_captures,
            last_move: self.last_move,
        };
        self.history.push(new_state);
        self.history_index = self.history.len() - 1;

        console_log!("Placed stone at ({}, {}), history index: {}", board_x, board_y, self.history_index);
        "Move successful".to_string()
    }

    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.history_index -= 1;
            let state = &self.history[self.history_index];
            self.board = state.board;
            self.current_player = state.current_player;
            self.black_captures = state.black_captures;
            self.white_captures = state.white_captures;
            self.last_move = state.last_move;
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
            self.black_captures = state.black_captures;
            self.white_captures = state.white_captures;
            self.last_move = state.last_move;
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

    pub fn get_black_captures(&self) -> u32 {
        self.black_captures
    }

    pub fn get_white_captures(&self) -> u32 {
        self.white_captures
    }

    // Get the last move position (returns None if no move has been made)
    pub fn get_last_move(&self) -> Option<Box<[u32]>> {
        match self.last_move {
            Some((x, y)) => Some(vec![x as u32, y as u32].into_boxed_slice()),
            None => None,
        }
    }

    // Handle pass move - player passes their turn
    pub fn handle_pass(&mut self) -> String {
        console_log!("Player {} passes", match self.current_player {
            StoneState::Black => "Black",
            StoneState::White => "White",
            StoneState::Empty => "Empty",
        });

        // Remove any future history if we're not at the end
        if self.history_index < self.history.len() - 1 {
            self.history.truncate(self.history_index + 1);
        }

        // Switch players
        self.current_player = match self.current_player {
            StoneState::Black => StoneState::White,
            StoneState::White => StoneState::Black,
            StoneState::Empty => StoneState::Black,
        };

        // Clear last move since this was a pass
        self.last_move = None;

        // Save the new state after the pass
        let new_state = GameState {
            board: self.board,
            current_player: self.current_player,
            black_captures: self.black_captures,
            white_captures: self.white_captures,
            last_move: self.last_move,
        };
        self.history.push(new_state);
        self.history_index = self.history.len() - 1;

        "Pass successful".to_string()
    }

    // Serialize current game state to a compact string format
    pub fn serialize_state(&self) -> String {
        let mut state_bytes = Vec::new();

        // Add board size (1 byte)
        state_bytes.push(self.board_size as u8);

        // Add current player (1 byte: 0=empty, 1=black, 2=white)
        let player_byte = match self.current_player {
            StoneState::Empty => 0,
            StoneState::Black => 1,
            StoneState::White => 2,
        };
        state_bytes.push(player_byte);

        // Add capture counts (4 bytes each, big-endian)
        state_bytes.extend_from_slice(&self.black_captures.to_be_bytes());
        state_bytes.extend_from_slice(&self.white_captures.to_be_bytes());

        // Add board state - use 2 bits per intersection
        // Pack 4 intersections per byte to save space
        let mut board_bytes = Vec::new();
        let mut current_byte = 0u8;
        let mut bits_used = 0;

        for y in 0..self.board_size {
            for x in 0..self.board_size {
                let state_value = match self.board[y][x] {
                    StoneState::Empty => 0u8,
                    StoneState::Black => 1u8,
                    StoneState::White => 2u8,
                };

                current_byte |= state_value << (6 - bits_used);
                bits_used += 2;

                if bits_used == 8 {
                    board_bytes.push(current_byte);
                    current_byte = 0;
                    bits_used = 0;
                }
            }
        }

        // Add any remaining bits
        if bits_used > 0 {
            board_bytes.push(current_byte);
        }

        state_bytes.extend(board_bytes);

        // Encode as base64
        base64_encode(&state_bytes)
    }

    // Restore game state from a serialized string
    pub fn deserialize_state(&mut self, state_str: &str) -> bool {
        if let Some(state_bytes) = base64_decode(state_str) {
            if state_bytes.len() < 10 {
                return false; // Too short to be valid
            }

            let mut idx = 0;

            // Read board size
            let board_size = state_bytes[idx] as usize;
            if board_size != 9 && board_size != 13 && board_size != 19 {
                return false; // Invalid board size
            }
            idx += 1;

            // Read current player
            let player_byte = state_bytes[idx];
            let current_player = match player_byte {
                0 => StoneState::Empty,
                1 => StoneState::Black,
                2 => StoneState::White,
                _ => return false,
            };
            idx += 1;

            // Read capture counts
            if idx + 8 > state_bytes.len() {
                return false;
            }
            let black_captures = u32::from_be_bytes([
                state_bytes[idx], state_bytes[idx + 1],
                state_bytes[idx + 2], state_bytes[idx + 3]
            ]);
            idx += 4;
            let white_captures = u32::from_be_bytes([
                state_bytes[idx], state_bytes[idx + 1],
                state_bytes[idx + 2], state_bytes[idx + 3]
            ]);
            idx += 4;

            // Read board state
            let total_intersections = board_size * board_size;
            let expected_board_bytes = (total_intersections + 3) / 4; // Round up

            if idx + expected_board_bytes > state_bytes.len() {
                return false;
            }

            // Clear current board
            self.board = [[StoneState::Empty; MAX_BOARD_SIZE]; MAX_BOARD_SIZE];

            let mut intersection_idx = 0;
            for &byte in &state_bytes[idx..idx + expected_board_bytes] {
                for bit_pos in (0..8).step_by(2).rev() {
                    if intersection_idx >= total_intersections {
                        break;
                    }

                    let state_value = (byte >> bit_pos) & 0b11;
                    let state = match state_value {
                        0 => StoneState::Empty,
                        1 => StoneState::Black,
                        2 => StoneState::White,
                        _ => StoneState::Empty, // Invalid, treat as empty
                    };

                    let y = intersection_idx / board_size;
                    let x = intersection_idx % board_size;
                    self.board[y][x] = state;
                    intersection_idx += 1;
                }
            }

            // Update game state
            self.board_size = board_size;
            self.current_player = current_player;
            self.black_captures = black_captures;
            self.white_captures = white_captures;
            self.last_move = None; // Clear last move when loading state

            // Reset history to current state
            let new_state = GameState {
                board: self.board,
                current_player: self.current_player,
                black_captures: self.black_captures,
                white_captures: self.white_captures,
                last_move: self.last_move,
            };
            self.history = vec![new_state];
            self.history_index = 0;

            console_log!("Successfully deserialized game state");
            true
        } else {
            false
        }
    }

    // Check if a group has any liberties (empty adjacent spaces)
    fn has_liberties(&self, x: usize, y: usize, color: StoneState, visited: &mut [[bool; MAX_BOARD_SIZE]; MAX_BOARD_SIZE]) -> bool {
        if visited[y][x] || self.board[y][x] != color {
            return false;
        }

        visited[y][x] = true;

        // Check all four adjacent positions
        let adjacent_positions = [
            (x.wrapping_sub(1), y), // Left
            (x + 1, y),             // Right
            (x, y.wrapping_sub(1)), // Up
            (x, y + 1),             // Down
        ];

        for (adj_x, adj_y) in adjacent_positions {
            if adj_x < self.board_size && adj_y < self.board_size {
                if self.board[adj_y][adj_x] == StoneState::Empty {
                    return true; // Found a liberty
                } else if self.board[adj_y][adj_x] == color {
                    // Check connected stones of the same color
                    if self.has_liberties(adj_x, adj_y, color, visited) {
                        return true;
                    }
                }
            }
        }

        false
    }

    // Capture a group if it has no liberties, return number of captured stones
    fn capture_group_if_no_liberties(&mut self, x: usize, y: usize, color: StoneState) -> u32 {
        let mut visited = [[false; MAX_BOARD_SIZE]; MAX_BOARD_SIZE];

        // Check if the group has liberties
        if self.has_liberties(x, y, color, &mut visited) {
            return 0; // Group has liberties, don't capture
        }

        // Group has no liberties, capture all stones in the group
        let mut captured = 0;
        let mut to_capture = Vec::new();
        self.find_group_stones(x, y, color, &mut to_capture);

        for (cap_x, cap_y) in to_capture {
            self.board[cap_y][cap_x] = StoneState::Empty;
            captured += 1;
        }

        console_log!("Captured group of {} stones at ({}, {})", captured, x, y);
        captured
    }

    // Find all stones in a connected group of the same color
    fn find_group_stones(&self, x: usize, y: usize, color: StoneState, group: &mut Vec<(usize, usize)>) {
        if x >= self.board_size || y >= self.board_size || self.board[y][x] != color {
            return;
        }

        // Check if already in group
        if group.contains(&(x, y)) {
            return;
        }

        group.push((x, y));

        // Recursively find connected stones
        let adjacent_positions = [
            (x.wrapping_sub(1), y), // Left
            (x + 1, y),             // Right
            (x, y.wrapping_sub(1)), // Up
            (x, y + 1),             // Down
        ];

        for (adj_x, adj_y) in adjacent_positions {
            if adj_x < self.board_size && adj_y < self.board_size {
                self.find_group_stones(adj_x, adj_y, color, group);
            }
        }
    }

    // Check if placing a stone would be suicidal (violate suicide rule)
    fn is_suicidal_move(&self, x: usize, y: usize, color: StoneState) -> bool {
        // Temporarily place the stone to test
        let mut test_board = self.board;
        test_board[y][x] = color;

        let opponent = match color {
            StoneState::Black => StoneState::White,
            StoneState::White => StoneState::Black,
            StoneState::Empty => return false,
        };

        // First check if this move would capture any opponent groups
        // If it captures opponents, it's not suicidal even if it has no liberties
        let adjacent_positions = [
            (x.wrapping_sub(1), y), // Left
            (x + 1, y),             // Right
            (x, y.wrapping_sub(1)), // Up
            (x, y + 1),             // Down
        ];

        for (adj_x, adj_y) in adjacent_positions {
            if adj_x < self.board_size && adj_y < self.board_size {
                if test_board[adj_y][adj_x] == opponent {
                    // Check if this opponent group would be captured
                    let mut visited = [[false; MAX_BOARD_SIZE]; MAX_BOARD_SIZE];
                    if !self.has_liberties_on_board(&test_board, adj_x, adj_y, opponent, &mut visited) {
                        // This move would capture opponent stones, so it's not suicidal
                        return false;
                    }
                }
            }
        }

        // Now check if the placed stone (and its group) would have any liberties
        let mut visited = [[false; MAX_BOARD_SIZE]; MAX_BOARD_SIZE];
        !self.has_liberties_on_board(&test_board, x, y, color, &mut visited)
    }

    // Check liberties on a specific board state (for testing moves)
    fn has_liberties_on_board(&self, board: &[[StoneState; MAX_BOARD_SIZE]; MAX_BOARD_SIZE], x: usize, y: usize, color: StoneState, visited: &mut [[bool; MAX_BOARD_SIZE]; MAX_BOARD_SIZE]) -> bool {
        if visited[y][x] || board[y][x] != color {
            return false;
        }

        visited[y][x] = true;

        // Check all four adjacent positions
        let adjacent_positions = [
            (x.wrapping_sub(1), y), // Left
            (x + 1, y),             // Right
            (x, y.wrapping_sub(1)), // Up
            (x, y + 1),             // Down
        ];

        for (adj_x, adj_y) in adjacent_positions {
            if adj_x < self.board_size && adj_y < self.board_size {
                if board[adj_y][adj_x] == StoneState::Empty {
                    return true; // Found a liberty
                } else if board[adj_y][adj_x] == color {
                    // Check connected stones of the same color
                    if self.has_liberties_on_board(board, adj_x, adj_y, color, visited) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

// Simple base64 encoding using web-safe characters
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b1 = chunk[0] as usize;
        let b2 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };
        let b3 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };

        let combined = (b1 << 16) | (b2 << 8) | b3;

        result.push(CHARS[(combined >> 18) & 63] as char);
        result.push(CHARS[(combined >> 12) & 63] as char);
        if chunk.len() > 1 {
            result.push(CHARS[(combined >> 6) & 63] as char);
        }
        if chunk.len() > 2 {
            result.push(CHARS[combined & 63] as char);
        }
    }

    result
}

// Simple base64 decoding
fn base64_decode(data: &str) -> Option<Vec<u8>> {
    const DECODE_TABLE: [u8; 128] = {
        let mut table = [255u8; 128];
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut i = 0;
        while i < chars.len() {
            table[chars[i] as usize] = i as u8;
            i += 1;
        }
        table
    };

    let mut result = Vec::new();
    let chars: Vec<u8> = data.bytes().collect();

    for chunk in chars.chunks(4) {
        if chunk.is_empty() {
            break;
        }

        let mut values = [0u8; 4];
        for (i, &c) in chunk.iter().enumerate() {
            if c as usize >= 128 {
                return None;
            }
            let val = DECODE_TABLE[c as usize];
            if val == 255 {
                return None;
            }
            values[i] = val;
        }

        let combined = (values[0] as u32) << 18 | (values[1] as u32) << 12 | (values[2] as u32) << 6 | values[3] as u32;

        result.push((combined >> 16) as u8);
        if chunk.len() > 2 {
            result.push((combined >> 8) as u8);
        }
        if chunk.len() > 3 {
            result.push(combined as u8);
        }
    }

    Some(result)
}

// Initialize function to be called from JavaScript
#[wasm_bindgen(start)]
pub fn init() {
    console_log!("WASM module loaded successfully!");
}
