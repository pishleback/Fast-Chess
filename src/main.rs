pub mod generic;

mod classical {
    use crate::generic::*;
    use colored::Colorize;
    use std::collections::HashMap;

    fn idx_to_sq(idx: usize) -> (u8, u8) {
        assert!(idx < 64);
        (idx as u8 % 8, idx as u8 / 8)
    }

    fn sq_to_idx(x: u8, y: u8) -> usize {
        assert!(x < 8 && y < 8);
        return (x + y * 8) as usize;
    }

    pub fn make_signature() -> Signature {
        let opp = |i: usize, j: usize| -> Vec<usize> {
            let (xi, yi) = idx_to_sq(i);
            let (xj, yj) = idx_to_sq(j);
            let (xk, yk) = (2 * xj as i8 - xi as i8, 2 * yj as i8 - yi as i8);
            if 0 <= xk && xk < 8 && 0 <= yk && yk < 8 {
                return vec![sq_to_idx(xk as u8, yk as u8)];
            }
            return vec![];
        };

        let flat_nbs = |idx: usize| -> Vec<usize> {
            let mut nbs: Vec<usize> = vec![];
            let (x, y) = idx_to_sq(idx);
            for (dx, dy) in vec![(1i8, 0i8), (-1, 0), (0, 1), (0, -1)] {
                let (ax, ay) = ((x as i8) + dx, (y as i8) + dy);
                if 0 <= ax && ax < 8 && 0 <= ay && ay < 8 {
                    nbs.push(sq_to_idx(ax as u8, ay as u8));
                }
            }
            return nbs;
        };

        let diag_nbs = |idx: usize| -> Vec<usize> {
            let mut nbs: Vec<usize> = vec![];
            let (x, y) = idx_to_sq(idx);
            for (dx, dy) in vec![(1i8, 1i8), (1, -1), (-1, 1), (-1, -1)] {
                let (ax, ay) = ((x as i8) + dx, (y as i8) + dy);
                if 0 <= ax && ax < 8 && 0 <= ay && ay < 8 {
                    nbs.push(sq_to_idx(ax as u8, ay as u8));
                }
            }
            return nbs;
        };

        let pawn_moves = |team: Team, idx: usize| -> (Vec<usize>, Vec<usize>) {
            let (x, y) = idx_to_sq(idx);

            match (team, y) {
                (_, 0) => (vec![], vec![]),
                (_, 7) => (vec![], vec![]),
                (Team::White, y) => {
                    if y == 1 {
                        (vec![sq_to_idx(x, y + 1)], vec![sq_to_idx(x, y + 2)])
                    } else if y <= 6 {
                        (vec![sq_to_idx(x, y + 1)], vec![])
                    } else {
                        panic!()
                    }
                }
                (Team::Black, y) => {
                    if y == 6 {
                        (vec![sq_to_idx(x, y - 1)], vec![sq_to_idx(x, y - 2)])
                    } else if y >= 1 {
                        (vec![sq_to_idx(x, y - 1)], vec![])
                    } else {
                        panic!()
                    }
                }
            }
        };

        let starting_board: Board = {
            let mut squares: Vec<Square> = vec![Square::Unoccupied; 64];

            //white team
            for x in 0..8u8 {
                squares[sq_to_idx(x, 1)] =
                    Square::Occupied(Piece::new(Team::White, PieceKind::Pawn));
            }
            squares[sq_to_idx(0, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Rook));
            squares[sq_to_idx(1, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Knight));
            squares[sq_to_idx(2, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Bishop));
            squares[sq_to_idx(3, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Queen));
            squares[sq_to_idx(4, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::King));
            squares[sq_to_idx(5, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Bishop));
            squares[sq_to_idx(6, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Knight));
            squares[sq_to_idx(7, 0)] = Square::Occupied(Piece::new(Team::White, PieceKind::Rook));

            //black team
            for x in 0..8u8 {
                squares[sq_to_idx(x, 6)] =
                    Square::Occupied(Piece::new(Team::Black, PieceKind::Pawn));
            }
            squares[sq_to_idx(0, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Rook));
            squares[sq_to_idx(1, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Knight));
            squares[sq_to_idx(2, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Bishop));
            squares[sq_to_idx(3, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Queen));
            squares[sq_to_idx(4, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::King));
            squares[sq_to_idx(5, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Bishop));
            squares[sq_to_idx(6, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Knight));
            squares[sq_to_idx(7, 7)] = Square::Occupied(Piece::new(Team::Black, PieceKind::Rook));

            Board::new_from_squares(Team::White, squares)
        };

        Signature::new(
            64,
            &flat_nbs,
            &diag_nbs,
            &opp,
            &opp,
            &pawn_moves,
            starting_board,
        )
    }

    pub struct TerminalInterface {}

    impl TerminalInterface {
        pub fn new() -> TerminalInterface {
            Self {}
        }
    }

    impl Interface for TerminalInterface {
        fn show_board(&self, board: &Board, highlights: Vec<Highlight>) {
            let mut highlight_map = HashMap::new();
            for h in highlights {
                highlight_map.insert(h.idx, h);
            }

            println!();
            for y in 0..8u8 {
                print!("{}   ", 8 - y);
                for x in 0..8u8 {
                    let idx = sq_to_idx(x, 7 - y);
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
            for idx in 0..64usize {
                let (x, y) = idx_to_sq(idx);
                assert_eq!(sq_to_idx(x, y), idx);
            }

            for x in 0..8u8 {
                for y in 0..8u8 {
                    let idx = sq_to_idx(x, y);
                    assert_eq!(idx_to_sq(idx), (x, y));
                }
            }
        }
    }
}

fn main() {
    let interface = classical::TerminalInterface::new();

    generic::run(classical::make_signature(), &interface);
}
