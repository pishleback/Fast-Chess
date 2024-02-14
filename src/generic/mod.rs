pub mod ai;
pub mod info;
pub mod signature;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub idx: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Team {
    White,
    Black,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl PieceKind {
    pub fn worth(&self) -> info::Score {
        match self {
            PieceKind::Pawn => info::Score::Finite(1),
            PieceKind::Rook => info::Score::Finite(5),
            PieceKind::Knight => info::Score::Finite(3),
            PieceKind::Bishop => info::Score::Finite(3),
            PieceKind::Queen => info::Score::Finite(9),
            PieceKind::King => info::Score::Finite(1000000000),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Piece {
    pub kind: PieceKind,
    pub team: Team,
    pub moved: bool,
}

#[derive(Debug, Clone)]
pub struct Board {
    turn: Team,
    signature: signature::Signature,
    white_pieces: HashMap<Square, Piece>,
    black_pieces: HashMap<Square, Piece>,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.turn == other.turn
            && self.signature.num() == other.signature.num()
            && self.white_pieces == other.white_pieces
            && self.black_pieces == other.black_pieces
    }
}

impl Board {
    pub fn new(
        turn: Team,
        signature: signature::Signature,
        white_piece_kinds: HashMap<Square, PieceKind>,
        black_piece_kinds: HashMap<Square, PieceKind>,
    ) -> Self {
        let mut white_pieces = HashMap::new();
        let mut black_pieces = HashMap::new();

        for (sq, kind) in white_piece_kinds {
            white_pieces.insert(
                sq,
                Piece {
                    kind,
                    team: Team::White,
                    moved: false,
                },
            );
        }
        for (sq, kind) in black_piece_kinds {
            black_pieces.insert(
                sq,
                Piece {
                    kind,
                    team: Team::Black,
                    moved: false,
                },
            );
        }

        let board = Self {
            turn,
            signature,
            white_pieces,
            black_pieces,
        };

        return board;
    }

    pub fn get_square(&self, sq: Square) -> Option<Piece> {
        match self.white_pieces.get(&sq) {
            Some(piece) => Some(*piece),
            None => match self.black_pieces.get(&sq) {
                Some(piece) => Some(*piece),
                None => None,
            },
        }
    }

    pub fn generate_info(&self) -> info::BoardInfo {
        info::BoardInfo::new(self)
    }

    pub fn get_turn(&self) -> Team {
        self.turn
    }

    pub fn get_pieces(&self) -> Vec<(Square, Piece)> {
        let mut pieces = vec![];
        for (sq, piece) in &self.white_pieces {
            pieces.push((*sq, piece.clone()));
        }
        for (sq, piece) in &self.black_pieces {
            pieces.push((*sq, piece.clone()));
        }
        pieces
    }

    fn good_pieces(&mut self) -> &mut HashMap<Square, Piece> {
        match self.turn {
            Team::White => &mut self.white_pieces,
            Team::Black => &mut self.black_pieces,
        }
    }

    fn bad_pieces(&mut self) -> &mut HashMap<Square, Piece> {
        match self.turn {
            Team::White => &mut self.black_pieces,
            Team::Black => &mut self.white_pieces,
        }
    }

    pub fn make_move(&mut self, m: info::Move) {
        match m {
            info::Move::Move {
                piece,
                from_sq,
                to_sq,
            } => {
                debug_assert_eq!(piece.team, self.turn);
                debug_assert_eq!(self.get_square(from_sq).unwrap(), piece);
                debug_assert!(self.get_square(to_sq).is_none());
                self.good_pieces().remove(&from_sq);
                self.good_pieces().insert(to_sq, piece);
            }
            info::Move::Capture {
                piece,
                victim,
                from_sq,
                to_sq,
            } => {
                debug_assert_eq!(piece.team, self.turn);
                debug_assert_ne!(victim.team, self.turn);
                debug_assert_eq!(self.get_square(from_sq).unwrap(), piece);
                debug_assert_eq!(self.get_square(to_sq).unwrap(), victim);
                self.good_pieces().remove(&from_sq);
                self.bad_pieces().remove(&to_sq);
                self.good_pieces().insert(to_sq, piece);
            }
        }

        self.turn = match self.turn {
            Team::White => Team::Black,
            Team::Black => Team::White,
        };
    }

    pub fn unmake_move(&mut self, m: info::Move) {
        self.turn = match self.turn {
            Team::White => Team::Black,
            Team::Black => Team::White,
        };

        match m {
            info::Move::Move {
                piece,
                from_sq,
                to_sq,
            } => {
                debug_assert_eq!(piece.team, self.turn);
                debug_assert_eq!(self.get_square(to_sq).unwrap(), piece);
                debug_assert!(self.get_square(from_sq).is_none());
                self.good_pieces().remove(&to_sq);
                self.good_pieces().insert(from_sq, piece);
            }
            info::Move::Capture {
                piece,
                victim,
                from_sq,
                to_sq,
            } => {
                debug_assert_eq!(piece.team, self.turn);
                debug_assert_ne!(victim.team, self.turn);
                debug_assert!(self.get_square(from_sq).is_none());
                debug_assert_eq!(self.get_square(to_sq).unwrap(), piece);
                self.good_pieces().remove(&to_sq);
                self.bad_pieces().insert(to_sq, victim);
                self.good_pieces().insert(from_sq, piece);
            }
        }
    }
}
