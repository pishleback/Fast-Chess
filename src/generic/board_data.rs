use super::ai::*;
use super::score::*;
use super::*;

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

#[derive(Debug, Clone)]
struct PseudoMoves {
    white_pseudomoves: Vec<Move>,
    black_pseudomoves: Vec<Move>,
    white_vision: Vec<Vec<Vision>>,
    black_vision: Vec<Vec<Vision>>,
}

impl PseudoMoves {
    fn get_pseudomoves(&self, team: Team) -> &Vec<Move> {
        match team {
            Team::White => &self.white_pseudomoves,
            Team::Black => &self.black_pseudomoves,
        }
    }

    fn get_vision(&self, team: Team, square: Square) -> &Vec<Vision> {
        match team {
            Team::White => &self.white_vision[square.idx],
            Team::Black => &self.black_vision[square.idx],
        }
    }

    fn new(board: &Board) -> Self {
        let mut white_vision: Vec<Vec<Vision>> =
            (0..board.signature.num()).map(|i| vec![]).collect();
        let mut black_vision: Vec<Vec<Vision>> =
            (0..board.signature.num()).map(|i| vec![]).collect();

        let mut white_pseudomoves = vec![];
        let mut black_pseudomoves = vec![];

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
                    Team::White => white_pseudomoves.push($m),
                    Team::Black => black_pseudomoves.push($m),
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
                                if blocking.kind != PieceKind::King {
                                    add_move!(
                                        Move::Capture {
                                            piece: *$piece,
                                            victim: blocking,
                                            from_sq: *$from_sq,
                                            to_sq: *to_sq,
                                        },
                                        $piece.team
                                    );
                                }
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
                                            if blocking.kind != PieceKind::King {
                                                add_move!(
                                                    Move::Capture {
                                                        piece: $piece,
                                                        victim: blocking,
                                                        from_sq: $from_sq,
                                                        to_sq: to_sq,
                                                    },
                                                    $piece.team
                                                );
                                            }
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
                                    if diag.kind != PieceKind::King {
                                        add_move!(
                                            Move::Capture {
                                                piece: *piece,
                                                victim: diag,
                                                from_sq: *from_sq,
                                                to_sq: *to_sq,
                                            },
                                            piece.team
                                        );
                                    }
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

        Self {
            white_pseudomoves,
            black_pseudomoves,
            white_vision,
            black_vision,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoardData {
    // info: BoardInfo,
    is_check: bool,
    evaluation: Score, //TODO: rename to evalutation
    moves: Vec<MoveData>,
}

impl BoardData {
    pub fn new(board: &Board) -> Self {
        let turn = board.get_turn();

        let pseudomoves = PseudoMoves::new(board);

        let is_check = pseudomoves
            .get_vision(turn.flip(), board.get_king_square(turn))
            .iter()
            .any(|vis| match vis.to {
                VisionTo::Visible => panic!(),
                VisionTo::Defend { piece } => panic!(),
                VisionTo::Attack { piece } => true,
            });

        let mut moves: Vec<Move> = vec![];
        for pseudo_move in match turn {
            Team::White => pseudomoves.white_pseudomoves,
            Team::Black => pseudomoves.black_pseudomoves,
        } {
            //check if the pseudo_move is legal

            // TODO: more efficient valid move algorithm
            // if cfg!(debug_assertions) {}

            let mut check = false;
            let mut test_board = board.clone();
            test_board.make_move(pseudo_move);
            let test_board_pseudomoves = PseudoMoves::new(&test_board);
            let king_square = test_board.get_king_square(turn);
            for vis in test_board_pseudomoves.get_vision(turn.flip(), king_square) {
                match &vis.to {
                    VisionTo::Visible => panic!(),
                    VisionTo::Defend { piece } => panic!(),

                    VisionTo::Attack { piece } => {
                        check = true;
                    }
                }
            }
            test_board.unmake_move(pseudo_move);

            if !check {
                moves.push(pseudo_move);
            }
        }

        macro_rules! signed_score {
            ($team:expr, $score:expr) => {
                match $team {
                    Team::White => $score,
                    Team::Black => -$score,
                }
            };
        }

        let score = {
            if moves.is_empty() {
                if is_check {
                    Score::Lost(board.move_num) //in check with no legal moves -> loose
                } else {
                    Score::Draw(board.move_num) //not in check with no legal moves -> draw
                }
            } else {
                let mut score = 0;
                for (sq, piece) in board.white_pieces.iter().chain(board.black_pieces.iter()) {
                    if piece.kind != PieceKind::King {
                        score += signed_score!(piece.team, piece.kind.worth().unwrap() * 1000);
                    }
                }
                for (sq_idx, visions, team) in pseudomoves
                    .white_vision
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i, v, Team::White))
                    .chain(
                        pseudomoves
                            .black_vision
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
                            VisionTo::Visible => {
                                score += signed_score!(team, 1);
                            }
                            VisionTo::Defend { piece } => {
                                if piece.kind != PieceKind::King
                                    && from_piece.kind != PieceKind::King
                                {
                                    let from_worth = from_piece.kind.worth().unwrap();
                                    let to_worth = piece.kind.worth().unwrap();
                                    score += signed_score!(team, (10 - from_worth) * 20);
                                }
                            }
                            VisionTo::Attack { piece } => {
                                if piece.kind != PieceKind::King
                                    && from_piece.kind != PieceKind::King
                                {
                                    let from_worth = from_piece.kind.worth().unwrap();
                                    let to_worth = piece.kind.worth().unwrap();
                                    score += signed_score!(team, (10 - from_worth) * 30);
                                }
                            }
                        }
                    }
                }
                Score::Heuristic(score)
            }
        };

        Self {
            moves: moves.into_iter().map(|m| MoveData::new(m)).collect(),
            is_check: is_check,
            evaluation: match board.get_turn() {
                Team::White => score,
                Team::Black => -score,
            },
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn get_evaluation(&self) -> Score {
        self.evaluation
    }

    pub fn get_moves_data(&self) -> &Vec<MoveData> {
        &self.moves
    }

    pub fn get_moves_data_mut(&mut self) -> &mut Vec<MoveData> {
        &mut self.moves
    }

    pub fn get_moves(&self) -> Vec<Move> {
        self.moves
            .iter()
            .map(|move_data| (move_data.get_move()))
            .collect()
    }

    pub fn get_move(&self, move_idx: MoveIdx) -> &MoveData {
        &self.moves[move_idx.idx]
    }

    pub fn get_move_mut(&mut self, move_idx: MoveIdx) -> &mut MoveData {
        &mut self.moves[move_idx.idx]
    }
}
