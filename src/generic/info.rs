use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Finite(i64),
    PosInf,
    NegInf,
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Score::NegInf, _) => std::cmp::Ordering::Less,
            (Score::PosInf, _) => std::cmp::Ordering::Greater,
            (_, Score::NegInf) => std::cmp::Ordering::Greater,
            (_, Score::PosInf) => std::cmp::Ordering::Less,
            (Score::Finite(a), Score::Finite(b)) => a.cmp(b),
        }
    }
}
impl AddAssign for Score {
    fn add_assign(mut self: &mut Self, rhs: Self) {
        match (&mut self, rhs) {
            (Score::Finite(a), Score::Finite(b)) => a.add_assign(b),
            (Score::Finite(a), Score::PosInf) => *self = Score::PosInf,
            (Score::Finite(a), Score::NegInf) => *self = Score::NegInf,
            (Score::PosInf, Score::Finite(b)) => {}
            (Score::PosInf, Score::PosInf) => {}
            (Score::PosInf, Score::NegInf) => panic!(),
            (Score::NegInf, Score::Finite(b)) => {}
            (Score::NegInf, Score::PosInf) => panic!(),
            (Score::NegInf, Score::NegInf) => {}
        }
    }
}
impl SubAssign for Score {
    fn sub_assign(mut self: &mut Self, rhs: Self) {
        match (&mut self, rhs) {
            (Score::Finite(a), Score::Finite(b)) => a.sub_assign(b),
            (Score::Finite(a), Score::PosInf) => *self = Score::NegInf,
            (Score::Finite(a), Score::NegInf) => *self = Score::PosInf,
            (Score::PosInf, Score::Finite(b)) => {}
            (Score::PosInf, Score::PosInf) => panic!(),
            (Score::PosInf, Score::NegInf) => {}
            (Score::NegInf, Score::Finite(b)) => {}
            (Score::NegInf, Score::PosInf) => {}
            (Score::NegInf, Score::NegInf) => panic!(),
        }
    }
}
impl Neg for Score {
    type Output = Score;

    fn neg(self) -> Self::Output {
        match self {
            Score::Finite(a) => Score::Finite(-a),
            Score::PosInf => Score::NegInf,
            Score::NegInf => Score::PosInf,
        }
    }
}
impl Add<Score> for Score {
    type Output = Score;

    fn add(self, rhs: Score) -> Self::Output {
        match (self, rhs) {
            (Score::Finite(a), Score::Finite(b)) => Score::Finite(a + b),
            (Score::Finite(a), Score::PosInf) => Score::PosInf,
            (Score::Finite(a), Score::NegInf) => Score::NegInf,
            (Score::PosInf, Score::Finite(b)) => Score::PosInf,
            (Score::PosInf, Score::PosInf) => Score::PosInf,
            (Score::PosInf, Score::NegInf) => panic!(),
            (Score::NegInf, Score::Finite(b)) => Score::NegInf,
            (Score::NegInf, Score::PosInf) => panic!(),
            (Score::NegInf, Score::NegInf) => Score::NegInf,
        }
    }
}
impl Sub<Score> for Score {
    type Output = Score;

    fn sub(self, rhs: Score) -> Self::Output {
        match (self, rhs) {
            (Score::Finite(a), Score::Finite(b)) => Score::Finite(a - b),
            (Score::Finite(a), Score::PosInf) => Score::NegInf,
            (Score::Finite(a), Score::NegInf) => Score::PosInf,
            (Score::PosInf, Score::Finite(b)) => Score::PosInf,
            (Score::PosInf, Score::PosInf) => panic!(),
            (Score::PosInf, Score::NegInf) => Score::PosInf,
            (Score::NegInf, Score::Finite(b)) => Score::NegInf,
            (Score::NegInf, Score::PosInf) => Score::NegInf,
            (Score::NegInf, Score::NegInf) => panic!(),
        }
    }
}
impl Mul<i64> for Score {
    type Output = Score;

