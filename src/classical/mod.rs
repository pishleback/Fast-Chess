use crate::generic::*;
use std::collections::HashMap;

use self::signature::CastleSignature;

// pub mod terminal;
pub mod graphical;

fn sq_to_grid(sq: Square) -> (u8, u8) {
    let idx = sq.idx;
    assert!(idx < 64);
    (idx as u8 % 8, idx as u8 / 8)
}

fn grid_to_sq(x: u8, y: u8) -> Square {
    assert!(x < 8 && y < 8);
    return Square {
        idx: (x + y * 8) as usize,
    };
}

pub fn create_signature() -> signature::Signature {
    let opp = |i: Square, j: Square| -> Vec<Square> {
        let (xi, yi) = sq_to_grid(i);
        let (xj, yj) = sq_to_grid(j);
        let (xk, yk) = (2 * xj as i8 - xi as i8, 2 * yj as i8 - yi as i8);
        if 0 <= xk && xk < 8 && 0 <= yk && yk < 8 {
            return vec![grid_to_sq(xk as u8, yk as u8)];
        }
        return vec![];
    };

    let flat_nbs = |idx: Square| -> Vec<Square> {
        let mut nbs: Vec<Square> = vec![];
        let (x, y) = sq_to_grid(idx);
        for (dx, dy) in vec![(1i8, 0i8), (-1, 0), (0, 1), (0, -1)] {
            let (ax, ay) = ((x as i8) + dx, (y as i8) + dy);
            if 0 <= ax && ax < 8 && 0 <= ay && ay < 8 {
                nbs.push(grid_to_sq(ax as u8, ay as u8));
            }
        }
        return nbs;
    };

    let diag_nbs = |idx: Square| -> Vec<Square> {
        let mut nbs: Vec<Square> = vec![];
        let (x, y) = sq_to_grid(idx);
        for (dx, dy) in vec![(1i8, 1i8), (1, -1), (-1, 1), (-1, -1)] {
            let (ax, ay) = ((x as i8) + dx, (y as i8) + dy);
            if 0 <= ax && ax < 8 && 0 <= ay && ay < 8 {
                nbs.push(grid_to_sq(ax as u8, ay as u8));
            }
        }
        return nbs;
    };

    let pawn_moves = |team: Team, idx: Square| -> Vec<(Square, Vec<Square>)> {
        let (x, y) = sq_to_grid(idx);

        match (team, y) {
            (_, 0) => vec![],
            (_, 7) => vec![],
            (Team::White, y) => {
                if y == 1 {
                    vec![(grid_to_sq(x, y + 1), vec![grid_to_sq(x, y + 2)])]
                } else if y <= 6 {
                    vec![(grid_to_sq(x, y + 1), vec![])]
                } else {
                    panic!()
                }
            }
            (Team::Black, y) => {
                if y == 6 {
                    vec![(grid_to_sq(x, y - 1), vec![grid_to_sq(x, y - 2)])]
                } else if y >= 1 {
                    vec![(grid_to_sq(x, y - 1), vec![])]
                } else {
                    panic!()
                }
            }
        }
    };

    signature::Signature::new(
        64,
        &flat_nbs,
        &diag_nbs,
        &opp,
        &opp,
        &pawn_moves,
        (0..8)
            .map(|x| {
                (
                    grid_to_sq(x, 7),
                    vec![
                        PieceKind::Knight,
                        PieceKind::Bishop,
                        PieceKind::Rook,
                        PieceKind::Queen,
                    ],
                )
            })
            .into_iter()
            .collect(),
        (0..8)
            .map(|x| {
                (
                    grid_to_sq(x, 0),
                    vec![
                        PieceKind::Knight,
                        PieceKind::Bishop,
                        PieceKind::Rook,
                        PieceKind::Queen,
                    ],
                )
            })
            .into_iter()
            .collect(),
        vec![
            CastleSignature {
                king_from: grid_to_sq(4, 0),
                king_to: grid_to_sq(2, 0),
                rook_from: grid_to_sq(0, 0),
                rook_to: grid_to_sq(3, 0),
                not_chcked: vec![grid_to_sq(3, 0)],
                not_occupied: vec![grid_to_sq(1, 0)],
            },
            CastleSignature {
                king_from: grid_to_sq(4, 0),
                king_to: grid_to_sq(6, 0),
                rook_from: grid_to_sq(7, 0),
                rook_to: grid_to_sq(5, 0),
                not_chcked: vec![grid_to_sq(5, 0)],
                not_occupied: vec![],
            },
        ],
        vec![
            CastleSignature {
                king_from: grid_to_sq(4, 7),
                king_to: grid_to_sq(2, 7),
                rook_from: grid_to_sq(0, 7),
                rook_to: grid_to_sq(3, 7),
                not_chcked: vec![grid_to_sq(3, 7)],
                not_occupied: vec![grid_to_sq(1, 7)],
            },
            CastleSignature {
                king_from: grid_to_sq(4, 7),
                king_to: grid_to_sq(6, 7),
                rook_from: grid_to_sq(7, 7),
                rook_to: grid_to_sq(5, 7),
                not_chcked: vec![grid_to_sq(5, 7)],
                not_occupied: vec![],
            },
        ],
    )
}

pub fn create_game() -> Board {
    //white team
    let mut white_pieces = HashMap::new();
    for x in 0..8u8 {
        white_pieces.insert(grid_to_sq(x, 1), PieceKind::Pawn);
    }
    white_pieces.insert(grid_to_sq(0, 0), PieceKind::Rook);
    white_pieces.insert(grid_to_sq(1, 0), PieceKind::Knight);
    white_pieces.insert(grid_to_sq(2, 0), PieceKind::Bishop);
    white_pieces.insert(grid_to_sq(3, 0), PieceKind::Queen);
    white_pieces.insert(grid_to_sq(4, 0), PieceKind::King);
    white_pieces.insert(grid_to_sq(5, 0), PieceKind::Bishop);
    white_pieces.insert(grid_to_sq(6, 0), PieceKind::Knight);
    white_pieces.insert(grid_to_sq(7, 0), PieceKind::Rook);

    //black team
    let mut black_pieces = HashMap::new();
    for x in 0..8u8 {
        black_pieces.insert(grid_to_sq(x, 6), PieceKind::Pawn);
    }
    black_pieces.insert(grid_to_sq(0, 7), PieceKind::Rook);
    black_pieces.insert(grid_to_sq(1, 7), PieceKind::Knight);
    black_pieces.insert(grid_to_sq(2, 7), PieceKind::Bishop);
    black_pieces.insert(grid_to_sq(3, 7), PieceKind::Queen);
    black_pieces.insert(grid_to_sq(4, 7), PieceKind::King);
    black_pieces.insert(grid_to_sq(5, 7), PieceKind::Bishop);
    black_pieces.insert(grid_to_sq(6, 7), PieceKind::Knight);
    black_pieces.insert(grid_to_sq(7, 7), PieceKind::Rook);

    Board::new(Team::White, create_signature(), white_pieces, black_pieces)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        for idx in 0..64usize {
            let sq = Square { idx };
            let (x, y) = sq_to_grid(sq);
            assert_eq!(grid_to_sq(x, y), sq);
        }

        for x in 0..8u8 {
            for y in 0..8u8 {
                let sq = grid_to_sq(x, y);
                assert_eq!(sq_to_grid(sq), (x, y));
            }
        }
    }
}
