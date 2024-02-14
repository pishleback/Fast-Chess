use std::cell::RefCell;
use std::os::windows::thread;
use std::sync::{Arc, Mutex};
use std::thread::{sleep_ms, JoinHandle, Thread};
use std::time::Duration;

use super::info::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApproxScore {
    score: Score,
}

#[derive(Debug, Clone)]
struct BoardData {
    info: BoardInfo,
    moves: Vec<MoveData>,
}

impl BoardData {
    fn new(board: &Board) -> Self {
        let info = board.generate_info();
        let moves = info.get_moves(board.get_turn()).clone();
        Self {
            info: info,
            moves: moves
                .into_iter()
                .map(|mv| MoveData {
                    mv: mv,
                    board: None,
                    score: None,
                })
                .collect(),
        }
    }

    fn get_moves(&self) -> Vec<Move> {
        self.moves.iter().map(|move_data| (move_data.mv)).collect()
    }

    fn get_move(&self, move_idx: MoveIdx) -> &MoveData {
        &self.moves[move_idx.idx]
    }
}

#[derive(Debug, Clone)]
struct MoveData {
    mv: Move,
    board: Option<BoardData>,
    score: Option<ApproxScore>,
}

impl MoveData {
    fn get_board(&mut self, board: &Board) -> &mut BoardData {
        if self.board.is_none() {
            self.board = Some(BoardData::new(board));
        }
        self.board.as_mut().unwrap()
    }

    fn get_approx_score(&self) -> Score {
        match &self.score {
            Some(approx_score) => approx_score.score,
            None => Score::Finite(0),
        }
    }

    fn compute_score(
        &mut self,
        stop_check: &impl Fn() -> bool,
        board: &mut Board,
        depth: usize,
    ) -> Result<Score, ()> {
        self.alpha_beta(stop_check, board, depth, Score::NegInf, Score::PosInf)
    }

    fn alpha_beta(
        &mut self,
        stop_check: &impl Fn() -> bool,
        board: &mut Board,
        depth: usize,
        mut alpha: Score,
        beta: Score,
    ) -> Result<Score, ()> {
        if stop_check() {
            return Err(());
        }

        //alpha is the score we already know we can achive
        //beta is the score the opponent knows they can achive
        let board_data = self.get_board(board);

        let score = -{
            if depth == 0 {
                -self.quiesce(stop_check, board, 0, alpha, beta)?
                // match board.get_turn() {
                //     Team::White => board_data.info.raw_score(),
                //     Team::Black => -board_data.info.raw_score(),
                // }
            } else {
                let mut bestscore = Score::NegInf;
                let mut moves = board_data.moves.iter_mut().collect::<Vec<_>>();
                moves.sort_by_key(|mv| -mv.get_approx_score());
                for move_data in &mut moves {
                    let m = move_data.mv;
                    // let og_board = self.board.clone();
                    board.make_move(m);
                    if let Ok(score) =
                        move_data.alpha_beta(stop_check, board, depth - 1, -beta, -alpha)
                    {
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
                    } else {
                        board.unmake_move(m);
                        return Err(());
                    }
                }
                bestscore
            }
        };
        self.score = Some(ApproxScore { score });
        Ok(score)
    }

    fn quiesce(
        &mut self,
        stop_check: &impl Fn() -> bool,
        board: &mut Board,
        deep: usize,
        mut alpha: Score,
        beta: Score,
    ) -> Result<Score, ()> {
        if stop_check() {
            return Err(());
        }

        //alpha is the score we already know we can achive
        //beta is the score the opponent knows they can achive
        let board_data = self.get_board(board);
        let score = -{
            let mut bestscore = match board.get_turn() {
                Team::White => board_data.info.raw_score(),
                Team::Black => -board_data.info.raw_score(),
            }; //not valid in late game when null move observation is false
            // let mut moves = board_data.moves.iter_mut().collect::<Vec<_>>();
            // moves.sort_by_key(|mv| -match mv.mv {
            //     Move::Move {
            //         piece,
            //         from_sq,
            //         to_sq,
            //     } => Score::Finite(0),
            //     Move::Capture {
            //         piece,
            //         victim,
            //         from_sq,
            //         to_sq,
            //     } => victim.kind.worth(),
            // });
            // for move_data in &mut moves {
            //     let m = move_data.mv;
            //     if match m {
            //         Move::Move { .. } => false,
            //         Move::Capture { .. } => true,
            //     } {
            //         // let og_board = self.board.clone();
            //         board.make_move(m);
            //         if let Ok(score) = move_data.quiesce(stop_check, board, deep + 1, -beta, -alpha)
            //         {
            //             board.unmake_move(m);
            //             // debug_assert_eq!(self.board, &og_board);
            //             if score > bestscore {
            //                 bestscore = score;
            //                 if score > alpha {
            //                     alpha = score;
            //                 }
            //             }
            //             if score >= beta {
            //                 break; //the opponent already has a method to avoid us getting this score here, so we can stop looking
            //             }
            //         } else {
            //             board.unmake_move(m);
            //             return Err(());
            //         }
            //     }
            // }
            bestscore
        };
        self.score = Some(ApproxScore { score });
        Ok(score)
    }
}

#[derive(Debug)]
pub struct BoardTree {
    board: Board,
    root: BoardData,
}

impl BoardTree {
    pub fn new(mut board: Board) -> Self {
        let root = BoardData::new(&mut board);
        let tree = BoardTree { board, root };
        tree
    }

