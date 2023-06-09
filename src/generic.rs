use std::{iter::Enumerate, slice::SliceIndex};
pub trait Signature: Sized {
    fn num_sq(&self) -> usize;

    //all the immediate neighbours of idx in horz and vert directions
    fn flat_nbs(&self, idx: usize) -> Vec<usize>;

    //all the immediate neighbours of idx in diagonal directions
    fn diag_nbs(&self, idx: usize) -> Vec<usize>;

    /*
    assuming i and j represent a set from one square to anther, either horz, vert, or diag
    return all possible (might be multiple e.g. on a wormhole board) next steps k so that i,j,k are "evenly spaced"
    */
    fn flat_opp(&self, i: usize, j: usize) -> Vec<usize>;

    fn diag_opp(&self, i: usize, j: usize) -> Vec<usize>;

    //given a flat move from i to j, what are the possible following orthogonal flat moves
    fn flat_nopp(&self, i: usize, j: usize) -> Vec<usize> {
        let opps = self.flat_opp(i, j);
        let mut ans: Vec<usize> = vec![];
        for k in self.flat_nbs(j) {
            if !opps.contains(&k) && k != i {
                ans.push(k);
            }
        }
        return ans;
    }

    /*
    generate all possible sliding moves from a given square
    note that there may be branching, for example in wormhole chess there can
    be multiple continuations of the same initial slide
    we must also take care to avoid infinite loops, for example round the wormhole
    if an infinite loop occurs, we end with the starting point (so effectively a null move can be played)
     */
    fn gen_slides<F: Fn(usize) -> Vec<usize>, G: Fn(usize, usize) -> Vec<usize>>(
        &self,
        idx: usize,
        nbs: F,
        opp: G,
    ) -> Vec<Vec<usize>> {
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
    }

    //return all the pawn moves. regular moves in the first list, and double moves in the second list
    fn pawn_moves(&self, team: Team, idx: usize) -> (Vec<usize>, Vec<usize>);

    fn starting_board(&self) -> Board;
}

#[derive(Clone, Eq, PartialEq)]
pub enum Team {
    White,
    Black,
}

#[derive(Clone, Eq, PartialEq)]
pub enum PieceKind {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Clone)]
pub struct Piece {
    team: Team,
    kind: PieceKind,
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

#[derive(Clone)]
pub enum Square {
    Unoccupied,
    Occupied(Piece),
}

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
        board.validate_structure();
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
        board.validate_structure();
        return board;
    }

    fn validate_structure(&self) {
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
        return &self.squares[idx];
    }

    pub fn generate_moves(&self) -> Vec<Move> {
        let mut moves = vec![];

        return moves
    }
}

pub struct NormalMove {
    from_idx: usize,
    to_idx: usize,
}

pub enum Move {
    NormalMove,
}

pub struct MoveManager {
    num: usize,
    flat_slides: Vec<Vec<Vec<usize>>>,
    diag_slides: Vec<Vec<Vec<usize>>>,
    knight_moves: Vec<Vec<usize>>,
    king_moves: Vec<Vec<usize>>,
}

impl MoveManager {
    pub fn new<'a, S: Signature>(signature: &'a S) -> Self {
        let num = signature.num_sq();

        let flat_slides = (0..num)
            .map(|a| {
                let mut ans = signature.gen_slides(
                    a,
                    |i| signature.flat_nbs(i),
                    |i, j| signature.flat_opp(i, j),
                );
                //remove duplicates
                ans.sort();
                ans.dedup();
                ans
            })
            .collect();

        let diag_slides = (0..num)
            .map(|a| {
                let mut ans = signature.gen_slides(
                    a,
                    |i| signature.diag_nbs(i),
                    |i, j| signature.diag_opp(i, j),
                );
                //remove duplicates
                ans.sort();
                ans.dedup();
                ans
            })
            .collect();

        let knight_moves = (0..num)
            .map(|a| {
                let mut ans: Vec<usize> = vec![];
                //flat 2 side 1
                for b in signature.flat_nbs(a) {
                    for c in signature.flat_opp(a, b) {
                        for d in signature.flat_nopp(b, c) {
                            ans.push(d);
                        }
                    }
                }
                //side 1 flat 2
                for b in signature.flat_nbs(a) {
                    for c in signature.flat_nopp(a, b) {
                        for d in signature.flat_opp(b, c) {
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

        let king_moves = (0..num)
            .map(|a| {
                let mut ans: Vec<usize> = vec![];
                for b in signature.flat_nbs(a) {
                    ans.push(b);
                }
                for b in signature.diag_nbs(a) {
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
        }
    }

    fn generate_moves(board: Board) -> Vec<Move> {
        vec![]
    }
}

#[derive(Clone, Copy)]
pub enum HighlightKind {
    Test,
}

#[derive(Clone, Copy)]
pub struct Highlight {
    pub idx: usize,
    pub kind: HighlightKind,
}

pub trait Interface<'a, S: Signature> {
    fn new(signature: &'a S) -> Self;

    fn show_board(&self, board: &Board, highlights: Vec<Highlight>);
}

pub struct Game<'a, S: Signature, I: Interface<'a, S>> {
    signature: &'a S,
    move_manager: MoveManager,
    interface: I,

    board_history: Vec<Board>,
}

impl<'a, S: Signature, I: Interface<'a, S>> Game<'a, S, I> {
    pub fn new(signature: &'a S) -> Self {
        let starting_board = signature.starting_board();
        let move_manager = MoveManager::new(signature);
        Self {
            signature: signature,
            move_manager: move_manager,
            interface: I::new(signature),

            board_history: vec![starting_board],
        }
    }

    fn current_board(&self) -> &Board {
        match self.board_history.len() {
            0 => panic!("Should have at least one board in the board history"),
            n => &self.board_history[n - 1],
        }
    }

    pub fn run(&self) {
        let current_board = self.current_board();
        self.interface.show_board(current_board, vec![]);
    }
}
