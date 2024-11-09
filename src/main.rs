use eframe::egui;
use rand::Rng;
use std::time::{Duration, Instant};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const BLOCK_SIZE: f32 = 30.0;
const TICK_DURATION: Duration = Duration::from_millis(500);

#[derive(Clone, Copy, PartialEq)]
enum BlockType {
    Empty,
    Filled,
}

#[derive(Clone)]
struct Tetromino {
    blocks: Vec<Vec<bool>>,
    x: i32,
    y: i32,
}

impl Tetromino {
    fn new() -> Self {
        let shapes = vec![
            // I
            vec![
                vec![true, true, true, true],
                vec![false, false, false, false],
            ],
            // O
            vec![
                vec![true, true],
                vec![true, true],
            ],
            // T
            vec![
                vec![false, true, false],
                vec![true, true, true],
            ],
            // L
            vec![
                vec![true, false, false],
                vec![true, true, true],
            ],
            // J
            vec![
                vec![false, false, true],
                vec![true, true, true],
            ],
            // S
            vec![
                vec![false, true, true],
                vec![true, true, false],
            ],
            // Z
            vec![
                vec![true, true, false],
                vec![false, true, true],
            ],
        ];

        let mut rng = rand::thread_rng();
        let shape = shapes[rng.gen_range(0..shapes.len())].clone();
        let width = shape[0].len() as i32;

        Tetromino {
            blocks: shape.clone(),
            x: (BOARD_WIDTH as i32 - width) / 2,
            y: 0,
        }
    }

    fn rotate(&mut self) {
        let rows = self.blocks.len();
        let cols = self.blocks[0].len();
        let mut rotated = vec![vec![false; rows]; cols];

        for i in 0..rows {
            for j in 0..cols {
                rotated[j][rows - 1 - i] = self.blocks[i][j];
            }
        }

        self.blocks = rotated;
    }
}

struct TetrisGame {
    board: Vec<Vec<BlockType>>,
    current_piece: Tetromino,
    last_update: Instant,
    game_over: bool,
    score: u32,
}

