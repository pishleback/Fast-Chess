use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Heuristic(i64),
    Lost(usize),
    Draw(usize),
    Won(usize),
    // NegInf,
    // PosInf,
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            // (Score::NegInf, Score::NegInf) => std::cmp::Ordering::Equal,
            // (Score::PosInf, Score::PosInf) => std::cmp::Ordering::Equal,
            // (Score::NegInf, _) => std::cmp::Ordering::Less,
            // (Score::PosInf, _) => std::cmp::Ordering::Greater,
            // (_, Score::NegInf) => std::cmp::Ordering::Greater,
            // (_, Score::PosInf) => std::cmp::Ordering::Less,
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
            // Score::NegInf => Score::PosInf,
            // Score::PosInf => Score::NegInf,
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
            (LowerBound::Finite(a), LowerBound::NegInf) => std::cmp::Ordering::Greater,
            (LowerBound::NegInf, LowerBound::Finite(b)) => std::cmp::Ordering::Less,
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
            (UpperBound::Finite(a), UpperBound::PosInf) => std::cmp::Ordering::Less,
            (UpperBound::PosInf, UpperBound::Finite(b)) => std::cmp::Ordering::Greater,
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
