use super::ai::*;
use super::score::*;
use super::*;

#[derive(Debug, Clone)]
enum Vision {
    Teleport {
        piece: Piece,
        from: Square,
    },
    Slide {
        piece: Piece,
        from: Square,
        slide: Vec<Square>,
        slide_idx: usize,
    },
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
            (0..board.signature.num()).map(|_i| vec![]).collect();
        let mut black_vision: Vec<Vec<Vision>> =
            (0..board.signature.num()).map(|_i| vec![]).collect();

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

        macro_rules! add_pawn_move {
            ($piece:expr, $victim:expr, $from_sq:expr, $to_sq:expr) => {
                match board.signature.get_pawn_promotions($to_sq, $piece.team) {
                    None => {
                        add_move!(
                            Move::Standard {
                                piece: $piece,
                                victim: $victim,
                                promotion: None,
                                from_sq: $from_sq,
                                to_sq: $to_sq
                            },
                            $piece.team
                        )
                    }
                    Some(promotions) => {
                        for kind in promotions {
                            add_move!(
                                Move::Standard {
                                    piece: $piece,
                                    victim: $victim,
                                    promotion: Some(*kind),
                                    from_sq: $from_sq,
                                    to_sq: $to_sq
                                },
                                $piece.team
                            )
                        }
                    }
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
                                Vision::Teleport {
                                    piece: *$piece,
                                    from: *$from_sq,
                                },
                                $piece.team
                            );
                            add_move!(
                                Move::Standard {
                                    piece: *$piece,
                                    victim: None,
                                    promotion: None,
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
                                    Vision::Teleport {
                                        piece: *$piece,
                                        from: *$from_sq,
                                    },
                                    $piece.team
                                );
                                if blocking.kind != PieceKind::King {
                                    add_move!(
                                        Move::Standard {
                                            piece: *$piece,
                                            victim: Some(blocking),
                                            promotion: None,
                                            from_sq: *$from_sq,
                                            to_sq: *to_sq,
                                        },
                                        $piece.team
                                    );
                                }
                            } else {
                                add_vision!(
                                    to_sq,
                                    Vision::Teleport {
                                        piece: *$piece,
                                        from: *$from_sq,
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
                                            Vision::Slide {
                                                piece: $piece,
                                                from: $from_sq,
                                                slide: slide.clone(),
                                                slide_idx: slide_idx,
                                            },
                                            $piece.team
                                        );
                                        add_move!(
                                            Move::Standard {
                                                piece: $piece,
                                                victim: None,
                                                promotion: None,
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
                                                Vision::Slide {
                                                    piece: $piece,
                                                    from: $from_sq,
                                                    slide: slide,
                                                    slide_idx: slide_idx,
                                                },
                                                $piece.team
                                            );
                                            if blocking.kind != PieceKind::King {
                                                add_move!(
                                                    Move::Standard {
                                                        piece: $piece,
                                                        victim: Some(blocking),
                                                        promotion: None,
                                                        from_sq: $from_sq,
                                                        to_sq: to_sq,
                                                    },
                                                    $piece.team
                                                );
                                            }
                                        } else {
                                            add_vision!(
                                                to_sq,
                                                Vision::Slide {
                                                    piece: $piece,
                                                    from: $from_sq,
                                                    slide: slide,
                                                    slide_idx: slide_idx,
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
                            add_pawn_move!(*piece, None, *from_sq, *first);
                            for second in seconds {
                                if board.get_square(*second).is_none() {
                                    add_pawn_move!(*piece, None, *from_sq, *second);
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
                                    Vision::Teleport {
                                        piece: *piece,
                                        from: *from_sq,
                                    },
                                    piece.team
                                );
                            }
                            Some(diag) => {
                                if diag.team != piece.team {
                                    add_vision!(
                                        to_sq,
                                        Vision::Teleport {
                                            piece: *piece,
                                            from: *from_sq,
                                        },
                                        piece.team
                                    );
                                    if diag.kind != PieceKind::King {
                                        add_pawn_move!(*piece, Some(diag), *from_sq, *to_sq);
                                    }
                                } else {
                                    add_vision!(
                                        to_sq,
                                        Vision::Teleport {
                                            piece: *piece,
                                            from: *from_sq,
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
    pub fn new(board: &mut Board) -> Self {
        let turn = board.get_turn();

        let pseudomoves = PseudoMoves::new(board);

        let checkers = pseudomoves.get_vision(turn.flip(), board.get_king_square(turn));
        let is_check = !checkers.is_empty();

        let is_illegal = |board: &mut Board, pseudo_move: Move| -> bool {
            //pseudo_move is legal iff our king is not under attack after making pseudo_move
            match pseudo_move {
                Move::Standard {
                    piece,
                    victim: victim_opt,
                    promotion,
                    from_sq,
                    to_sq,
                } => {
                    if piece.kind == PieceKind::King {
                        let to_visions = pseudomoves.get_vision(turn.flip(), to_sq);
                        if !to_visions.is_empty() {
                            return true; //illegal move because we are moving into check
                        }
                        if is_check {
                            //moving the king and currently in check
                            //need to check whether we are moving into a check obscured by ourself
                            let from_visions = pseudomoves.get_vision(turn.flip(), from_sq);
                            for from_vision in from_visions {
                                match from_vision {
                                    Vision::Teleport { piece, from } => {}
                                    Vision::Slide {
                                        piece,
                                        from,
                                        slide,
                                        slide_idx,
                                    } => {
                                        if *slide_idx < slide.len() - 1 {
                                            let danger_sq = slide[slide_idx + 1];
                                            if to_sq == danger_sq {
                                                return true; //illegal move because we are moving into a check which was obscured by ourself
                                            }
                                        }
                                    }
                                }
                            }
                            return false;
                        } else {
                            return false; //moving to a non-checked square is always legal
                        }
                    } else {
                        //find all pieces we are pinned by
                        let mut pinners = vec![];
                        board.make_move(pseudo_move);
                        for pinner_vis in pseudomoves.get_vision(turn.flip(), from_sq) {
                            match pinner_vis {
                                Vision::Teleport { piece, from } => {}
                                Vision::Slide {
                                    piece: pinner_piece,
                                    from: pinner_from,
                                    slide: pinner_slide,
                                    slide_idx: pinner_slide_idx,
                                } => {
                                    'PIN_LOOP: for danger_sq in pinner_slide {
                                        //compute whether we are in check after making pseudomove
                                        match board.get_square(*danger_sq) {
                                            Some(Piece { kind, team, .. }) => {
                                                if kind == PieceKind::King && team == turn {
                                                    pinners.push(pinner_vis);
                                                }
                                                break 'PIN_LOOP;
                                            }
                                            None => {}
                                        }
                                    }
                                }
                            }
                        }
                        board.unmake_move();
                        match pinners.len() {
                            0 => {
                                //not pinned
                            }
                            1 => {
                                //pinned by one piece. Legal iff we are taking it
                                match pinners[0] {
                                    Vision::Teleport { piece, from } => panic!(),
                                    Vision::Slide {
                                        piece: pinner_piece,
                                        from: pinner_from,
                                        slide: pinner_slide,
                                        slide_idx: pinner_slide_idx,
                                    } => {
                                        if to_sq == *pinner_from {
                                            debug_assert_eq!(victim_opt.unwrap(), *pinner_piece);
                                        } else {
                                            return true; //we are not taking the pinner piece, so this move is illegal
                                        }
                                    }
                                }
                            }
                            _ => {
                                //pinned by two or more pieces. Never legal to move
                                return true;
                            }
                        }

                        //from here onwards we are not pinned
                        if is_check {
                            debug_assert!(checkers.len() >= 1);
                            if checkers.len() == 1 {
                                let unique_sliding_checker = &checkers[0];
                                match victim_opt {
                                    Some(victim) => {
                                        let (checking_piece, checking_from) =
                                            match unique_sliding_checker {
                                                Vision::Teleport {
                                                    piece: checking_piece,
                                                    from: checking_from,
                                                } => (checking_piece, checking_from),
                                                Vision::Slide {
                                                    piece: checking_piece,
                                                    from: checking_from,
                                                    slide: checking_slide,
                                                    slide_idx: checking_slide_idx,
                                                } => (checking_piece, checking_from),
                                            };
                                        if to_sq == *checking_from {
                                            debug_assert_eq!(victim, *checking_piece);
                                            return false; //taking a unique checking piece is legal
                                        }
                                    }
                                    None => {}
                                }
                            }
                            //we are in check and taking any checking pieces is not legal
                            for checker in checkers {
                                match checker {
                                    Vision::Teleport { .. } => {
                                        return true; //in check by a teleporter, not moving the king, and not taking the checker is not legal
                                    }
                                    Vision::Slide { .. } => {}
                                }
                            }
                            //we are in check only by sliders and taking a checking slider is not legal
                            if checkers.iter().all(|checker| match checker {
                                Vision::Teleport { .. } => panic!(),
                                Vision::Slide {
                                    piece: checking_piece,
                                    from: checking_from,
                                    slide: checking_slide,
                                    slide_idx: checking_slide_idx,
                                } => (0..*checking_slide_idx)
                                    .map(|block_idx| checking_slide[block_idx])
                                    .any(|block_sq| block_sq == to_sq),
                            }) {
                                //the move blocks all checking sliders so is legal
                                false
                            } else {
                                //the move failed to block all checking sliders so is illegal
                                true
                            }
                        } else {
                            return false; //not pinned and not in check is a legal move
                        }
                    }
                }
            }
        };

        let mut moves: Vec<Move> = vec![];
        for pseudo_move in pseudomoves.get_pseudomoves(turn) {
            //compute whether pseudo_move is legal is not
            let illegal = is_illegal(board, *pseudo_move);

            if cfg!(debug_assertions) {
                //check that the fast check calculation is valid
                let mut test_board = board.clone();
                test_board.make_move(*pseudo_move);
                let test_board_pseudomoves = PseudoMoves::new(&test_board);
                let king_square = test_board.get_king_square(turn);
                let test_illegal = !test_board_pseudomoves
                    .get_vision(turn.flip(), king_square)
                    .is_empty();
                test_board.unmake_move();
                if test_illegal != illegal {
                    println!("NUM = {:?}", board.get_move_num());
                    println!("MOVES = {:#?}", board.moves);
                    println!("DODGY MOVE = {:?}", pseudo_move);
                }
                assert_eq!(test_illegal, illegal);
            }

            if !illegal {
                moves.push(*pseudo_move);
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
                    Score::Lost(board.get_move_num()) //in check with no legal moves -> loose
                } else {
                    Score::Draw(board.get_move_num()) //not in check with no legal moves -> draw
                }
            } else {
                let mut score = 0;
                for (sq, piece) in board.white_pieces.iter().chain(board.black_pieces.iter()) {
                    if piece.kind != PieceKind::King {
                        score += signed_score!(piece.team, piece.kind.worth().unwrap() * 1000);
                    }
                    if piece.kind == PieceKind::Pawn {
                        match board.signature.get_pawn_promotion_distance(*sq, piece.team) {
                            Some(1) => {
                                score += signed_score!(piece.team, 2500);
                            }
                            Some(2) => {
                                score += signed_score!(piece.team, 300);
                            }
                            Some(3) => {
                                score += signed_score!(piece.team, 100);
                            }
                            Some(4) => {
                                score += signed_score!(piece.team, 20);
                            }
                            Some(5) => {
                                score += signed_score!(piece.team, 1);
                            }
                            Some(_) => {}
                            None => {}
                        }
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
                        let from_piece = match &vision {
                            Vision::Teleport { piece, from } => piece,
                            Vision::Slide {
                                piece,
                                from,
                                slide,
                                slide_idx,
                            } => piece,
                        };
                        if from_piece.kind != PieceKind::King {
                            let from_worth = from_piece.kind.worth().unwrap();
                            match board.get_square(sq) {
                                Some(to_piece) => {
                                    if to_piece.kind != PieceKind::King {
                                        let to_worth = to_piece.kind.worth();
                                        if to_piece.team == from_piece.team {
                                            //defend
                                        } else {
                                            //attack
                                            score += signed_score!(team, (10 - from_worth) * 3);
                                        }
                                    }
                                }
                                None => {
                                    //visible

                                    score += signed_score!(team, 10 - from_worth);
                                }
                            }
                        }
                    }
                }
                Score::Heuristic(match board.get_turn() {
                    Team::White => score,
                    Team::Black => -score,
                })
            }
        };

        Self {
            moves: moves.into_iter().map(|m| MoveData::new(m)).collect(),
            is_check: is_check,
            evaluation: score,
        }
    }

    pub fn is_terminal(&self) -> bool {
        self.moves.is_empty()
    }

    pub fn is_check(&self) -> bool {
        self.is_check
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
