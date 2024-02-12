use colored::Colorize;

use super::super::generic;
use super::*;

pub struct Game {
    signature: generic::Signature,
    current_board: generic::Board,
}

impl Game {
    pub fn new() -> Self {
        let signature = super::create_signature();
        let current_board = signature.get_starting_board().clone();
        Self {
            signature,
            current_board,
        }
    }

    fn draw(&self) {
        println!();
        for y in 0..8u8 {
            print!("{}   ", 8 - y);
            for x in 0..8u8 {
                let idx = super::sq_to_idx(x, 7 - y);
                let sq = &self.current_board.at(idx);
                let sym_text = match sq {
                    generic::Square::Unoccupied => "▢ ",
                    generic::Square::Occupied(piece) => match piece.get_kind() {
                        generic::PieceKind::Pawn => "♙ ",
                        generic::PieceKind::Rook => "♜ ",
                        generic::PieceKind::Knight => "♞ ",
                        generic::PieceKind::Bishop => "♝ ",
                        generic::PieceKind::Queen => "♛ ",
                        generic::PieceKind::King => "♚ ",
                    },
                };
                let sym_coloured = match sq {
                    generic::Square::Unoccupied => sym_text.black(),
                    generic::Square::Occupied(piece) => match piece.get_team() {
                        generic::Team::White => sym_text.bright_blue(),
                        generic::Team::Black => sym_text.bright_red(),
                    },
                };
                // let sym_highlighted = match highlight_map.get(&idx) {
                //     Some(_) => sym_coloured.on_yellow(),
                //     None => sym_coloured,
                // };

                print!("{}", sym_coloured);
            }
            print!("\n");
        }
        println!();
        println!("    a b c d  e f g h");
        println!();
    }

    pub fn run(&mut self) {
        self.draw();
    }
}
