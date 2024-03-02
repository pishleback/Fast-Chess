use std::ops::Neg;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

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
    depth: isize,
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
    depth: isize,
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

    pub fn get_move(&self) -> &Move {
        &self.mv
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
        node_count: Arc<Mutex<usize>>,
        board: &mut Board,
        depth: usize,
        max_depth: usize,
        max_quiesce_depth: usize,
        max_node_count: usize,
        alpha: LowerBoundRef,
        beta: UpperBoundRef,
    ) -> Result<AlphaBetaMinimizingResult, ()> {
        {
            let mut node_count_value = node_count.lock().unwrap();
            *node_count_value += 1;
            if *node_count_value > max_node_count {
                return Err(());
            }
        }

        board.make_move(self.mv.clone());
        let board_data = self.get_board(board);
        if let Ok(abres) = board_data.alpha_beta(
            stop_check,
            node_count,
            board,
            depth,
            max_depth,
            max_quiesce_depth,
            max_node_count,
            -beta,
            -alpha,
        ) {
            board.unmake_move().unwrap();
            self.approx_score = Some(-abres);
            Ok(-abres)
        } else {
            board.unmake_move().unwrap();
            Err(())
        }
    }
}

//alpha is the score we already know we can achive
//beta is the score the opponent knows they can achive
impl BoardData {
    fn alpha_beta(
        &mut self,
        stop_check: &impl Fn() -> bool,
        node_count: Arc<Mutex<usize>>,
        board: &mut Board,
        depth: usize,
        max_depth: usize,
        max_quiesce_depth: usize,
        max_node_count: usize,
        alpha: LowerBoundRef,
        beta: UpperBoundRef,
    ) -> Result<AlphaBetaMaximizingResult, ()> {
        if stop_check() {
            return Err(());
        }

        if depth >= max_depth {
            return self.quiescence(
                stop_check,
                node_count,
                board,
                depth,
                max_depth,
                max_quiesce_depth,
                max_node_count,
                alpha,
                beta,
            );
        }

        let eval = self.get_evaluation();
        match eval {
            Score::Lost(_) | Score::Draw(_) | Score::Won(_) => {
                //board is terminal
                Ok(AlphaBetaMaximizingResult {
                    score: eval,
                    depth: max_depth as isize - depth as isize,
                    exact: true,
                })
            }
            Score::Heuristic(_stand_pat) => {
                macro_rules! get_score_and_beta_prune {
                    ($move_data:expr) => {{
                        let score = $move_data
                            .alpha_beta(
                                stop_check,
                                node_count.clone(),
                                board,
                                depth + 1,
                                max_depth,
                                max_quiesce_depth,
                                max_node_count,
                                alpha.clone(),
                                beta.branch(),
                            )?
                            .score;
                        alpha.refine_bound(score);
                        if !beta.get_bound().is_improvement(&score) {
                            //beta prune
                            return Ok(AlphaBetaMaximizingResult {
                                score: score,
                                depth: max_depth as isize - depth as isize,
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
                    }
                }
                Ok(AlphaBetaMaximizingResult {
                    score: bestscore,
                    depth: max_depth as isize - depth as isize,
                    exact: true,
                })
            }
        }
    }

    fn quiescence(
        &mut self,
        stop_check: &impl Fn() -> bool,
        node_count: Arc<Mutex<usize>>,
        board: &mut Board,
        depth: usize,
        max_depth: usize,
        max_quiesce_depth: usize,
        max_node_count: usize,
        alpha: LowerBoundRef,
        beta: UpperBoundRef,
    ) -> Result<AlphaBetaMaximizingResult, ()> {
        if stop_check() {
            return Err(());
        }

        {
            let mut node_count_value = node_count.lock().unwrap();
            *node_count_value += 1;
            if *node_count_value > max_node_count {
                return Err(());
            }
        }

        let eval = self.get_evaluation();
        if depth >= max_quiesce_depth {
            return Ok(AlphaBetaMaximizingResult {
                score: eval,
                depth: max_depth as isize - depth as isize,
                exact: true,
            });
        }
        match eval {
            Score::Lost(_) | Score::Draw(_) | Score::Won(_) => {
                //board is terminal
                Ok(AlphaBetaMaximizingResult {
                    score: eval,
                    depth: max_depth as isize - depth as isize,
                    exact: true,
                })
            }
            Score::Heuristic(stand_pat) => {
                macro_rules! get_score_and_beta_prune {
                    ($move_data:expr) => {{
                        let score = $move_data
                            .alpha_beta(
                                stop_check,
                                node_count.clone(),
                                board,
                                depth + 1,
                                max_depth,
                                max_quiesce_depth,
                                max_node_count,
                                alpha.clone(),
                                beta.branch(),
                            )?
                            .score;
                        alpha.refine_bound(score);
                        if !beta.get_bound().is_improvement(&score) {
                            //beta prune
                            return Ok(AlphaBetaMaximizingResult {
                                score: score,
                                depth: max_depth as isize - depth as isize,
                                exact: false,
                            });
                        }
                        score
                    }};
                }

                let mut bestscore = Score::Heuristic(stand_pat);
                if !beta.get_bound().is_improvement(&bestscore) {
                    //beta prune the stand_pat
                    return Ok(AlphaBetaMaximizingResult {
                        score: bestscore,
                        depth: max_depth as isize - depth as isize,
                        exact: false,
                    });
                }

                let mut moves = self
                    .get_moves_data_mut()
                    .iter_mut()
                    .filter(|move_data| match &move_data.mv {
                        Move::Standard {
                            victim: victim_opt, ..
                        } => match victim_opt {
                            Some(_) => {
                                //delta prune
                                true
                                // alpha.get_bound().is_improvement(&Score::Heuristic(
                                //     stand_pat + victim.kind.worth().unwrap() * 1000 + 200, //delta prune
                                // ))
                            }
                            None => false,
                        },
                        Move::Castle { .. } => false,
                        Move::EnCroissant { .. } => false,
                    })
                    .collect::<Vec<_>>();
                moves.sort_by_key(|mv| mv.get_approx_score());
                for move_data in moves.into_iter().rev() {
                    let score = get_score_and_beta_prune!(move_data);
                    if score > bestscore {
                        bestscore = score;
                    }
                }
                Ok(AlphaBetaMaximizingResult {
                    score: bestscore,
                    depth: max_depth as isize - depth as isize,
                    exact: true,
                })
            }
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
        max_node_count: usize,
        stop_flag: Arc<Mutex<bool>>,
    ) -> Result<Option<(MoveIdx, Score, usize)>, ()> {
        let mut moves = self
            .root
            .get_moves_data_mut()
            .iter_mut()
            .enumerate()
            .map(|(idx, mv)| (MoveIdx { idx }, mv, self.board.clone()))
            .collect::<Vec<_>>();
        moves.sort_by_key(|(_mv_idx, mv, _board)| mv.get_approx_score());
        let n = moves.len();
        if n == 0 {
            Ok(None)
        } else {
            let alpha = LowerBoundRef::new_inf();
            let beta = UpperBoundRef::new_inf();
            let node_count = Arc::new(Mutex::new(0));

            use rayon::prelude::*;

            let results = moves
                .into_iter()
                .rev()
                .enumerate()
                .par_bridge() //so that moves start processing in order - to help with alpha-beta pruning
                .into_par_iter()
                .map(|(idx, (move_idx, move_data, mut board))| {
                    let stop_check = || *stop_flag.lock().unwrap();
                    match move_data.alpha_beta(
                        &stop_check,
                        node_count.clone(),
                        &mut board,
                        0,
                        max_depth,
                        max_quiesce_depth,
                        max_node_count,
                        alpha.clone(),
                        beta.branch(),
                    ) {
                        Ok(score) => {
                            let score = score.score;
                            println!("  {:?}/{:?}", idx + 1, n);
                            alpha.refine_bound(score);
                            Ok((move_idx, score))
                        }
                        Err(()) => Err(()),
                    }
                })
                .collect::<Vec<_>>();

            if results.iter().any(|result| result.is_err()) {
                Err(())
            } else {
                let scores = results
                    .into_iter()
                    .map(|result| result.unwrap())
                    .collect::<Vec<_>>();
                if let Some(best) = scores.into_iter().max_by_key(|(_move_idx, score)| *score) {
                    Ok(Some((best.0, best.1, *node_count.lock().unwrap())))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn make_move(&mut self, m: MoveIdx) {
        let md = self.root.get_move_mut(m);
        self.board.make_move(md.mv.clone());
        self.root = match &md.board {
            Some(board) => board.clone(),
            None => md.get_board(&mut self.board).clone(),
        }
    }

    pub fn unmake_move(&mut self) -> Result<(), ()> {
        match self.board.unmake_move() {
            Ok(()) => {
                self.root = BoardData::new(&mut self.board);
                Ok(())
            }
            Err(()) => Err(()),
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
        let mut depth = 1;
        println!("Search started");
        loop {
            match tree.best_move_at_depth(depth - 1, depth * 3 - 1, 3000000, stop_flag.clone()) {
                Ok(None) => {
                    println!("No moves");
                    break;
                }
                Ok(Some((best_move_answer, score, node_count))) => {
                    *best_move.lock().unwrap() = Some(best_move_answer);
                    println!(
                        "Done at depth = {:?} with score = {:?} and {:?} boards checked",
                        depth, score, node_count
                    );
                }
                Err(()) => {
                    println!("Search stopped");
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

    pub fn get_moves(&self) -> Vec<&Move> {
        self.tree.root.get_moves()
    }

    pub fn make_move(&mut self, m: MoveIdx) {
        self.tree.make_move(m);
    }

    pub fn unmake_move(&mut self) -> Result<(), ()> {
        self.tree.unmake_move()
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
