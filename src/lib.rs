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

// Move representation for sequence encoding
#[derive(Clone, Debug)]
struct Move {
    x: Option<usize>, // None for pass moves
    y: Option<usize>, // None for pass moves
    player: StoneState,
}

// Simple Go game struct without WebGPU for now
#[wasm_bindgen]
pub struct GoGame {
    board: [[StoneState; MAX_BOARD_SIZE]; MAX_BOARD_SIZE],
    move_numbers: [[u32; MAX_BOARD_SIZE]; MAX_BOARD_SIZE], // Track move number for each position (0 = no move)
    board_size: usize,
    current_player: StoneState,
    canvas_width: u32,
    canvas_height: u32,
    move_sequence: Vec<Move>, // Chronological sequence of moves - replaces history
    move_index: usize, // Current position in move sequence (for undo/redo)
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
        let initial_move_numbers = [[0u32; MAX_BOARD_SIZE]; MAX_BOARD_SIZE];

        GoGame {
            board: initial_board,
            move_numbers: initial_move_numbers,
            board_size: valid_size,
            current_player: StoneState::Black,
            canvas_width: canvas.width(),
            canvas_height: canvas.height(),
            move_sequence: Vec::new(),
            move_index: 0,
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

    pub fn get_move_number(&self, x: usize, y: usize) -> u32 {
        if x >= self.board_size || y >= self.board_size {
            return 0;
        }
        self.move_numbers[y][x]
    }

    // Reconstruct game state from move sequence up to move_index
    fn reconstruct_state_to_index(&mut self, target_index: usize) {
        // Reset to initial state
        self.board = [[StoneState::Empty; MAX_BOARD_SIZE]; MAX_BOARD_SIZE];
        self.move_numbers = [[0u32; MAX_BOARD_SIZE]; MAX_BOARD_SIZE];
        self.current_player = StoneState::Black;
        self.black_captures = 0;
        self.white_captures = 0;
        self.last_move = None;

        // Replay moves up to target_index
        for (i, mv) in self.move_sequence.iter().enumerate().take(target_index) {
            match (mv.x, mv.y) {
                (Some(x), Some(y)) => {
                    // Stone placement move
                    self.board[y][x] = mv.player;
                    self.move_numbers[y][x] = (i + 1) as u32;
                    self.last_move = Some((x, y));

                    // Handle captures
                    let opponent = match mv.player {
                        StoneState::Black => StoneState::White,
                        StoneState::White => StoneState::Black,
                        StoneState::Empty => StoneState::Empty,
                    };

                    let adjacent_positions = [
                        (x.wrapping_sub(1), y), // Left
                        (x + 1, y),             // Right
                        (x, y.wrapping_sub(1)), // Up
                        (x, y + 1),             // Down
                    ];

                    let mut total_captured = 0;
                    for (adj_x, adj_y) in adjacent_positions {
                        if adj_x < self.board_size && adj_y < self.board_size {
                            if self.board[adj_y][adj_x] == opponent {
                                let captured = self.capture_group_if_no_liberties(adj_x, adj_y, opponent);
                                total_captured += captured;
                            }
                        }
                    }

                    // Update capture count
                    match mv.player {
                        StoneState::Black => self.black_captures += total_captured,
                        StoneState::White => self.white_captures += total_captured,
                        StoneState::Empty => {},
                    }
                }
                (None, None) => {
                    // Pass move
                    self.last_move = None;
                }
                (None, Some(_)) | (Some(_), None) => {
                    // Invalid move data - this should never happen in a properly constructed move sequence
                    console_log!("Warning: Invalid move data encountered during state reconstruction");
                }
            }

            // Update current player for next move
            self.current_player = match mv.player {
                StoneState::Black => StoneState::White,
                StoneState::White => StoneState::Black,
                StoneState::Empty => StoneState::Black,
            };
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

        // Remove any future moves if we're not at the end (truncate for new branch)
        if self.move_index < self.move_sequence.len() {
            self.move_sequence.truncate(self.move_index);
        }

        // Add move to sequence
        self.move_sequence.push(Move {
            x: Some(board_x),
            y: Some(board_y),
            player: placed_stone,
        });
        self.move_index += 1;

        // Place the stone
        self.board[board_y][board_x] = placed_stone;

        // Assign move number to this position
        self.move_numbers[board_y][board_x] = self.move_index as u32;

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

        console_log!("Placed stone at ({}, {}), move index: {}", board_x, board_y, self.move_index);
        "Move successful".to_string()
    }

    pub fn undo(&mut self) -> bool {
        if self.can_undo() {
            self.move_index -= 1;
            self.reconstruct_state_to_index(self.move_index);
            console_log!("Undo: moved to move index {}", self.move_index);
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.can_redo() {
            self.move_index += 1;
            self.reconstruct_state_to_index(self.move_index);
            console_log!("Redo: moved to move index {}", self.move_index);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        self.move_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.move_index < self.move_sequence.len()
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

        // Remove any future moves if we're not at the end (truncate for new branch)
        if self.move_index < self.move_sequence.len() {
            self.move_sequence.truncate(self.move_index);
        }

        // Add pass move to sequence
        self.move_sequence.push(Move {
            x: None,
            y: None,
            player: self.current_player,
        });
        self.move_index += 1;

        // Switch players
        self.current_player = match self.current_player {
            StoneState::Black => StoneState::White,
            StoneState::White => StoneState::Black,
            StoneState::Empty => StoneState::Black,
        };

        // Clear last move since this was a pass
        self.last_move = None;

        "Pass successful".to_string()
    }

    // Serialize current game state to a compact string format
    pub fn serialize_state(&self) -> String {
        let mut state_bytes = Vec::new();

        // Pack board size (3 bits: 0=9, 1=13, 2=19) and current player (2 bits) into 1 byte
        let board_size_code = match self.board_size {
            9 => 0u8,
            13 => 1u8,
            19 => 2u8,
            _ => 2u8, // Default to 19
        };
        let player_code = match self.current_player {
            StoneState::Empty => 0u8,
            StoneState::Black => 1u8,
            StoneState::White => 2u8,
        };
        let header_byte = (board_size_code << 2) | player_code;
        state_bytes.push(header_byte);

        // Variable-length encoding for capture counts (saves space for small numbers)
        encode_varint(&mut state_bytes, self.black_captures);
        encode_varint(&mut state_bytes, self.white_captures);

        // Encode move sequence up to current move_index
        encode_varint(&mut state_bytes, self.move_index as u32);
        for mv in self.move_sequence.iter().take(self.move_index) {
            match (mv.x, mv.y) {
                (Some(x), Some(y)) => {
                    // Stone placement: encode position (9 bits for 19x19) + player (2 bits)
                    let position = (y * self.board_size + x) as u16;
                    let player_bits = match mv.player {
                        StoneState::Black => 1u16,
                        StoneState::White => 2u16,
                        StoneState::Empty => 0u16,
                    };
                    let encoded = (position << 2) | player_bits;
                    // Store as 2 bytes (little endian)
                    state_bytes.push(encoded as u8);
                    state_bytes.push((encoded >> 8) as u8);
                }
                (None, None) => {
                    // Pass move: use special encoding 0xFFFF
                    state_bytes.push(0xFF);
                    state_bytes.push(0xFF);
                }
            }
        }

        // Encode as base64
        base64_encode(&state_bytes)
    }

    // Restore game state from a serialized string
    pub fn deserialize_state(&mut self, state_str: &str) -> bool {
        if let Some(state_bytes) = base64_decode(state_str) {
            if state_bytes.is_empty() {
                return false;
            }

            let mut idx = 0;

            // Decode header byte
            let header_byte = state_bytes[idx];
            idx += 1;

            let board_size_code = (header_byte >> 2) & 0b111;
            let board_size = match board_size_code {
                0 => 9,
                1 => 13,
                2 => 19,
                _ => return false,
            };

            let player_code = header_byte & 0b11;
            let _current_player = match player_code {
                0 => StoneState::Empty,
                1 => StoneState::Black,
                2 => StoneState::White,
                _ => return false,
            };

            // Decode variable-length capture counts (for validation)
            if let Some((_black_captures, new_idx)) = decode_varint(&state_bytes, idx) {
                idx = new_idx;
                if let Some((_white_captures, new_idx)) = decode_varint(&state_bytes, idx) {
                    idx = new_idx;

                    // Decode move count
                    if let Some((move_count, new_idx)) = decode_varint(&state_bytes, idx) {
                        idx = new_idx;

                        // Decode move sequence
                        let mut move_sequence = Vec::new();
                        for _ in 0..move_count {
                            if idx + 1 >= state_bytes.len() {
                                return false;
                            }

                            let encoded = state_bytes[idx] as u16 | ((state_bytes[idx + 1] as u16) << 8);
                            idx += 2;

                            if encoded == 0xFFFF {
                                // Pass move
                                // Player alternates: Black starts, so odd moves are Black, even are White
                                let player = if move_sequence.len() % 2 == 0 {
                                    StoneState::Black
                                } else {
                                    StoneState::White
                                };
                                move_sequence.push(Move {
                                    x: None,
                                    y: None,
                                    player,
                                });
                            } else {
                                // Stone placement
                                let position = (encoded >> 2) as usize;
                                let player_bits = encoded & 0b11;
                                let player = match player_bits {
                                    1 => StoneState::Black,
                                    2 => StoneState::White,
                                    _ => return false,
                                };

                                let x = position % board_size;
                                let y = position / board_size;

                                if x >= board_size || y >= board_size {
                                    return false;
                                }

                                move_sequence.push(Move {
                                    x: Some(x),
                                    y: Some(y),
                                    player,
                                });
                            }
                        }

                        // Update game state
                        self.board_size = board_size;
                        self.move_sequence = move_sequence;
                        self.move_index = move_count as usize;

                        // Reconstruct the current game state
                        self.reconstruct_state_to_index(self.move_index);

                        console_log!("Successfully deserialized game state with {} moves", move_count);
                        return true;
                    }
                }
            }

            false
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
            self.move_numbers[cap_y][cap_x] = 0; // Clear move number when captured
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

    // Check if there are any stones on the board
    pub fn has_stones_on_board(&self) -> bool {
        for y in 0..self.board_size {
            for x in 0..self.board_size {
                if self.board[y][x] != StoneState::Empty {
                    return true;
                }
            }
        }
        false
    }

    // Directly set a board position for edit mode
    pub fn set_board_position(&mut self, x: usize, y: usize, state: u8) -> String {
        if x >= self.board_size || y >= self.board_size {
            return "Invalid position".to_string();
        }

        let stone_state = match state {
            0 => StoneState::Empty,
            1 => StoneState::Black,
            2 => StoneState::White,
            _ => return "Invalid state".to_string(),
        };

        self.board[y][x] = stone_state;

        // Clear move number when setting position in edit mode
        if stone_state == StoneState::Empty {
            self.move_numbers[y][x] = 0;
        }

        return "Position set successfully".to_string();
    }
}

// Variable-length integer encoding (LEB128-style)
// Uses 7 bits per byte for data, 1 bit to indicate continuation
fn encode_varint(bytes: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        bytes.push((value & 0x7F) as u8 | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

fn decode_varint(bytes: &[u8], mut idx: usize) -> Option<(u32, usize)> {
    let mut result = 0u32;
    let mut shift = 0;

    while idx < bytes.len() {
        let byte = bytes[idx];
        idx += 1;

        result |= ((byte & 0x7F) as u32) << shift;

        if byte & 0x80 == 0 {
            return Some((result, idx));
        }

        shift += 7;
        if shift >= 32 {
            return None; // Overflow
        }
    }

    None // Incomplete varint
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