    fn mul(self, rhs: i64) -> Self::Output {
        assert!(rhs > 0);
        match self {
            Score::Finite(a) => match a.checked_mul(rhs) {
                Some(ans) => Score::Finite(ans),
                None => match a.cmp(&0) {
                    std::cmp::Ordering::Less => Score::NegInf,
                    std::cmp::Ordering::Equal => Score::Finite(0),
                    std::cmp::Ordering::Greater => Score::PosInf,
                },
            },
            Score::PosInf => Score::PosInf,
            Score::NegInf => Score::NegInf,
        }
    }
}

#[derive(Debug, Clone)]
enum VisionFrom {
    Teleport {
        piece: Piece,
        from: Square,
    },
    Slide {
        piece: Piece,
        slide: Vec<Square>,
        slide_idx: usize,
    },
}
#[derive(Debug, Clone)]
enum VisionTo {
    Visible,
    Defend { piece: Piece },
    Attack { piece: Piece },
}
#[derive(Debug, Clone)]
struct Vision {
    from: VisionFrom,
    to: VisionTo,
}

#[derive(Debug, Clone, Copy)]
pub enum Move {
    Move {
        piece: Piece,
        from_sq: Square,
        to_sq: Square,
    },
    Capture {
        piece: Piece,
        victim: Piece,
        from_sq: Square,
        to_sq: Square,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct MoveIdx {
    pub idx: usize,
}

#[derive(Debug, Clone)]
pub struct BoardInfo {
    white_moves: Vec<Move>,
    black_moves: Vec<Move>,
    white_vision: Vec<Vec<Vision>>, //Square -> Vision
    black_vision: Vec<Vec<Vision>>, //Square -> Vision
    score: Score,                   //+is good for white, -is good for black
}

impl BoardInfo {
    pub fn new(board: &Board) -> Self {
        let mut white_vision: Vec<Vec<Vision>> =
            (0..board.signature.num()).map(|i| vec![]).collect();
        let mut black_vision: Vec<Vec<Vision>> =
            (0..board.signature.num()).map(|i| vec![]).collect();

        let mut white_moves = vec![];
        let mut black_moves = vec![];

        macro_rules! add_vision {
            ($s:expr, $v:expr, $t:expr) => {
                match $t {
                    Team::White => white_vision[$s.idx].push($v),
                    Team::Black => black_vision[$s.idx].push($v),
                }
            };
        }

        macro_rules! add_move {
            ($m:expr, $t:expr) => {
                match $t {
                    Team::White => white_moves.push($m),
                    Team::Black => black_moves.push($m),
                }
            };
        }

        macro_rules! add_teleports {
            ($teles:expr, $piece:expr, $from_sq:expr) => {{
                for to_sq in $teles {
                    match board.get_square(*to_sq) {
                        None => {
                            add_vision!(
                                to_sq,
                                Vision {
                                    from: VisionFrom::Teleport {
                                        piece: *$piece,
                                        from: *$from_sq,
                                    },
                                    to: VisionTo::Visible,
                                },
                                $piece.team
                            );
                            add_move!(
                                Move::Move {
                                    piece: *$piece,
                                    from_sq: *$from_sq,
                                    to_sq: *to_sq,
                                },
                                $piece.team
                            );
                        }
                        Some(blocking) => {
                            if blocking.team != $piece.team {
                                add_vision!(
                                    to_sq,
                                    Vision {
                                        from: VisionFrom::Teleport {
                                            piece: *$piece,
                                            from: *$from_sq,
                                        },
                                        to: VisionTo::Attack { piece: blocking },
                                    },
                                    $piece.team
                                );
                                add_move!(
                                    Move::Capture {
                                        piece: *$piece,
                                        victim: blocking,
                                        from_sq: *$from_sq,
                                        to_sq: *to_sq,
                                    },
                                    $piece.team
                                );
                            } else {
                                add_vision!(
                                    to_sq,
                                    Vision {
                                        from: VisionFrom::Teleport {
                                            piece: *$piece,
                                            from: *$from_sq,
                                        },
                                        to: VisionTo::Defend { piece: blocking },
                                    },
                                    $piece.team
                                );
                            }
                        }
                    }
                }
            }};
        }

        macro_rules! add_slides {
            ($all_slides:expr, $piece:expr, $from_sq:expr) => {{
                let mut done = HashSet::new();
                for slides in $all_slides {
                    for slide in slides {
                        let mut slide_idx = 0;
                        while slide_idx < slide.len() {
                            let to_sq = slide[slide_idx];
                            match board.get_square(to_sq) {
                                None => {
                                    if !done.contains(&to_sq) {
                                        done.insert(to_sq);
                                        add_vision!(
                                            to_sq,
                                            Vision {
                                                from: VisionFrom::Slide {
                                                    piece: $piece,
                                                    slide: slide.clone(),
                                                    slide_idx: slide_idx,
                                                },
                                                to: VisionTo::Visible,
                                            },
                                            $piece.team
                                        );
                                        add_move!(
                                            Move::Move {
                                                piece: $piece,
                                                from_sq: $from_sq,
                                                to_sq: to_sq,
                                            },
                                            $piece.team
                                        );
                                    }
                                }
                                Some(blocking) => {
                                    if !done.contains(&to_sq) {
                                        done.insert(to_sq);
                                        if blocking.team != $piece.team {
                                            add_vision!(
                                                to_sq,
                                                Vision {
                                                    from: VisionFrom::Slide {
                                                        piece: $piece,
                                                        slide: slide,
                                                        slide_idx: slide_idx,
                                                    },
                                                    to: VisionTo::Attack { piece: blocking },
                                                },
                                                $piece.team
                                            );
                                            add_move!(
                                                Move::Capture {
                                                    piece: $piece,
                                                    victim: blocking,
                                                    from_sq: $from_sq,
                                                    to_sq: to_sq,
                                                },
                                                $piece.team
                                            );
                                        } else {
                                            add_vision!(
                                                to_sq,
                                                Vision {
                                                    from: VisionFrom::Slide {
                                                        piece: $piece,
                                                        slide: slide,
                                                        slide_idx: slide_idx,
                                                    },
                                                    to: VisionTo::Defend { piece: blocking },
                                                },
                                                $piece.team
                                            );
                                        }
                                    }
                                    break;
                                }
                            };
                            slide_idx += 1;
                        }
                    }
                }
            }};
        }

        //sort the pieces so that two equal boards produce moves in the same order
        let mut all_pieces: Vec<_> = board
            .white_pieces
            .iter()
            .chain(board.black_pieces.iter())
            .collect();
        all_pieces.sort_by_key(|(s, p)| s.idx);
        for (from_sq, piece) in all_pieces {
            match piece.kind {
                PieceKind::Pawn => {
                    //pawn movement
                    for pm in board.signature.get_pawn_moves(*from_sq, piece.team) {
                        let (first, seconds) = pm;
                        if board.get_square(*first).is_none() {
                            add_move!(
                                Move::Move {
                                    piece: *piece,
                                    from_sq: *from_sq,
                                    to_sq: *first,
                                },
                                piece.team
                            );
                            for second in seconds {
                                if board.get_square(*second).is_none() {
                                    add_move!(
                                        Move::Move {
                                            piece: *piece,
                                            from_sq: *from_sq,
                                            to_sq: *second,
                                        },
                                        piece.team
                                    );
                                }
                            }
                        }
                    }
                    //pawn attacks
                    for to_sq in board.signature.get_pawn_takes(*from_sq, piece.team) {
                        match board.get_square(*to_sq) {
                            None => {
                                add_vision!(
                                    to_sq,
                                    Vision {
                                        from: VisionFrom::Teleport {
                                            piece: *piece,
                                            from: *from_sq,
                                        },
                                        to: VisionTo::Visible,
                                    },
                                    piece.team
                                );
                            }
                            Some(diag) => {
                                if diag.team != piece.team {
                                    add_vision!(
                                        to_sq,
                                        Vision {
                                            from: VisionFrom::Teleport {
                                                piece: *piece,
                                                from: *from_sq,
                                            },
                                            to: VisionTo::Attack { piece: diag },
                                        },
                                        piece.team
                                    );
                                    add_move!(
                                        Move::Capture {
                                            piece: *piece,
                                            victim: diag,
                                            from_sq: *from_sq,
                                            to_sq: *to_sq,
                                        },
                                        piece.team
                                    );
                                } else {
                                    add_vision!(
                                        to_sq,
                                        Vision {
                                            from: VisionFrom::Teleport {
                                                piece: *piece,
                                                from: *from_sq,
                                            },
                                            to: VisionTo::Defend { piece: diag },
                                        },
                                        piece.team
                                    );
                                }
                            }
                        }
                    }
                }
                PieceKind::Rook => {
                    add_slides!(
                        vec![board.signature.get_flat_slides(*from_sq).clone()],
                        *piece,
                        *from_sq
                    );
                }
                PieceKind::Bishop => {
                    add_slides!(
                        vec![board.signature.get_diag_slides(*from_sq).clone()],
                        *piece,
                        *from_sq
                    );
                }
                PieceKind::Queen => {
                    add_slides!(
                        vec![
                            board.signature.get_flat_slides(*from_sq).clone(),
                            board.signature.get_diag_slides(*from_sq).clone()
                        ],
                        *piece,
                        *from_sq
                    );
                }
                PieceKind::Knight => {
                    add_teleports!(board.signature.get_knight_moves(*from_sq), piece, from_sq);
                }
                PieceKind::King => {
                    add_teleports!(board.signature.get_king_moves(*from_sq), piece, from_sq);
                }
            }
        }

        let mut score = Score::Finite(0);
        for (sq, piece) in board.white_pieces.iter().chain(board.black_pieces.iter()) {
            match piece.team {
                Team::White => {
                    score += piece.kind.worth() * 10000;
                }
                Team::Black => {
                    score -= piece.kind.worth() * 10000;
                }
            }
        }

        for (sq_idx, visions, team) in white_vision
            .iter()
            .enumerate()
            .map(|(i, v)| (i, v, Team::White))
            .chain(
                black_vision
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i, v, Team::Black)),
            )
        {
            let sq = Square { idx: sq_idx };
            for vision in visions {
                let from_piece;
                match &vision.from {
                    VisionFrom::Teleport { piece, from } => {
                        from_piece = piece;
                    }
                    VisionFrom::Slide {
                        piece,
                        slide,
                        slide_idx,
                    } => {
                        from_piece = piece;
                    }
                }
                match &vision.to {
                    VisionTo::Visible => match team {
                        Team::White => {
                            score += Score::Finite(1);
                        }
                        Team::Black => {
                            score -= Score::Finite(1);
                        }
                    },
                    VisionTo::Defend { piece } => {
                        if piece.kind != PieceKind::King && from_piece.kind != PieceKind::King {
                            let from_worth = from_piece.kind.worth();
                            let to_worth = piece.kind.worth();
                            match team {
                                Team::White => {
                                    score += Score::Finite(10);
                                }
                                Team::Black => {
                                    score -= Score::Finite(10);
                                }
                            }
                            if from_worth < to_worth {
                                match team {
                                    Team::White => {
                                        score += (to_worth - from_worth) * 20;
                                    }
                                    Team::Black => {
                                        score -= (to_worth - from_worth) * 20;
                                    }
                                }
                            };
                        }
                    }
                    VisionTo::Attack { piece } => {
                        if piece.kind != PieceKind::King && from_piece.kind != PieceKind::King {
                            let from_worth = from_piece.kind.worth();
                            let to_worth = piece.kind.worth();
                            match team {
                                Team::White => {
                                    score += Score::Finite(20);
                                }
                                Team::Black => {
                                    score -= Score::Finite(20);
                                }
                            }
                            if from_worth < to_worth {
                                match team {
                                    Team::White => {
                                        score += (to_worth - from_worth) * 30;
                                    }
                                    Team::Black => {
                                        score -= (to_worth - from_worth) * 30;
                                    }
                                }
                            };
                        }
                    }
                }
            }
        }

        Self {
            white_moves,
            black_moves,
            white_vision,
            black_vision,
            score,
        }
    }

