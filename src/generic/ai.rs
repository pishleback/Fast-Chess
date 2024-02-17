use std::cell::RefCell;
use std::cmp::max;
use std::ops::Neg;
use std::os::windows::thread;
use std::sync::{Arc, Mutex};
use std::thread::{sleep_ms, JoinHandle, Thread};
use std::time::Duration;

use super::board_data::*;
use super::score::*;
use super::*;

// #[derive(Debug, Clone, PartialEq, Eq)]
// struct ApproxScore {
//     score: Score,
// }

#[derive(Debug, Clone, Copy)]
struct AlphaBetaMaximizingResult {
    score: Score,
    depth: usize,
    exact: bool,
}
impl Neg for AlphaBetaMaximizingResult {
    type Output = AlphaBetaMinimizingResult;

    fn neg(self) -> Self::Output {
        AlphaBetaMinimizingResult {
            score: -self.score,
            depth: self.depth,
            exact: self.exact,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AlphaBetaMinimizingResult {
    score: Score,
    depth: usize,
    exact: bool,
}
impl Neg for AlphaBetaMinimizingResult {
    type Output = AlphaBetaMaximizingResult;

    fn neg(self) -> Self::Output {
        AlphaBetaMaximizingResult {
            score: -self.score,
            depth: self.depth,
            exact: self.exact,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MoveData {
    mv: Move,
    board: Option<BoardData>,
    approx_score: Option<AlphaBetaMinimizingResult>,
}

impl MoveData {
    pub fn new(mv: Move) -> Self {
        Self {
            mv,
            board: None,
            approx_score: None,
        }
    }

    pub fn get_move(&self) -> Move {
        self.mv
    }

    fn get_board(&mut self, board: &mut Board) -> &mut BoardData {
        if self.board.is_none() {
            self.board = Some(BoardData::new(board));
        }
        self.board.as_mut().unwrap()
    }

    fn get_approx_score(&self) -> Score {
        match &self.approx_score {
            Some(approx_score) => approx_score.score,
            None => Score::Heuristic(0),
        }
    }

    fn alpha_beta(
        &mut self,
        stop_check: &impl Fn() -> bool,
        board: &mut Board,
        depth: usize,
        max_depth: usize,
        max_quiesce_depth: usize,
        alpha: LowerBound,
        beta: UpperBound,
    ) -> Result<AlphaBetaMinimizingResult, ()> {
        board.make_move(self.mv);
        let board_data = self.get_board(board);
        if let Ok(abres) = board_data.alpha_beta(
            stop_check,
            board,
            depth,
            max_depth,
            max_quiesce_depth,
            -beta,
            -alpha,
        ) {
            board.unmake_move();
            self.approx_score = Some(-abres);
            Ok(-abres)
        } else {
            board.unmake_move();
            Err(())
        }
    }

    // fn quiesce(
    //     &mut self,
    //     stop_check: &impl Fn() -> bool,
    //     board: &mut Board,
    //     depth: usize,
    //     max_quiesce_depth: usize,
    //     mut alpha: LowerBound,
    //     beta: UpperBound,
    // ) -> Result<AlphaBetaResult, ()> {
    //     if stop_check() {
    //         return Err(());
    //     }

    //     //alpha is the score we already know we can achive
    //     //beta is the score the opponent knows they can achive
    //     let board_data = self.get_board(board);
    //     if board_data.is_terminal() {
    //         return Ok(-board_data.get_evaluation());
    //     }

    //     let score = -{
    //         let standpat = board_data.get_evaluation();
    //         if depth >= max_quiesce_depth {
    //             standpat
    //         } else {
    //             // if alpha < standpat {
    //             //     alpha = standpat;
    //             // }
    //             // if standpat >= beta {
    //             //     return Ok(standpat);
    //             // }
    //             let mut bestscore = standpat; // valid by null move observation, not as valid in very late game
    //             let mut moves = board_data
    //                 .get_moves_data_mut()
    //                 .iter_mut()
    //                 .collect::<Vec<_>>();
    //             moves.sort_by_key(|mv| -mv.get_approx_score());
    //             for move_data in &mut moves {
    //                 let m = move_data.mv;
    //                 if let Some(material_gain) = match m {
    //                     Move::Standard { victim: None, .. } => None,
    //                     Move::Standard {
    //                         victim: Some(victim),
    //                         ..
    //                     } => Some(victim.kind.worth().unwrap() * 1000),
    //                 } {
    //                     if standpat.add_heuristic(material_gain + 200) < alpha {
    //                         //delta prune
    //                         // println!("delta prune {:?}", deep);
    //                         continue;
    //                     }
    //                     // let og_board = self.board.clone();
    //                     board.make_move(m);
    //                     if let Ok(score) = move_data.quiesce(
    //                         stop_check,
    //                         board,
    //                         depth + 1,
    //                         max_quiesce_depth,
    //                         -beta,
    //                         -alpha,
    //                     ) {
    //                         board.unmake_move();
    //                         // debug_assert_eq!(self.board, &og_board);
    //                         if score > bestscore {
    //                             bestscore = score;
    //                             if score > alpha {
    //                                 alpha = score;
    //                             }
    //                         }
    //                         if score >= beta {
    //                             break; //the opponent already has a method to avoid us getting this score here, so we can stop looking
    //                         }
    //                     } else {
    //                         board.unmake_move();
    //                         return Err(());
    //                     }
    //                     // }
    //                 }
    //             }
    //             bestscore
    //         }
    //     };
    //     self.score = Some(ApproxScore { score });
    //     Ok(score)
    // }
}

impl BoardData {
    fn alpha_beta(
        &mut self,
        stop_check: &impl Fn() -> bool,
        board: &mut Board,
        depth: usize,
        max_depth: usize,
        max_quiesce_depth: usize,
        mut alpha: LowerBound,
        beta: UpperBound,
    ) -> Result<AlphaBetaMaximizingResult, ()> {
        if stop_check() {
            return Err(());
        }

        //alpha is the score we already know we can achive
        //beta is the score the opponent knows they can achive
        if self.is_terminal() {
            //no moves, return whether it is a win, draw, or loss
            Ok(AlphaBetaMaximizingResult {
                score: self.get_evaluation(),
                depth: max_depth - depth,
                exact: true,
            })
        } else if depth >= max_depth {
            //TODO: quiesce
            Ok(AlphaBetaMaximizingResult {
                score: self.get_evaluation(),
                depth: max_depth - depth,
                exact: true,
            })
        } else {
            macro_rules! get_score_and_beta_prune {
                ($move_data:expr) => {{
                    let score = $move_data
                        .alpha_beta(
                            stop_check,
                            board,
                            depth + 1,
                            max_depth,
                            max_quiesce_depth,
                            alpha,
                            beta,
                        )?
                        .score;
                    if !beta.is_improvement(&score) {
                        //beta prune
                        return Ok(AlphaBetaMaximizingResult {
                            score: score,
                            depth: max_depth - depth,
                            exact: false,
                        });
                    }
                    score
                }};
            }

            let mut moves = self.get_moves_data_mut().iter_mut().collect::<Vec<_>>();
            moves.sort_by_key(|mv| mv.get_approx_score());
            debug_assert!(!moves.is_empty());
            let first_move_data = moves.pop().unwrap();
            let mut bestscore = get_score_and_beta_prune!(first_move_data);
            for move_data in moves.into_iter().rev() {
                let score = get_score_and_beta_prune!(move_data);
                if score > bestscore {
                    bestscore = score;
                    if alpha.is_improvement(&score) {
                        alpha = LowerBound::Finite(score);
                    }
                }
            }
            Ok(AlphaBetaMaximizingResult {
                score: bestscore,
                depth: max_depth - depth,
                exact: true,
            })
        }
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

    fn best_move_at_depth(
        &mut self,
        max_depth: usize,
        max_quiesce_depth: usize,
        stop_check: impl Fn() -> bool,
    ) -> Result<(Option<MoveIdx>, LowerBound), ()> {
        let mut moves = self
            .root
            .get_moves_data_mut()
            .iter_mut()
            .enumerate()
            .map(|(idx, mv)| (MoveIdx { idx }, mv))
            .collect::<Vec<_>>();
        moves.sort_by_key(|(mv_idx, mv)| mv.get_approx_score());
        let n = moves.len();
        if n == 0 {
            Ok((None, LowerBound::NegInf))
        } else {
            let (mut best_move_idx, first_move_data) = moves.pop().unwrap();
            let mut bestscore = first_move_data
                .alpha_beta(
                    &stop_check,
                    &mut self.board,
                    0,
                    max_depth,
                    max_quiesce_depth,
                    LowerBound::NegInf,
                    UpperBound::PosInf,
                )?
                .score;
            println!("  1/{:?}", n);
            for (idx, (move_idx, move_data)) in moves.into_iter().rev().enumerate() {
                let score = move_data
                    .alpha_beta(
                        &stop_check,
                        &mut self.board,
                        0,
                        max_depth,
                        max_quiesce_depth,
                        LowerBound::Finite(bestscore),
                        UpperBound::PosInf,
                    )?
                    .score;
                println!("  {:?}/{:?}", idx + 2, n);
                if score > bestscore {
                    bestscore = score;
                    best_move_idx = move_idx;
                }
            }
            Ok((Some(best_move_idx), LowerBound::Finite(bestscore)))
        }
    }

    pub fn make_move(&mut self, m: MoveIdx) {
        let md = self.root.get_move_mut(m);
        self.board.make_move(md.mv);
        self.root = match &md.board {
            Some(board) => board.clone(),
            None => md.get_board(&mut self.board).clone(),
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
        let mut depth = 1;
        println!("start");
        loop {
            match tree.best_move_at_depth(depth - 1, depth * 2 - 1, stop_check) {
                Ok((best_move_answer, score)) => {
                    *best_move.lock().unwrap() = best_move_answer;
                    println!("done at depth = {:?} with score = {:?}", depth, score);
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
        self.tree.root.get_moves()
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
