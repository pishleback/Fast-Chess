#[derive(Debug)]
pub struct Signature {
    num: usize,
    flat_slides: Vec<Vec<Vec<usize>>>,
    diag_slides: Vec<Vec<Vec<usize>>>,
    knight_moves: Vec<Vec<usize>>,
    king_moves: Vec<Vec<usize>>,
    starting_board: Board,
}

impl Signature {
    //flat_nbs : all the immediate neighbours of idx in horz and vert directions
    //diag_nbs : all the immediate neighbours of idx in diagonal directions
    //flat_opp and diag_opp : assuming i and j represent a set from one square to anther, either horz, vert, or diag
    //return all possible (might be multiple e.g. on a wormhole board) next steps k so that i,j,k are "evenly spaced"
    //pawn_moves : return all the pawn moves. regular moves in the first list, and double moves in the second list
    pub fn new(
        num: usize,
        flat_nbs: &dyn Fn(usize) -> Vec<usize>,
        diag_nbs: &dyn Fn(usize) -> Vec<usize>,
        flat_opp: &dyn Fn(usize, usize) -> Vec<usize>,
        diag_opp: &dyn Fn(usize, usize) -> Vec<usize>,
        pawn_moves: &dyn Fn(Team, usize) -> (Vec<usize>, Vec<usize>),
        starting_board: Board,
    ) -> Self {
        //given a flat move from i to j, what are the possible following orthogonal flat moves
        let flat_nopp = |i: usize, j: usize| -> Vec<usize> {
            let opps = flat_opp(i, j);
            let mut ans: Vec<usize> = vec![];
            for k in flat_nbs(j) {
                if !opps.contains(&k) && k != i {
                    ans.push(k);
                }
            }
            return ans;
        };

        /*
        generate all possible sliding moves from a given square
        note that there may be branching, for example in wormhole chess there can
        be multiple continuations of the same initial slide
        we must also take care to avoid infinite loops, for example round the wormhole
        if an infinite loop occurs, we end with the starting point (so effectively a null move can be played)
         */
        let gen_slides = |idx: usize,
                          nbs: &dyn Fn(usize) -> Vec<usize>,
                          opp: &dyn Fn(usize, usize) -> Vec<usize>|
         -> Vec<Vec<usize>> {
            struct RestSlide<'a> {
                f: &'a dyn Fn(&RestSlide, Vec<usize>, usize, usize) -> Vec<Vec<usize>>,
            }
            let rest_slide = RestSlide {
                f: &|rest_slide, block, i, j| {
                    if block.contains(&j) {
                        //we found a loop
                        return vec![block];
                    } else {
                        //not found a loop, so look at all possible next steps and all continuations of the slide from that next step
                        let mut slides = vec![];
                        let mut none = true;
                        for k in opp(i, j) {
                            none = false;
                            for slide in (rest_slide.f)(
                                rest_slide,
                                {
                                    let mut tmp = vec![];
                                    tmp.extend(&block);
                                    tmp.append(&mut vec![j]);
                                    tmp
                                },
                                j,
                                k,
                            ) {
                                slides.push({
                                    let mut tmp: Vec<usize> = vec![j];
                                    tmp.extend(slide);
                                    tmp
                                });
                            }
                        }
                        if none {
                            slides.push(vec![j]);
                        }
                        return slides;
                    }
                },
            };

            // let rest_slide = |block: Vec<usize>, i: usize, j: usize| -> Vec<Vec<usize>> {

            let mut ans = vec![];
            for jdx in nbs(idx) {
                for slide in (rest_slide.f)(&rest_slide, vec![idx], idx, jdx) {
                    ans.push(slide);
                }
            }
            return ans;
        };

        let flat_slides: Vec<Vec<Vec<usize>>> = (0..num)
            .map(|idx| gen_slides(idx, flat_nbs, flat_opp))
            .collect();

        let diag_slides: Vec<Vec<Vec<usize>>> = (0..num)
            .map(|idx| gen_slides(idx, diag_nbs, diag_opp))
            .collect();

        let knight_moves: Vec<Vec<usize>> = (0..num)
            .map(|a| {
                let mut ans: Vec<usize> = vec![];

                //flat 2 side 1
                for b in flat_nbs(a) {
                    for c in flat_opp(a, b) {
                        for d in flat_nopp(b, c) {
                            ans.push(d);
                        }
                    }
                }
                //side 1 flat 2
                for b in flat_nbs(a) {
                    for c in flat_nopp(a, b) {
                        for d in flat_opp(b, c) {
                            ans.push(d);
                        }
                    }
                }
                //remove duplicates
                ans.sort();
                ans.dedup();
                ans
            })
            .collect();

        let king_moves: Vec<Vec<usize>> = (0..num)
            .map(|a| {
                let mut ans: Vec<usize> = vec![];
                for b in flat_nbs(a) {
                    ans.push(b);
                }
                for b in diag_nbs(a) {
                    ans.push(b);
                }
                //remove duplicates
                ans.sort();
                ans.dedup();
                ans
            })
            .collect();

        Self {
            num,
            flat_slides,
            diag_slides,
            knight_moves,
            king_moves,
            starting_board,
        }
    }

    pub fn get_num(&self) -> usize {
        self.num
    }

    pub fn get_starting_board(&self) -> &Board {
        &self.starting_board
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Team {
    White,
    Black,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Clone)]
