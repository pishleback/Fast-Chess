pub mod generic;

use std::collections::HashMap;

use generic::*;

struct ClassicalSignature {}

impl ClassicalSignature {
    fn idx_to_sq(&self, idx: usize) -> (u8, u8) {
        assert!(idx < 64);
        (idx as u8 % 8, idx as u8 / 8)
    }

    fn sq_to_idx(&self, x: u8, y: u8) -> usize {
        assert!(x < 8 && y < 8);
        return (x + y * 8) as usize;
    }
}

impl ClassicalSignature {
    fn opp(&self, i: usize, j: usize) -> Vec<usize> {
        let (xi, yi) = self.idx_to_sq(i);
        let (xj, yj) = self.idx_to_sq(j);
        let (xk, yk) = (2 * xj as i8 - xi as i8, 2 * yj as i8 - yi as i8);
        if 0 <= xk && xk < 8 && 0 <= yk && yk < 8 {
            return vec![self.sq_to_idx(xk as u8, yk as u8)];
        }
        return vec![];
    }
}

impl Signature for ClassicalSignature {
    fn num_sq(&self) -> usize {
        64
    }

    fn flat_nbs(&self, idx: usize) -> Vec<usize> {
        let mut nbs: Vec<usize> = vec![];
        let (x, y) = self.idx_to_sq(idx);
        for (dx, dy) in vec![(1i8, 0i8), (-1, 0), (0, 1), (0, -1)] {
            let (ax, ay) = ((x as i8) + dx, (y as i8) + dy);
            if 0 <= ax && ax < 8 && 0 <= ay && ay < 8 {
                nbs.push(self.sq_to_idx(ax as u8, ay as u8));
            }
        }
        return nbs;
    }

    fn diag_nbs(&self, idx: usize) -> Vec<usize> {
        let mut nbs: Vec<usize> = vec![];
        let (x, y) = self.idx_to_sq(idx);
        for (dx, dy) in vec![(1i8, 1i8), (1, -1), (-1, 1), (-1, -1)] {
            let (ax, ay) = ((x as i8) + dx, (y as i8) + dy);
            if 0 <= ax && ax < 8 && 0 <= ay && ay < 8 {
                nbs.push(self.sq_to_idx(ax as u8, ay as u8));
            }
        }
        return nbs;
    }

    fn flat_opp(&self, i: usize, j: usize) -> Vec<usize> {
        self.opp(i, j)
    }

    fn diag_opp(&self, i: usize, j: usize) -> Vec<usize> {
        self.opp(i, j)
    }

    fn pawn_moves(&self, team: Team, idx: usize) -> (Vec<usize>, Vec<usize>) {
        let (x, y) = self.idx_to_sq(idx);

        match (team, y) {
            (_, 0) => (vec![], vec![]),
            (_, 7) => (vec![], vec![]),
            (Team::White, y) => {
                if y == 1 {
                    (
                        vec![self.sq_to_idx(x, y + 1)],
                        vec![self.sq_to_idx(x, y + 2)],
                    )
                } else if y <= 6 {
                    (vec![self.sq_to_idx(x, y + 1)], vec![])
                } else {
                    panic!()
                }
            }
            (Team::Black, y) => {
                if y == 6 {
                    (
                        vec![self.sq_to_idx(x, y - 1)],
                        vec![self.sq_to_idx(x, y - 2)],
                    )
                } else if y >= 1 {
                    (vec![self.sq_to_idx(x, y - 1)], vec![])
                } else {
                    panic!()
                }
            }
        }
    }

