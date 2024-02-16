use std::cell::RefCell;
use std::os::windows::thread;
use std::sync::{Arc, Mutex};
use std::thread::{sleep_ms, JoinHandle, Thread};
use std::time::Duration;

use super::board_data::*;
use super::score::*;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApproxScore {
    score: Score,
}

#[derive(Debug, Clone)]
pub struct MoveData {
    mv: Move,
    board: Option<BoardData>,
    score: Option<ApproxScore>,
}

impl MoveData {
    pub fn new(mv: Move) -> Self {
        Self {
            mv,
            board: None,
            score: None,
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
        match &self.score {
            Some(approx_score) => approx_score.score,
            None => Score::Heuristic(0),
        }
    }

    fn compute_score(
        &mut self,
        stop_check: &impl Fn() -> bool,
        board: &mut Board,
        max_depth: usize,
        max_quiesce_depth: usize,
    ) -> Result<Score, ()> {
        let mut node_count = 0;
        self.alpha_beta(
            stop_check,
            board,
            0,
            max_depth,
            max_quiesce_depth,
            Score::NegInf,
            Score::PosInf,
            &mut node_count,
        )
    }

    fn alpha_beta(
        &mut self,
        stop_check: &impl Fn() -> bool,
        board: &mut Board,
        depth: usize,
        max_depth: usize,
        max_quiesce_depth: usize,
        mut alpha: Score,
        beta: Score,
        node_count: &mut usize,
    ) -> Result<Score, ()> {
        *node_count += 1;

        if stop_check() {
            return Err(());
        }

        //alpha is the score we already know we can achive
        //beta is the score the opponent knows they can achive
        let board_data = self.get_board(board);
        if board_data.is_terminal() {
            return Ok(board_data.get_evaluation());
        }

        let score = -{
            if depth >= max_depth {
                -self.quiesce(stop_check, board, depth + 1, max_quiesce_depth, alpha, beta)?
                // match board.get_turn() {
                //     Team::White => board_data.info.raw_score(),
                //     Team::Black => -board_data.info.raw_score(),
                // }
                // board_data.get_evaluation()
            } else {
                let mut bestscore = Score::NegInf;
                let mut moves = board_data
                    .get_moves_data_mut()
                    .iter_mut()
                    .collect::<Vec<_>>();
                moves.sort_by_key(|mv| -mv.get_approx_score());
                for move_data in &mut moves {
                    let m = move_data.mv;
                    // let og_board = self.board.clone();
                    board.make_move(m);
                    if let Ok(score) = move_data.alpha_beta(
                        stop_check,
                        board,
                        depth + 1,
                        max_depth,
                        max_quiesce_depth,
                        -beta,
                        -alpha,
                        node_count,
                    ) {
                        board.unmake_move();
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
                        board.unmake_move();
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
        depth: usize,
        max_quiesce_depth: usize,
        mut alpha: Score,
        beta: Score,
    ) -> Result<Score, ()> {
        if stop_check() {
            return Err(());
        }

        //alpha is the score we already know we can achive
        //beta is the score the opponent knows they can achive
        let board_data = self.get_board(board);
        if board_data.is_terminal() {
            return Ok(-board_data.get_evaluation());
        }

        let score = -{
            let standpat = board_data.get_evaluation();
            if depth >= max_quiesce_depth {
                standpat
            } else {
                // if alpha < standpat {
                //     alpha = standpat;
                // }
                // if standpat >= beta {
                //     return Ok(standpat);
                // }
                let mut bestscore = standpat; // valid by null move observation, not as valid in very late game
                let mut moves = board_data
                    .get_moves_data_mut()
                    .iter_mut()
                    .collect::<Vec<_>>();
                moves.sort_by_key(|mv| -mv.get_approx_score());
                for move_data in &mut moves {
                    let m = move_data.mv;
                    if let Some(material_gain) = match m {
                        Move::Standard { victim: None, .. } => None,
                        Move::Standard {
                            victim: Some(victim),
                            ..
                        } => Some(victim.kind.worth().unwrap() * 1000),
                    } {
                        if standpat.add_heuristic(material_gain + 200) < alpha {
                            //delta prune
                            // println!("delta prune {:?}", deep);
                            continue;
                        }
                        // let og_board = self.board.clone();
                        board.make_move(m);
                        if let Ok(score) = move_data.quiesce(
                            stop_check,
                            board,
                            depth + 1,
                            max_quiesce_depth,
                            -beta,
                            -alpha,
                        ) {
                            board.unmake_move();
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
                            board.unmake_move();
                            return Err(());
                        }
                        // }
                    }
                }
                bestscore
            }
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
        max_depth: usize,
        max_quiesce_depth: usize,
        stop_check: impl Fn() -> bool,
    ) -> Result<(Option<MoveIdx>, Score), ()> {
        let mut best_move_idx = None;
        let mut highest_score = Score::NegInf;
        let n = self.root.get_moves_data().len();
        for idx in 0..n {
            let move_idx = MoveIdx { idx };
            let move_data = &mut self.root.get_move_mut(move_idx);
            self.board.make_move(move_data.mv);
            if let Ok(score) =
                move_data.compute_score(&stop_check, &mut self.board, max_depth, max_quiesce_depth)
            {
                self.board.unmake_move();
                if score > highest_score {
                    highest_score = score;
                    best_move_idx = Some(move_idx);
                }
            } else {
                self.board.unmake_move();
                return Err(());
            }
            println!("  {:?}/{:?}", idx + 1, n);
        }
        Ok((best_move_idx, highest_score))
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
