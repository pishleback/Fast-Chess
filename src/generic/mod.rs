pub mod ai;
pub mod board_data;
pub mod score;
pub mod signature;

use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub idx: usize,
}
impl PartialOrd for Square {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.idx.partial_cmp(&other.idx)
    }
}
impl Ord for Square {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.idx.cmp(&other.idx)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Team {
    White,
    Black,
}

impl Team {
    pub fn flip(&self) -> Self {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl PieceKind {
    pub fn worth(&self) -> Option<i64> {
        match self {
            PieceKind::Pawn => Some(1),
            PieceKind::Rook => Some(5),
            PieceKind::Knight => Some(3),
            PieceKind::Bishop => Some(3),
            PieceKind::Queen => Some(9),
            PieceKind::King => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Piece {
    pub kind: PieceKind,
    pub team: Team,
    pub moved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Standard {
        piece: Piece,
        victim: Option<Piece>,
        promotion: Option<PieceKind>,
        from_sq: Square,
        to_sq: Square,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct MoveIdx {
    pub idx: usize,
}

#[derive(Debug, Clone)]
pub struct Board {
    turn: Team,
    moves: Vec<Move>,
    signature: signature::Signature,
    white_pieces: BTreeMap<Square, Piece>,
    black_pieces: BTreeMap<Square, Piece>,
    white_king: Square,
    black_king: Square,
}
impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.turn == other.turn
            && self.white_pieces == other.white_pieces
            && self.black_pieces == other.black_pieces
    }
}
impl Eq for Board {}
impl core::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.turn.hash(state);
        self.white_pieces.hash(state);
        self.black_pieces.hash(state);
    }
}

impl Board {
    pub fn new(
        turn: Team,
        signature: signature::Signature,
        white_piece_kinds: HashMap<Square, PieceKind>,
        black_piece_kinds: HashMap<Square, PieceKind>,
    ) -> Self {
        let mut white_pieces = BTreeMap::new();
        let mut black_pieces = BTreeMap::new();

        let mut white_king = None;
        let mut black_king = None;

        for (sq, kind) in white_piece_kinds {
            white_pieces.insert(
                sq,
                Piece {
                    kind,
                    team: Team::White,
                    moved: false,
                },
            );
            if kind == PieceKind::King {
                assert!(white_king.is_none());
                white_king = Some(sq);
            }
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
            if kind == PieceKind::King {
                assert!(black_king.is_none());
                black_king = Some(sq);
            }
        }

        let board = Self {
            turn,
            moves: vec![],
            signature,
            white_pieces,
            black_pieces,
            white_king: white_king.unwrap(),
            black_king: black_king.unwrap(),
        };

        return board;
    }

    pub fn get_move_num(&self) -> usize {
        self.moves.len()
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

    // pub fn generate_info(&self) -> score::BoardInfo {
    //     score::BoardInfo::new(self)
    // }

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

    fn get_king_square(&self, team: Team) -> Square {
        match team {
            Team::White => self.white_king,
            Team::Black => self.black_king,
        }
    }

    fn good_pieces(&mut self) -> &mut BTreeMap<Square, Piece> {
        match self.turn {
            Team::White => &mut self.white_pieces,
            Team::Black => &mut self.black_pieces,
        }
    }

    fn bad_pieces(&mut self) -> &mut BTreeMap<Square, Piece> {
        match self.turn {
            Team::White => &mut self.black_pieces,
            Team::Black => &mut self.white_pieces,
        }
    }

    fn check(&self) {
        let mut white_king = None;
        for (sq, piece) in &self.white_pieces {
            if piece.kind == PieceKind::King {
                assert!(white_king.is_none());
                white_king = Some(sq);
            }
        }
        assert_eq!(white_king.unwrap(), &self.white_king);

        let mut black_king = None;
        for (sq, piece) in &self.black_pieces {
            if piece.kind == PieceKind::King {
                assert!(black_king.is_none());
                black_king = Some(sq);
            }
        }
        assert_eq!(black_king.unwrap(), &self.black_king);
    }

    pub fn make_move(&mut self, m: Move) {
        match m {
            Move::Standard {
                piece,
                victim: victim_opt,
                promotion: promotion_opt,
                from_sq,
                to_sq,
            } => {
                debug_assert_eq!(piece.team, self.turn);
                debug_assert_eq!(self.get_square(from_sq).unwrap(), piece);
                self.good_pieces().remove(&from_sq);
                match victim_opt {
                    Some(victim) => {
                        debug_assert_eq!(self.get_square(to_sq).unwrap(), victim);
                        debug_assert_ne!(victim.team, self.turn);
                        self.bad_pieces().remove(&to_sq);
                    }
                    None => {
                        debug_assert!(self.get_square(to_sq).is_none());
                    }
                }
                match promotion_opt {
                    Some(promotion) => {
                        self.good_pieces().insert(
                            to_sq,
                            Piece {
                                kind: promotion,
                                team: piece.team,
                                moved: true,
                            },
                        );
                    }
                    None => {
                        self.good_pieces().insert(to_sq, piece);
                    }
                }
                if piece.kind == PieceKind::King {
                    match piece.team {
                        Team::White => self.white_king = to_sq,
                        Team::Black => self.black_king = to_sq,
                    }
                }
            }
        }

        self.turn = self.turn.flip();
        self.moves.push(m);

        if cfg!(debug_assertions) {
            self.check();
        }
    }

    pub fn unmake_move(&mut self) -> Result<(), ()> {
        match self.moves.pop() {
            Some(m) => {
                self.turn = self.turn.flip();

                match m {
                    Move::Standard {
                        piece,
                        victim: victim_opt,
                        promotion: promotion_opt,
                        from_sq,
                        to_sq,
                    } => {
                        debug_assert_eq!(piece.team, self.turn);
                        debug_assert!(self.get_square(from_sq).is_none());
                        match promotion_opt {
                            Some(promotion) => {
                                debug_assert_eq!(
                                    self.get_square(to_sq).unwrap(),
                                    Piece {
                                        team: piece.team,
                                        moved: piece.moved,
                                        kind: promotion
                                    }
                                );
                            }
                            None => {
                                debug_assert_eq!(self.get_square(to_sq).unwrap(), piece);
                            }
                        }
                        self.good_pieces().remove(&to_sq);
                        match victim_opt {
                            Some(victim) => {
                                debug_assert_ne!(victim.team, self.turn);
                                self.bad_pieces().insert(to_sq, victim);
                            }
                            None => {}
                        }
                        self.good_pieces().insert(from_sq, piece);
                        if piece.kind == PieceKind::King {
                            match piece.team {
                                Team::White => self.white_king = from_sq,
                                Team::Black => self.black_king = from_sq,
                            }
                        }
                    }
                }

                if cfg!(debug_assertions) {
                    self.check();
                }

                Ok(())
            }
            None => Err(()),
        }
    }
}
