use std::{
    ops::Neg,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Heuristic(i64),
    Lost(usize),
    Draw(usize),
    Won(usize),
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Score::Lost(a), Score::Lost(b)) => a.cmp(b),
            (Score::Won(a), Score::Won(b)) => a.cmp(b).reverse(),
            (Score::Lost(_), _) => std::cmp::Ordering::Less,
            (Score::Won(_), _) => std::cmp::Ordering::Greater,
            (_, Score::Lost(_)) => std::cmp::Ordering::Greater,
            (_, Score::Won(_)) => std::cmp::Ordering::Less,
            (Score::Heuristic(a), Score::Heuristic(b)) => a.cmp(b),
            (Score::Heuristic(a), Score::Draw(_)) => a.cmp(&0),
            (Score::Draw(_), Score::Heuristic(b)) => 0.cmp(b),
            (Score::Draw(a), Score::Draw(b)) => a.cmp(b), //I guess drawing eariler is better than drawing later because you wasted less of your life
        }
    }
}

impl Neg for Score {
    type Output = Score;

    fn neg(self) -> Self::Output {
        match self {
            Score::Heuristic(v) => Score::Heuristic(-v),
            Score::Lost(n) => Score::Won(n),
            Score::Draw(n) => Score::Draw(n),
            Score::Won(n) => Score::Lost(n),
        }
    }
}

impl Score {
    pub fn add_heuristic(self, offset: i64) -> Self {
        match self {
            Score::Heuristic(v) => Score::Heuristic(v + offset),
            _ => self,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LowerBound {
    Finite(Score),
    NegInf,
}

impl Neg for LowerBound {
    type Output = UpperBound;

    fn neg(self) -> Self::Output {
        match self {
            LowerBound::Finite(v) => UpperBound::Finite(-v),
            LowerBound::NegInf => UpperBound::PosInf,
        }
    }
}

impl PartialOrd for LowerBound {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LowerBound {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (LowerBound::Finite(a), LowerBound::Finite(b)) => a.cmp(b),
            (LowerBound::Finite(_), LowerBound::NegInf) => std::cmp::Ordering::Greater,
            (LowerBound::NegInf, LowerBound::Finite(_)) => std::cmp::Ordering::Less,
            (LowerBound::NegInf, LowerBound::NegInf) => std::cmp::Ordering::Equal,
        }
    }
}

impl LowerBound {
    pub fn is_improvement(&self, score: &Score) -> bool {
        match self {
            LowerBound::Finite(lb) => lb <= score,
            LowerBound::NegInf => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpperBound {
    Finite(Score),
    PosInf,
}

impl Neg for UpperBound {
    type Output = LowerBound;

    fn neg(self) -> Self::Output {
        match self {
            UpperBound::Finite(v) => LowerBound::Finite(-v),
            UpperBound::PosInf => LowerBound::NegInf,
        }
    }
}

impl PartialOrd for UpperBound {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UpperBound {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (UpperBound::Finite(a), UpperBound::Finite(b)) => a.cmp(b),
            (UpperBound::Finite(_), UpperBound::PosInf) => std::cmp::Ordering::Less,
            (UpperBound::PosInf, UpperBound::Finite(_)) => std::cmp::Ordering::Greater,
            (UpperBound::PosInf, UpperBound::PosInf) => std::cmp::Ordering::Equal,
        }
    }
}

impl UpperBound {
    pub fn is_improvement(&self, score: &Score) -> bool {
        match self {
            UpperBound::Finite(ub) => score <= ub,
            UpperBound::PosInf => true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LowerBoundRef {
    values: Vec<Arc<Mutex<UpperBound>>>, //we are the maximum of these upper bounds
}
impl Neg for LowerBoundRef {
    type Output = UpperBoundRef;

    fn neg(self) -> Self::Output {
        UpperBoundRef {
            values: self.values,
        }
    }
}
impl LowerBoundRef {
    pub fn new_inf() -> Self {
        Self {
            values: vec![Arc::new(Mutex::new(UpperBound::PosInf))],
        }
    }
    pub fn get_bound(&self) -> LowerBound {
        -self
            .values
            .iter()
            .map(|value| *value.lock().unwrap())
            .min()
            .unwrap()
    }
    pub fn refine_bound(&self, score: Score) -> bool {
        let mut bound = self.values.last().unwrap().lock().unwrap();
        let bound_neg = -*bound;
        if bound_neg.is_improvement(&score) {
            *bound = -LowerBound::Finite(score);
            true
        } else {
            false
        }
    }
    pub fn branch(&self) -> Self {
        let mut branch = self.clone();
        branch.values.push(Arc::new(Mutex::new(UpperBound::PosInf)));
        branch
    }
}

#[derive(Debug, Clone)]
pub struct UpperBoundRef {
    values: Vec<Arc<Mutex<UpperBound>>>, //we are the minimum of these lower bounds represented as negative upper bounds
}
impl Neg for UpperBoundRef {
    type Output = LowerBoundRef;

    fn neg(self) -> Self::Output {
        LowerBoundRef {
            values: self.values,
        }
    }
}
impl UpperBoundRef {
    pub fn new_inf() -> Self {
        Self {
            values: vec![Arc::new(Mutex::new(UpperBound::PosInf))],
        }
    }
    pub fn get_bound(&self) -> UpperBound {
        self.values
            .iter()
            .map(|value| *value.lock().unwrap())
            .min()
            .unwrap()
    }
    pub fn refine_bound(&self, score: Score) -> bool {
        let mut bound = self.values.last().unwrap().lock().unwrap();
        if bound.is_improvement(&score) {
            *bound = UpperBound::Finite(score);
            true
        } else {
            false
        }
    }
    pub fn branch(&self) -> Self {
        let mut branch = self.clone();
        branch.values.push(Arc::new(Mutex::new(UpperBound::PosInf)));
        branch
    }
}
