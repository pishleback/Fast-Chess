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
pub enum EnCroissantable {
    No,
    Yes { move_num: usize, take_sq: Square },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum PieceKind {
    Pawn(EnCroissantable),
    Grasshopper,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

impl PieceKind {
    pub fn worth(&self) -> Option<i64> {
        match self {
            PieceKind::Pawn(..) => Some(2),
            PieceKind::Grasshopper => Some(1),
            PieceKind::Rook => Some(10),
            PieceKind::Knight => Some(6),
            PieceKind::Bishop => Some(6),
            PieceKind::Queen => Some(18),
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

impl Piece {
    fn moved(self) -> Self {
        Self {
            kind: self.kind,
            team: self.team,
            moved: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Move {
    Standard {
        from_piece: Piece,
        to_piece: Piece,
        victim: Option<Piece>,
        from_sq: Square,
        to_sq: Square,
    },
    Castle {
        king_from: Square,
        king_through: Vec<Square>,
        king_to: Square,
        king_piece: Piece,
        rook_from: Square,
        rook_to: Square,
        rook_piece: Piece,
    },
    EnCroissant {
        //I know...
        pawn: Piece,
        pawn_from: Square,
        pawn_to: Square,
        victim: Piece,
        victim_sq: Square,
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
        match &m {
            Move::Standard {
                from_piece,
                to_piece,
                victim: victim_opt,
                from_sq,
                to_sq,
            } => {
                debug_assert_eq!(from_piece.team, self.turn);
                debug_assert_eq!(to_piece.team, self.turn);
                debug_assert_eq!(self.get_square(*from_sq), Some(*from_piece));
                self.good_pieces().remove(&from_sq);
                match victim_opt {
                    Some(victim) => {
                        debug_assert_eq!(self.get_square(*to_sq), Some(*victim));
                        debug_assert_ne!(victim.team, self.turn);
                        self.bad_pieces().remove(&to_sq);
                    }
                    None => {
                        debug_assert!(self.get_square(*to_sq).is_none());
                    }
                }
                self.good_pieces().insert(*to_sq, *to_piece);
                if from_piece.kind == PieceKind::King {
                    debug_assert!(to_piece.kind == PieceKind::King);
                    match from_piece.team {
                        Team::White => self.white_king = *to_sq,
                        Team::Black => self.black_king = *to_sq,
                    }
                }
            }
            Move::Castle {
                king_from,
                king_through,
                king_to,
                king_piece,
                rook_from,
                rook_to,
                rook_piece,
            } => {
                debug_assert_eq!(self.get_square(*king_from), Some(*king_piece));
                for sq in king_through {
                    debug_assert_eq!(self.get_square(*sq), None);
                }
                debug_assert_eq!(self.get_square(*king_to), None);
                debug_assert_eq!(self.get_square(*rook_from), Some(*rook_piece));
                debug_assert_eq!(self.get_square(*rook_to), None);
                debug_assert_eq!(king_piece.team, self.turn);
                debug_assert_eq!(rook_piece.team, self.turn);

                self.good_pieces().remove(&king_from);
                self.good_pieces().remove(&rook_from);
                self.good_pieces().insert(*rook_to, rook_piece.moved());
                self.good_pieces().insert(*king_to, king_piece.moved());

                if king_piece.kind == PieceKind::King {
                    match king_piece.team {
                        Team::White => self.white_king = *king_to,
                        Team::Black => self.black_king = *king_to,
                    }
                }
            }
            Move::EnCroissant {
                pawn,
                pawn_from,
                pawn_to,
                victim,
                victim_sq,
            } => {
                debug_assert_eq!(self.get_square(*pawn_from), Some(*pawn));
                debug_assert_eq!(self.get_square(*pawn_to), None);
                debug_assert_eq!(self.get_square(*victim_sq), Some(*victim));
                self.good_pieces().remove(&pawn_from);
                self.good_pieces().insert(*pawn_to, pawn.moved());
                self.bad_pieces().remove(victim_sq);
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
                        from_piece,
                        to_piece,
                        victim: victim_opt,
                        from_sq,
                        to_sq,
                    } => {
                        debug_assert_eq!(from_piece.team, self.turn);
                        debug_assert_eq!(to_piece.team, self.turn);
                        debug_assert!(self.get_square(from_sq).is_none());
                        self.good_pieces().remove(&to_sq);
                        match victim_opt {
                            Some(victim) => {
                                debug_assert_ne!(victim.team, self.turn);
                                self.bad_pieces().insert(to_sq, victim);
                            }
                            None => {}
                        }
                        self.good_pieces().insert(from_sq, from_piece);
                        if from_piece.kind == PieceKind::King {
                            debug_assert!(to_piece.kind == PieceKind::King);
                            match from_piece.team {
                                Team::White => self.white_king = from_sq,
                                Team::Black => self.black_king = from_sq,
                            }
                        }
                    }
                    Move::Castle {
                        king_from,
                        king_through: _,
                        king_to,
                        king_piece,
                        rook_from,
                        rook_to,
                        rook_piece,
                    } => {
                        debug_assert_eq!(self.get_square(king_to), Some(king_piece.moved()));
                        debug_assert_eq!(self.get_square(king_from), None);

                        debug_assert_eq!(self.get_square(rook_to), Some(rook_piece.moved()));
                        debug_assert_eq!(self.get_square(rook_from), None);
                        debug_assert_eq!(king_piece.team, self.turn);
                        debug_assert_eq!(rook_piece.team, self.turn);

                        self.good_pieces().remove(&rook_to);
                        self.good_pieces().remove(&king_to);
                        self.good_pieces().insert(rook_from, rook_piece);
                        self.good_pieces().insert(king_from, king_piece);

                        if king_piece.kind == PieceKind::King {
                            match king_piece.team {
                                Team::White => self.white_king = king_from,
                                Team::Black => self.black_king = king_from,
                            }
                        }
                    }
                    Move::EnCroissant {
                        pawn,
                        pawn_from,
                        pawn_to,
                        victim,
                        victim_sq,
                    } => {
                        debug_assert_eq!(self.get_square(pawn_from), None);
                        debug_assert_eq!(self.get_square(pawn_to), Some(pawn.moved()));
                        debug_assert_eq!(self.get_square(victim_sq), None);
                        self.good_pieces().remove(&pawn_to);
                        self.good_pieces().insert(pawn_from, pawn);
                        self.bad_pieces().insert(victim_sq, victim);
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
