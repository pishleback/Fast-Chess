use super::ai::*;
use super::score::*;
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GrasshopperVisionKind {
    Slide,
    Hurdle,
    Land,
}

#[derive(Debug, Clone)]
enum Vision {
    Teleport {
        piece: Piece,
        from: Square,
        to: Square,
    },
    Slide {
        piece: Piece,
        from: Square,
        slide: Vec<Square>,
        slide_idx: usize,
    },
    Grasshopper {
        piece: Piece,
        from: Square,
        slide: Vec<Square>,
        slide_idx: usize,
        kind: GrasshopperVisionKind,
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
            ($piece:expr, $victim:expr, $from_sq:expr, $to_sq:expr, $en_crossantable:expr) => {
                match board.signature.get_pawn_promotions($to_sq, $piece.team) {
                    None => {
                        add_move!(
                            Move::Standard {
                                from_piece: $piece,
                                to_piece: match $en_crossantable {
                                    None => $piece.moved(),
                                    Some(take_sq) => {
                                        Piece {
                                            team: $piece.team,
                                            moved: true,
                                            kind: PieceKind::Pawn(EnCroissantable::Yes {
                                                move_num: board.get_move_num() + 1,
                                                take_sq: *take_sq,
                                            }),
                                        }
                                    }
                                },
                                victim: $victim,
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
                                    from_piece: $piece,
                                    to_piece: Piece {
                                        team: $piece.team,
                                        moved: true,
                                        kind: *kind
                                    },
                                    victim: $victim,
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
                                    to: *to_sq,
                                },
                                $piece.team
                            );
                            add_move!(
                                Move::Standard {
                                    from_piece: *$piece,
                                    to_piece: $piece.moved(),
                                    victim: None,
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
                                        to: *to_sq,
                                    },
                                    $piece.team
                                );
                                if blocking.kind != PieceKind::King {
                                    add_move!(
                                        Move::Standard {
                                            from_piece: *$piece,
                                            to_piece: $piece.moved(),
                                            victim: Some(blocking),
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
                                        to: *to_sq,
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
                            if !done.contains(&to_sq) {
                                done.insert(to_sq);
                                match board.get_square(to_sq) {
                                    None => {
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
                                                from_piece: $piece,
                                                to_piece: $piece.moved(),
                                                victim: None,
                                                from_sq: $from_sq,
                                                to_sq: to_sq,
                                            },
                                            $piece.team
                                        );
                                    }
                                    Some(blocking) => {
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
                                                        from_piece: $piece,
                                                        to_piece: $piece.moved(),
                                                        victim: Some(blocking),
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

                                        break;
                                    }
                                }
                            };
                            slide_idx += 1;
                        }
                    }
                }
            }};
        }

        macro_rules! add_grasshopper_slides {
            ($all_slides:expr, $piece:expr, $from_sq:expr) => {{
                let mut done = HashSet::new();
                for slides in $all_slides {
                    for slide in slides {
                        let mut slide_idx = 0;
                        while slide_idx < slide.len() {
                            let jump_sq = slide[slide_idx];
                            if !done.contains(&jump_sq) {
                                done.insert(jump_sq);
                                match board.get_square(jump_sq) {
                                    None => {
                                        add_vision!(
                                            jump_sq,
                                            Vision::Grasshopper {
                                                piece: $piece,
                                                from: $from_sq,
                                                slide: slide.clone(),
                                                slide_idx: slide_idx,
                                                kind: GrasshopperVisionKind::Slide
                                            },
                                            $piece.team
                                        );
                                    }
                                    Some(hurdle) => {
                                        add_vision!(
                                            jump_sq,
                                            Vision::Grasshopper {
                                                piece: $piece,
                                                from: $from_sq,
                                                slide: slide.clone(),
                                                slide_idx: slide_idx,
                                                kind: GrasshopperVisionKind::Hurdle
                                            },
                                            $piece.team
                                        );
                                        let land_slide_idx = slide_idx + 1;
                                        if land_slide_idx < slide.len() {
                                            let land_sq = slide[land_slide_idx];
                                            match board.get_square(land_sq) {
                                                None => {
                                                    add_move!(
                                                        Move::Standard {
                                                            from_piece: $piece,
                                                            to_piece: $piece.moved(),
                                                            victim: None,
                                                            from_sq: $from_sq,
                                                            to_sq: land_sq,
                                                        },
                                                        $piece.team
                                                    );
                                                    add_vision!(
                                                        land_sq,
                                                        Vision::Grasshopper {
                                                            piece: $piece,
                                                            from: $from_sq,
                                                            slide: slide,
                                                            slide_idx: land_slide_idx,
                                                            kind: GrasshopperVisionKind::Land
                                                        },
                                                        $piece.team
                                                    );
                                                }
                                                Some(land_piece) => {
                                                    if land_piece.team != $piece.team {
                                                        if land_piece.kind != PieceKind::King {
                                                            add_move!(
                                                                Move::Standard {
                                                                    from_piece: $piece,
                                                                    to_piece: $piece.moved(),
                                                                    victim: Some(land_piece),
                                                                    from_sq: $from_sq,
                                                                    to_sq: land_sq,
                                                                },
                                                                $piece.team
                                                            );
                                                        }
                                                    }
                                                    add_vision!(
                                                        land_sq,
                                                        Vision::Grasshopper {
                                                            piece: $piece,
                                                            from: $from_sq,
                                                            slide: slide,
                                                            slide_idx: land_slide_idx,
                                                            kind: GrasshopperVisionKind::Land
                                                        },
                                                        $piece.team
                                                    );
                                                }
                                            }
                                        }
                                        break;
                                    }
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
        all_pieces.sort_by_key(|(s, _p)| s.idx);

        let mut crossant_map = HashMap::new();
        for (sq, piece) in &all_pieces {
            match piece {
                Piece {
                    kind: PieceKind::Pawn(EnCroissantable::Yes { move_num, take_sq }),
                    ..
                } => {
                    if *move_num == board.get_move_num() {
                        crossant_map.insert(take_sq, *sq);
                    }
                }
                _ => {}
            }
        }

        for (from_sq, piece) in all_pieces {
            match piece.kind {
                PieceKind::Pawn(..) => {
                    //pawn movement
                    for pm in board.signature.get_pawn_moves(*from_sq, piece.team) {
                        let (first, seconds) = pm;
                        if board.get_square(*first).is_none() {
                            add_pawn_move!(*piece, None, *from_sq, *first, None::<&Square>);
                            for second in seconds {
                                if board.get_square(*second).is_none() {
                                    add_pawn_move!(*piece, None, *from_sq, *second, Some(first));
                                }
                            }
                        }
                    }
                    //pawn attacks
                    for to_sq in board.signature.get_pawn_takes(*from_sq, piece.team) {
                        match board.get_square(*to_sq) {
                            None => {
                                for (take_sq, victim_sq) in &crossant_map {
                                    if *take_sq == to_sq {
                                        //en crossant
                                        let victim = board.get_square(**victim_sq).unwrap();
                                        if victim.team != piece.team {
                                            add_move!(
                                                Move::EnCroissant {
                                                    pawn: *piece,
                                                    pawn_from: *from_sq,
                                                    pawn_to: *to_sq,
                                                    victim: victim,
                                                    victim_sq: **victim_sq
                                                },
                                                board.get_turn()
                                            );
                                        }
                                    }
                                }

                                add_vision!(
                                    to_sq,
                                    Vision::Teleport {
                                        piece: *piece,
                                        from: *from_sq,
                                        to: *to_sq,
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
                                            to: *to_sq,
                                        },
                                        piece.team
                                    );
                                    if diag.kind != PieceKind::King {
                                        add_pawn_move!(
                                            *piece,
                                            Some(diag),
                                            *from_sq,
                                            *to_sq,
                                            None::<&Square>
                                        );
                                    }
                                } else {
                                    add_vision!(
                                        to_sq,
                                        Vision::Teleport {
                                            piece: *piece,
                                            from: *from_sq,
                                            to: *to_sq,
                                        },
                                        piece.team
                                    );
                                }
                            }
                        }
                    }
                }
                PieceKind::Grasshopper => {
                    add_grasshopper_slides!(
                        vec![
                            board.signature.get_flat_slides(*from_sq).clone(),
                            board.signature.get_diag_slides(*from_sq).clone()
                        ],
                        *piece,
                        *from_sq
                    );
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

        for (team, castle_signature) in board.signature.get_castles() {
            if let (Some(king_piece), Some(rook_piece)) = (
                board.get_square(castle_signature.king_from),
                board.get_square(castle_signature.rook_from),
            ) {
                //note that king_piece may not actually be the king
                if !king_piece.moved && !rook_piece.moved {
                    if castle_signature
                        .not_occupied
                        .iter()
                        .chain(vec![&castle_signature.king_to, &castle_signature.rook_to])
                        .all(|sq| board.get_square(*sq).is_none())
                    {
                        add_move!(
                            Move::Castle {
                                king_from: castle_signature.king_from,
                                king_through: castle_signature.not_chcked.clone(),
                                king_to: castle_signature.king_to,
                                king_piece: king_piece,
                                rook_from: castle_signature.rook_from,
                                rook_to: castle_signature.rook_to,
                                rook_piece: rook_piece
                            },
                            team
                        );
                    }
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

        let is_illegal = |board: &mut Board, pseudo_move: &Move| -> bool {
            //compile a list of things which might be checking the king after the move is made
            let mut hot_squares = vec![];
            match pseudo_move {
                Move::Standard {
                    from_piece,
                    to_piece,
                    victim,
                    from_sq,
                    to_sq,
                } => {
                    if from_piece.kind == PieceKind::King {
                        debug_assert!(to_piece.kind == PieceKind::King);
                        hot_squares.push(*from_sq);
                        hot_squares.push(*to_sq);
                    } else {
                        hot_squares.push(board.get_king_square(turn));
                        hot_squares.push(*from_sq);
                        hot_squares.push(*to_sq);
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
                    if king_piece.kind == PieceKind::King {
                        if is_check {
                            return true; //can't castle while in check
                        }
                        for through in king_through {
                            //can't castle through check
                            for vis in pseudomoves.get_vision(turn.flip(), through.clone()) {
                                match vis {
                                    Vision::Teleport { .. } => {
                                        return true;
                                    }
                                    Vision::Slide { .. } => {
                                        return true;
                                    }
                                    Vision::Grasshopper {
                                        piece,
                                        from,
                                        slide,
                                        slide_idx,
                                        kind,
                                    } => {
                                        if kind == &GrasshopperVisionKind::Land {
                                            return true;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    hot_squares.push(*king_from);
                    for sq in king_through {
                        hot_squares.push(*sq);
                    }
                    hot_squares.push(*king_to);
                    hot_squares.push(*rook_from);
                    hot_squares.push(*rook_to);
                }
                Move::EnCroissant {
                    pawn,
                    pawn_from,
                    pawn_to,
                    victim,
                    victim_sq,
                } => {
                    hot_squares.push(*pawn_from);
                    hot_squares.push(*pawn_to);
                    hot_squares.push(*victim_sq);
                }
            }

            let mut hot_vis = vec![];
            for hot_sq in hot_squares {
                for vis in pseudomoves.get_vision(turn.flip(), hot_sq) {
                    hot_vis.push(vis);
                }
            }

            board.make_move(pseudo_move.clone());
            let new_king_sq = board.get_king_square(turn);

            let mut is_illegal = false;

            //go through each possible check and see if it is actually a checker
            'IS_ILLEGAL: for possible_checker in hot_vis {
                match possible_checker {
                    Vision::Teleport { piece, from, to } => match board.get_square(*from) {
                        Some(after_piece) => {
                            if after_piece.team == piece.team && *to == new_king_sq {
                                is_illegal = true;
                                break 'IS_ILLEGAL;
                            }
                        }
                        None => {}
                    },
                    Vision::Slide {
                        piece,
                        from,
                        slide,
                        slide_idx,
                    } => match board.get_square(*from) {
                        Some(after_piece) => {
                            if after_piece.team == piece.team {
                                for sq in slide {
                                    match board.get_square(*sq) {
                                        Some(slide_piece) => {
                                            if *sq == new_king_sq {
                                                is_illegal = true;
                                                break 'IS_ILLEGAL;
                                            }
                                            break;
                                        }
                                        None => {}
                                    }
                                }
                            }
                        }
                        None => {}
                    },
                    Vision::Grasshopper {
                        piece,
                        from,
                        slide,
                        slide_idx,
                        kind,
                    } => match board.get_square(*from) {
                        Some(after_piece) => {
                            if after_piece.team == piece.team {
                                for idx in 0..(slide.len() - 1) {
                                    let slide_sq = slide[idx];
                                    match board.get_square(slide_sq) {
                                        Some(hurdle_piece) => {
                                            let land_sq = slide[idx + 1];
                                            if land_sq == new_king_sq {
                                                is_illegal = true;
                                                break 'IS_ILLEGAL;
                                            }
                                            break;
                                        }
                                        None => {}
                                    }
                                }
                            }
                        }
                        None => {}
                    },
                }
            }

            board.unmake_move().unwrap();
            is_illegal
        };

        let mut moves: Vec<Move> = vec![];
        for pseudo_move in pseudomoves.get_pseudomoves(turn) {
            //compute whether pseudo_move is legal is not
            let illegal = is_illegal(board, pseudo_move);

            if cfg!(debug_assertions) {
                match pseudo_move {
                    Move::Castle { .. } => {}
                    _ => {
                        //check that the fast check calculation is valid
                        let mut test_board = board.clone();
                        test_board.make_move(pseudo_move.clone());
                        let test_board_pseudomoves = PseudoMoves::new(&test_board);
                        let king_square = test_board.get_king_square(turn);
                        let test_illegal = test_board_pseudomoves
                            .get_vision(turn.flip(), king_square)
                            .into_iter()
                            .any(|vis| match vis {
                                Vision::Teleport { .. } => true,
                                Vision::Slide { .. } => true,
                                Vision::Grasshopper { kind, .. } => {
                                    *kind == GrasshopperVisionKind::Land
                                }
                            });
                        test_board.unmake_move().unwrap();
                        if test_illegal != illegal {
                            println!("NUM = {:?}", board.get_move_num());
                            println!("MOVES = {:#?}", board.moves);
                            println!("DODGY MOVE = {:#?}", pseudo_move);
                        }
                        assert_eq!(test_illegal, illegal);
                    }
                }
            }

            if !illegal {
                moves.push(pseudo_move.clone());
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
                    if let PieceKind::Pawn(_) = piece.kind {
                        match board.signature.get_pawn_promotion_distance(*sq, piece.team) {
                            Some(1) => {
                                score += signed_score!(piece.team, 2500);
                            }
                            Some(2) => {
                                score += signed_score!(piece.team, 300);
                            }
                            Some(3) => {
                                score += signed_score!(piece.team, 25);
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
                            Vision::Teleport { piece, .. } => piece,
                            Vision::Slide { piece, .. } => piece,
                            Vision::Grasshopper { piece, .. } => piece,
                        };
                        if from_piece.kind != PieceKind::King {
                            let from_worth = from_piece.kind.worth().unwrap();
                            match board.get_square(sq) {
                                Some(to_piece) => {
                                    if to_piece.kind != PieceKind::King {
                                        let _to_worth = to_piece.kind.worth();
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

    pub fn get_moves(&self) -> Vec<&Move> {
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