pub struct Piece {
    pub team: Team,
    pub kind: PieceKind,
}

impl Piece {
    pub fn new(team: Team, kind: PieceKind) -> Piece {
        Piece { team, kind }
    }

    pub fn get_team(&self) -> &Team {
        &self.team
    }

    pub fn get_kind(&self) -> &PieceKind {
        &self.kind
    }
}

#[derive(Debug, Clone)]
pub enum Square {
    Unoccupied,
    Occupied(Piece),
}

#[derive(Debug, Clone)]
pub struct Board {
    turn: Team,
    squares: Vec<Square>,
    white_pieces: Vec<(usize, PieceKind)>,
    black_pieces: Vec<(usize, PieceKind)>,
}

impl Board {
    pub fn new_from_squares(turn: Team, squares: Vec<Square>) -> Self {
        let mut white_pieces = vec![];
        let mut black_pieces = vec![];

        for (idx, sq) in squares.iter().enumerate() {
            match sq {
                Square::Occupied(piece) => match piece.team {
                    Team::White => {
                        white_pieces.push((idx, piece.kind.clone()));
                    }
                    Team::Black => {
                        black_pieces.push((idx, piece.kind.clone()));
                    }
                },
                Square::Unoccupied => {}
            };
        }

        let board = Self {
            turn,
            squares,
            white_pieces,
            black_pieces,
        };
        board.validate();
        return board;
    }

    pub fn new_from_pieces(
        turn: Team,
        num: usize,
        white_pieces: Vec<(usize, PieceKind)>,
        black_pieces: Vec<(usize, PieceKind)>,
    ) -> Self {
        let mut squares = vec![Square::Unoccupied; num];
        for (idx, kind) in &white_pieces {
            squares[*idx] = Square::Occupied(Piece {
                team: Team::White,
                kind: kind.clone(),
            });
        }
        for (idx, kind) in &black_pieces {
            squares[*idx] = Square::Occupied(Piece {
                team: Team::Black,
                kind: kind.clone(),
            });
        }

        let board = Self {
            turn,
            squares,
            white_pieces,
            black_pieces,
        };
        board.validate();
        return board;
    }

    fn validate(&self) {
        let mut white_count: usize = 0;
        let mut black_count: usize = 0;
        for (idx, sq) in self.squares.iter().enumerate() {
            debug_assert!(idx < self.squares.len());
            match &self.squares[idx] {
                Square::Unoccupied => {}
                Square::Occupied(piece) => match piece.team {
                    Team::White => {
                        white_count += 1;
                    }
                    Team::Black => {
                        black_count += 1;
                    }
                },
            };
        }
        debug_assert_eq!(white_count, self.white_pieces.len());
        debug_assert_eq!(black_count, self.black_pieces.len());

        for (idx, kind) in &self.black_pieces {
            debug_assert!(idx < &self.squares.len());
            debug_assert!(match &self.squares[*idx] {
                Square::Unoccupied => false,
                Square::Occupied(piece) => piece.team == Team::Black && &piece.kind == kind,
            })
        }
    }

    pub fn at(&self, idx: usize) -> &Square {
        &self.squares[idx]
    }

    pub fn get_turn(&self) -> &Team {
        &self.turn
    }

    pub fn get_pieces(&self) -> Vec<(usize, Piece)> {
        let mut pieces = vec![];
        for (sq, piece_kind) in &self.white_pieces {
            pieces.push((
                *sq,
                Piece {
                    team: Team::White,
                    kind: piece_kind.clone(),
                },
            ));
        }
        for (sq, piece_kind) in &self.black_pieces {
            pieces.push((
                *sq,
                Piece {
                    team: Team::Black,
                    kind: piece_kind.clone(),
                },
            ));
        }
        pieces
    }
}

// #[derive(Clone, Copy)]
// pub enum HighlightKind {
//     Test,
// }

// #[derive(Clone, Copy)]
// pub struct Highlight {
//     pub idx: usize,
//     pub kind: HighlightKind,
// }

// pub trait Interface {
//     fn show_board(&self, board: &Board, highlights: Vec<Highlight>);
// }

// pub fn run(signature: Signature, interface: &dyn Interface) {
//     interface.show_board(signature.get_starting_board(), vec![]);

//     let num = signature.get_num();

//     for idx in 0..num {
//         let mut highlights: Vec<Highlight> = vec![];

//         // highlights.extend(signature.king_moves[idx].iter().map(|jdx| -> Highlight {
//         //     Highlight {
//         //         idx: *jdx,
//         //         kind: HighlightKind::Test,
//         //     }
//         // }));

//         for slide in &signature.flat_slides[idx] {
//             for jdx in slide {
//                 highlights.push(Highlight {
//                     idx: *jdx,
//                     kind: HighlightKind::Test,
//                 });
//             }
//         }

//         for slide in &signature.diag_slides[idx] {
//             for jdx in slide {
//                 highlights.push(Highlight {
//                     idx: *jdx,
//                     kind: HighlightKind::Test,
//                 });
//             }
//         }

//         interface.show_board(signature.get_starting_board(), highlights);
//     }
// }