    // pub fn get_pieces(&self) -> Vec<(Square, Piece)> {
    //     self.board.get_pieces()
    // }

    // pub fn get_square(&self, sq: Square) -> Option<Piece> {
    //     self.board.get_square(sq)
    // }

    // pub fn get_turn(&self) -> Team {
    //     self.board.get_turn()
    // }

    // pub fn get_enumerated_moves(&mut self) -> Vec<(MoveIdx, Move)> {
    //     self.root
    //         .get_moves()
    //         .into_iter()
    //         .enumerate()
    //         .map(|(idx, mv)| (MoveIdx { idx }, mv))
    //         .collect()
    // }

    // pub fn get_moves(&self) -> Vec<Move> {
    //     self.root.get_moves()
    // }

    // pub fn get_move(&self, m: MoveIdx) -> Move {
    //     self.get_moves()[m.idx]
    // }

    // pub fn get_best_move(&mut self) -> Option<MoveIdx> {
    //     let mut depth = 0;
    //     let mut overall_best_move_idx = None;
    //     let start = std::time::Instant::now();
    //     loop {
    //         let mut best_move_idx = None;
    //         let mut highest_score = Score::NegInf;
    //         let n = self.root.moves.len();
    //         for idx in 0..n {
    //             if start.elapsed().as_secs() > 2 {
    //                 return overall_best_move_idx;
    //             }
    //             let move_idx = MoveIdx { idx };
    //             let move_data = &mut self.root.moves[idx];
    //             self.board.make_move(move_data.mv);
    //             let score = move_data
    //                 .compute_score(&|| false, &mut self.board, depth)
    //                 .unwrap();
    //             self.board.unmake_move(move_data.mv);
    //             println!(
    //                 "depth {:?}    {:?} / {:?} = {:?}",
    //                 depth + 1,
    //                 idx + 1,
    //                 n,
    //                 score
    //             );
    //             if score > highest_score {
    //                 highest_score = score;
    //                 best_move_idx = Some(move_idx);
    //             }
    //         }
    //         overall_best_move_idx = best_move_idx;
    //         depth += 1;
    //     }
    // }

    fn best_move_at_depth(
        &mut self,
        depth: usize,
        stop_check: impl Fn() -> bool,
    ) -> Result<Option<MoveIdx>, ()> {
        let mut best_move_idx = None;
        let mut highest_score = Score::NegInf;
        let n = self.root.moves.len();
        for idx in 0..n {
            let move_idx = MoveIdx { idx };
            let move_data = &mut self.root.moves[idx];
            self.board.make_move(move_data.mv);
            if let Ok(score) = move_data.compute_score(&stop_check, &mut self.board, depth) {
                self.board.unmake_move(move_data.mv);
                if score > highest_score {
                    highest_score = score;
                    best_move_idx = Some(move_idx);
                }
            } else {
                self.board.unmake_move(move_data.mv);
                return Err(());
            }
        }
        Ok(best_move_idx)
    }

    pub fn make_move(&mut self, m: MoveIdx) {
        let md = self.root.get_move(m);
        self.board.make_move(md.mv);
        self.root = match &md.board {
            Some(board) => board.clone(),
            None => BoardData::new(&mut self.board),
        }
    }
}

#[derive(Debug)]
pub struct AiOn {
    stop_flag: Arc<Mutex<bool>>,
    best_move: Arc<Mutex<Option<MoveIdx>>>,
    handler: JoinHandle<(BoardTree, Option<MoveIdx>)>,
}

impl AiOn {
    fn think(
        stop_flag: Arc<Mutex<bool>>,
        best_move: Arc<Mutex<Option<MoveIdx>>>,
        mut tree: BoardTree,
    ) -> (BoardTree, Option<MoveIdx>) {
        let stop_check = || *stop_flag.lock().unwrap();
        let mut depth = 0;
        println!("start");
        loop {
            println!("depth = {:?}", depth);
            match tree.best_move_at_depth(depth, stop_check) {
                Ok(best_move_answer) => {
                    *best_move.lock().unwrap() = best_move_answer;
                }
                Err(()) => {
                    break;
                }
            }
            depth += 1;
        }
        (tree, *best_move.lock().unwrap())
    }

    pub fn current_best_move(&self) -> Option<MoveIdx> {
        *self.best_move.lock().unwrap()
    }

    pub fn finish(self) -> (AiOff, Option<MoveIdx>) {
        *self.stop_flag.lock().unwrap() = true;
        let (tree, best_move) = self.handler.join().unwrap();
        (AiOff { tree }, best_move)
    }
}

#[derive(Debug)]
pub struct AiOff {
    tree: BoardTree,
}

impl AiOff {
    pub fn new(board: Board) -> Self {
        Self {
            tree: BoardTree::new(board),
        }
    }

    pub fn get_board(&self) -> &Board {
        &self.tree.board
    }

    pub fn get_moves(&self) -> Vec<Move> {
        self.tree
            .root
            .moves
            .iter()
            .map(|move_data| move_data.mv)
            .collect()
    }

    pub fn make_move(&mut self, m: MoveIdx) {
        self.tree.make_move(m);
    }

    pub fn start(self) -> AiOn {
        let stop_flag = Arc::new(Mutex::new(false));
        let best_move = Arc::new(Mutex::new(None));

        AiOn {
            stop_flag: stop_flag.clone(),
            best_move: best_move.clone(),
            handler: std::thread::spawn(move || {
                AiOn::think(stop_flag.clone(), best_move.clone(), self.tree)
            }),
        }
    }
}