    pub fn get_moves(&self, team: Team) -> &Vec<Move> {
        match team {
            Team::White => &self.white_moves,
            Team::Black => &self.black_moves,
        }
    }

    pub fn best_move(&mut self, board: &mut Board) -> Option<MoveIdx> {
        let team = board.get_turn();
        let mut alpha = Score::NegInf;
        let mut best_m_idx = None;
        let n = self.get_moves(team).len();
        for m_idx in 0..n {
            let m = self.get_moves(team)[m_idx];
            // let og_board = self.board.clone();
            board.make_move(m);
            let score = -board
                .generate_info()
                .alpha_beta_score(board, 2, Score::NegInf, -alpha);
            board.unmake_move(m);
            // debug_assert_eq!(self.board, &og_board);
            println!("{:?} / {:?} = {:?}", m_idx + 1, n, score);
            if score > alpha {
                alpha = score;
                best_m_idx = Some(MoveIdx { idx: m_idx });
            }
        }
        best_m_idx
    }

    pub fn raw_score(&self) -> Score {
        self.score
    }

    pub fn heuristic_score(&self, board: &mut Board) -> Score {
        match board.get_turn() {
            Team::White => self.score,
            Team::Black => -self.score,
        }
    }

    pub fn alpha_beta_score(
        &mut self,
        board: &mut Board,
        depth: usize,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        //alpha is the score we already know we can achive
        //beta is the score the opponent knows they can achive
        if depth == 0 {
            self.quiesce_score(board, alpha, beta, 0)
        } else {
            let team = board.get_turn();
            let mut bestscore = Score::NegInf;
            let n = self.get_moves(team).len();
            for m_idx in 0..n {
                let m = self.get_moves(team)[m_idx];
                // let og_board = self.board.clone();
                board.make_move(m);
                let score =
                    -board
                        .generate_info()
                        .alpha_beta_score(board, depth - 1, -beta, -alpha);
                board.unmake_move(m);
                // debug_assert_eq!(self.board, &og_board);
                if score > bestscore {
                    bestscore = score;
                    if score > alpha {
                        alpha = score;
                    }
                }
                if score >= beta {
                    break; //the opponent already has a method to avoid us getting this score here, so we can stop looking
                }
            }
            bestscore
        }
    }

    pub fn quiesce_score(
        &mut self,
        board: &mut Board,
        mut alpha: Score,
        beta: Score,
        deep: usize,
    ) -> Score {
        let team = board.get_turn();
        let mut bestscore = self.heuristic_score(board); //assume we can do at least this well by the null-move observation
        if deep >= 10 {
            return bestscore;
        }
        if bestscore >= beta {
            return bestscore;
        }
        if bestscore >= alpha {
            alpha = bestscore;
        }

        let n = self.get_moves(team).len();
        for m_idx in 0..n {
            let m = self.get_moves(team)[m_idx];
            if match &m {
                Move::Move { .. } => false,
                Move::Capture { .. } => true,
            } {
                // let og_board = self.board.clone();
                board.make_move(m);
                let score = -board
                    .generate_info()
                    .quiesce_score(board, -beta, -alpha, deep + 1);
                board.unmake_move(m);
                // debug_assert_eq!(self.board, &og_board);
                if score > bestscore {
                    bestscore = score;
                    if score > alpha {
                        alpha = score;
                    }
                }
                if score >= beta {
                    break; //the opponent already has a method to avoid us getting this score here, so we can stop looking
                }
            }
        }
        bestscore
    }
}