    fn starting_board(&self) -> Board {
        let mut squares: Vec<Square> = vec![Square::Unoccupied; 64];

        //white team
        for x in 0..8u8 {
            squares[self.sq_to_idx(x, 1)] =
                Square::Occupied(Piece::new(Team::White, PieceKind::Pawn));
        }
        squares[self.sq_to_idx(0, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Rook));
        squares[self.sq_to_idx(1, 0)] =
            Square::Occupied(Piece::new(Team::White, PieceKind::Knight));
        squares[self.sq_to_idx(2, 0)] =
            Square::Occupied(Piece::new(Team::White, PieceKind::Bishop));
        squares[self.sq_to_idx(3, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Queen));
        squares[self.sq_to_idx(4, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::King));
        squares[self.sq_to_idx(5, 0)] =
            Square::Occupied(Piece::new(Team::White, PieceKind::Bishop));
        squares[self.sq_to_idx(6, 0)] =
            Square::Occupied(Piece::new(Team::White, PieceKind::Knight));
        squares[self.sq_to_idx(7, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Rook));

        //black team
        for x in 0..8u8 {
            squares[self.sq_to_idx(x, 6)] =
                Square::Occupied(Piece::new(Team::Black, PieceKind::Pawn));
        }
        squares[self.sq_to_idx(0, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Rook));
        squares[self.sq_to_idx(1, 7)] =
            Square::Occupied(Piece::new(Team::Black, PieceKind::Knight));
        squares[self.sq_to_idx(2, 7)] =
            Square::Occupied(Piece::new(Team::Black, PieceKind::Bishop));
        squares[self.sq_to_idx(3, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Queen));
        squares[self.sq_to_idx(4, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::King));
        squares[self.sq_to_idx(5, 7)] =
            Square::Occupied(Piece::new(Team::Black, PieceKind::Bishop));
        squares[self.sq_to_idx(6, 7)] =
            Square::Occupied(Piece::new(Team::Black, PieceKind::Knight));
        squares[self.sq_to_idx(7, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Rook));

        return Board::new(Team::White, squares);
    }
}

use colored::Colorize;

struct ClassicalTerminalInterface<'a> {
    signature: &'a ClassicalSignature,
}

impl<'a> Interface<'a, ClassicalSignature> for ClassicalTerminalInterface<'a> {
    fn new(signature: &'a ClassicalSignature) -> Self {
        ClassicalTerminalInterface { signature }
    }

    fn show_board(&self, board: &Board, highlights: Vec<Highlight>) {
        let mut highlight_map = HashMap::new();
        for h in highlights {
            highlight_map.insert(h.idx, h);
        }

        println!();
        for y in 0..8u8 {
            print!("{}   ", 8 - y);
            for x in 0..8u8 {
                let idx = self.signature.sq_to_idx(x, 7 - y);
                let sq = &board.at(idx);
                let sym_text = match sq {
                    Square::Unoccupied => "▢ ",
                    Square::Occupied(piece) => match piece.get_kind() {
                        PieceKind::Pawn => "♙ ",
                        PieceKind::Rook => "♜ ",
                        PieceKind::Knight => "♞ ",
                        PieceKind::Bishop => "♝ ",
                        PieceKind::Queen => "♛ ",
                        PieceKind::King => "♚ ",
                    },
                };
                let sym_coloured = match sq {
                    Square::Unoccupied => sym_text.black(),
                    Square::Occupied(piece) => match piece.get_team() {
                        Team::White => sym_text.bright_blue(),
                        Team::Black => sym_text.bright_red(),
                    },
                };
                let sym_highlighted = match highlight_map.get(&idx) {
                    Some(_) => sym_coloured.on_yellow(),
                    None => sym_coloured,
                };

                print!("{}", sym_highlighted);
            }
            print!("\n");
        }
        println!();
        println!("    a b c d  e f g h");
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let sig = ClassicalSignature {};

        for idx in 0..64usize {
            let (x, y) = sig.idx_to_sq(idx);
            assert_eq!(sig.sq_to_idx(x, y), idx);
        }

        for x in 0..8u8 {
            for y in 0..8u8 {
                let idx = sig.sq_to_idx(x, y);
                assert_eq!(sig.idx_to_sq(idx), (x, y));
            }
        }
    }
}

fn main() {
    let sig = ClassicalSignature {};
    let game: Game<_, ClassicalTerminalInterface> = Game::new(&sig);

    game.run();
}