impl Default for TetrisGame {
    fn default() -> Self {
        Self {
            board: vec![vec![BlockType::Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            current_piece: Tetromino::new(),
            last_update: Instant::now(),
            game_over: false,
            score: 0,
        }
    }
}

impl TetrisGame {
    fn update(&mut self) {
        if self.game_over {
            return;
        }

        if !self.can_move(0, 1) {
            self.merge_piece();
            self.clear_lines();
            self.current_piece = Tetromino::new();
            if !self.can_move(0, 0) {
                self.game_over = true;
            }
            return;
        }

        self.current_piece.y += 1;
    }

    fn can_move(&self, dx: i32, dy: i32) -> bool {
        let new_x = self.current_piece.x + dx;
        let new_y = self.current_piece.y + dy;

        for (i, row) in self.current_piece.blocks.iter().enumerate() {
            for (j, &is_block) in row.iter().enumerate() {
                if !is_block {
                    continue;
                }

                let board_x = new_x + j as i32;
                let board_y = new_y + i as i32;

                if board_x < 0 || board_x >= BOARD_WIDTH as i32 ||
                   board_y >= BOARD_HEIGHT as i32 {
                    return false;
                }

                if board_y >= 0 && self.board[board_y as usize][board_x as usize] == BlockType::Filled {
                    return false;
                }
            }
        }
        true
    }

    fn merge_piece(&mut self) {
        for (i, row) in self.current_piece.blocks.iter().enumerate() {
            for (j, &is_block) in row.iter().enumerate() {
                if is_block {
                    let board_x = self.current_piece.x + j as i32;
                    let board_y = self.current_piece.y + i as i32;
                    if board_y >= 0 {
                        self.board[board_y as usize][board_x as usize] = BlockType::Filled;
                    }
                }
            }
        }
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;
        let mut y = BOARD_HEIGHT - 1;
        while y > 0 {
            if self.board[y].iter().all(|&block| block == BlockType::Filled) {
                self.board.remove(y);
                self.board.insert(0, vec![BlockType::Empty; BOARD_WIDTH]);
                lines_cleared += 1;
            } else {
                y -= 1;
            }
        }
        self.score += lines_cleared * 100;
    }

    fn move_piece(&mut self, dx: i32) {
        if self.can_move(dx, 0) {
            self.current_piece.x += dx;
        }
    }

    fn rotate_piece(&mut self) {
        let mut rotated = self.current_piece.clone();
        rotated.rotate();
        
        let mut valid = false;
        for test_x in -1..=1 {
            rotated.x = self.current_piece.x + test_x;
            if self.is_valid_position(&rotated) {
                valid = true;
                break;
            }
        }

        if valid {
            self.current_piece = rotated;
        }
    }

    fn is_valid_position(&self, piece: &Tetromino) -> bool {
        for (i, row) in piece.blocks.iter().enumerate() {
            for (j, &is_block) in row.iter().enumerate() {
                if !is_block {
                    continue;
                }

                let board_x = piece.x + j as i32;
                let board_y = piece.y + i as i32;

                if board_x < 0 || board_x >= BOARD_WIDTH as i32 ||
                   board_y >= BOARD_HEIGHT as i32 {
                    return false;
                }

                if board_y >= 0 && self.board[board_y as usize][board_x as usize] == BlockType::Filled {
                    return false;
                }
            }
        }
        true
    }

    fn hard_drop(&mut self) {
        while self.can_move(0, 1) {
            self.current_piece.y += 1;
        }
        self.update();
    }
}

#[derive(Default)]
pub struct TetrisApp {
    game: TetrisGame,
}

impl eframe::App for TetrisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.game.game_over && self.game.last_update.elapsed() >= TICK_DURATION {
            self.game.update();
            self.game.last_update = Instant::now();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.game.game_over {
                ui.centered_and_justified(|ui| {
                    ui.heading("Game Over!");
                    if ui.button("Restart").clicked() {
                        self.game = TetrisGame::default();
                    }
                });
                return;
            }

            ui.label(format!("Score: {}", self.game.score));

            if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                self.game.move_piece(-1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
                self.game.move_piece(1);
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                self.game.update();
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                self.game.rotate_piece();
            }
            if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                self.game.hard_drop();
            }

            let (response, painter) = ui.allocate_painter(
                egui::vec2(BOARD_WIDTH as f32 * BLOCK_SIZE, BOARD_HEIGHT as f32 * BLOCK_SIZE),
                egui::Sense::hover(),
            );

            let board_rect = response.rect;
            painter.rect_filled(board_rect, 0.0, egui::Color32::from_gray(20));

            for (y, row) in self.game.board.iter().enumerate() {
                for (x, block) in row.iter().enumerate() {
                    if *block == BlockType::Filled {
                        let block_rect = egui::Rect::from_min_size(
                            board_rect.min + egui::vec2(x as f32 * BLOCK_SIZE, y as f32 * BLOCK_SIZE),
                            egui::vec2(BLOCK_SIZE, BLOCK_SIZE),
                        );
                        painter.rect_filled(block_rect, 0.0, egui::Color32::BLUE);
                    }
                }
            }

            for (i, row) in self.game.current_piece.blocks.iter().enumerate() {
                for (j, &is_block) in row.iter().enumerate() {
                    if is_block {
                        let block_rect = egui::Rect::from_min_size(
                            board_rect.min + egui::vec2(
                                (self.game.current_piece.x + j as i32) as f32 * BLOCK_SIZE,
                                (self.game.current_piece.y + i as i32) as f32 * BLOCK_SIZE,
                            ),
                            egui::vec2(BLOCK_SIZE, BLOCK_SIZE),
                        );
                        painter.rect_filled(block_rect, 0.0, egui::Color32::RED);
                    }
                }
            }

            for x in 0..=BOARD_WIDTH {
                painter.line_segment(
                    [
                        board_rect.min + egui::vec2(x as f32 * BLOCK_SIZE, 0.0),
                        board_rect.min + egui::vec2(x as f32 * BLOCK_SIZE, board_rect.height()),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_gray(40)),
                );
            }
            for y in 0..=BOARD_HEIGHT {
                painter.line_segment(
                    [
                        board_rect.min + egui::vec2(0.0, y as f32 * BLOCK_SIZE),
                        board_rect.min + egui::vec2(board_rect.width(), y as f32 * BLOCK_SIZE),
                    ],
                    egui::Stroke::new(1.0, egui::Color32::from_gray(40)),
                );
            }
        });

        ctx.request_repaint();
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([
                BOARD_WIDTH as f32 * BLOCK_SIZE + 40.0,
                BOARD_HEIGHT as f32 * BLOCK_SIZE + 80.0,
            ]),
        ..Default::default()
    };

    eframe::run_native(
        "Tetris",
        options,
        Box::new(|_cc| Box::new(TetrisApp::default())),
    ).unwrap();
}