use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Score {
    Heuristic(i64),
    Lost(usize),
    Draw(usize),
    Won(usize),
    NegInf,
    PosInf,
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Score {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Score::NegInf, Score::NegInf) => std::cmp::Ordering::Equal,
            (Score::PosInf, Score::PosInf) => std::cmp::Ordering::Equal,
            (Score::NegInf, _) => std::cmp::Ordering::Less,
            (Score::PosInf, _) => std::cmp::Ordering::Greater,
            (_, Score::NegInf) => std::cmp::Ordering::Greater,
            (_, Score::PosInf) => std::cmp::Ordering::Less,
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
            Score::NegInf => Score::PosInf,
            Score::PosInf => Score::NegInf,
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

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Score {
//     Finite(i64),
//     PosInf,
//     NegInf,
// }

// impl PartialOrd for Score {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }
// impl Ord for Score {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         match (self, other) {
//             (Score::NegInf, _) => std::cmp::Ordering::Less,
//             (Score::PosInf, _) => std::cmp::Ordering::Greater,
//             (_, Score::NegInf) => std::cmp::Ordering::Greater,
//             (_, Score::PosInf) => std::cmp::Ordering::Less,
//             (Score::Finite(a), Score::Finite(b)) => a.cmp(b),
//         }
//     }
// }
// impl AddAssign for Score {
//     fn add_assign(mut self: &mut Self, rhs: Self) {
//         match (&mut self, rhs) {
//             (Score::Finite(a), Score::Finite(b)) => a.add_assign(b),
//             (Score::Finite(a), Score::PosInf) => *self = Score::PosInf,
//             (Score::Finite(a), Score::NegInf) => *self = Score::NegInf,
//             (Score::PosInf, Score::Finite(b)) => {}
//             (Score::PosInf, Score::PosInf) => {}
//             (Score::PosInf, Score::NegInf) => panic!(),
//             (Score::NegInf, Score::Finite(b)) => {}
//             (Score::NegInf, Score::PosInf) => panic!(),
//             (Score::NegInf, Score::NegInf) => {}
//         }
//     }
// }
// impl SubAssign for Score {
//     fn sub_assign(mut self: &mut Self, rhs: Self) {
//         match (&mut self, rhs) {
//             (Score::Finite(a), Score::Finite(b)) => a.sub_assign(b),
//             (Score::Finite(a), Score::PosInf) => *self = Score::NegInf,
//             (Score::Finite(a), Score::NegInf) => *self = Score::PosInf,
//             (Score::PosInf, Score::Finite(b)) => {}
//             (Score::PosInf, Score::PosInf) => panic!(),
//             (Score::PosInf, Score::NegInf) => {}
//             (Score::NegInf, Score::Finite(b)) => {}
//             (Score::NegInf, Score::PosInf) => {}
//             (Score::NegInf, Score::NegInf) => panic!(),
//         }
//     }
// }
// impl Neg for Score {
//     type Output = Score;

//     fn neg(self) -> Self::Output {
//         match self {
//             Score::Finite(a) => Score::Finite(-a),
//             Score::PosInf => Score::NegInf,
//             Score::NegInf => Score::PosInf,
//         }
//     }
// }
// impl Add<Score> for Score {
//     type Output = Score;

//     fn add(self, rhs: Score) -> Self::Output {
//         match (self, rhs) {
//             (Score::Finite(a), Score::Finite(b)) => Score::Finite(a + b),
//             (Score::Finite(a), Score::PosInf) => Score::PosInf,
//             (Score::Finite(a), Score::NegInf) => Score::NegInf,
//             (Score::PosInf, Score::Finite(b)) => Score::PosInf,
//             (Score::PosInf, Score::PosInf) => Score::PosInf,
//             (Score::PosInf, Score::NegInf) => panic!(),
//             (Score::NegInf, Score::Finite(b)) => Score::NegInf,
//             (Score::NegInf, Score::PosInf) => panic!(),
//             (Score::NegInf, Score::NegInf) => Score::NegInf,
//         }
//     }
// }
// impl Sub<Score> for Score {
//     type Output = Score;

//     fn sub(self, rhs: Score) -> Self::Output {
//         match (self, rhs) {
//             (Score::Finite(a), Score::Finite(b)) => Score::Finite(a - b),
//             (Score::Finite(a), Score::PosInf) => Score::NegInf,
//             (Score::Finite(a), Score::NegInf) => Score::PosInf,
//             (Score::PosInf, Score::Finite(b)) => Score::PosInf,
//             (Score::PosInf, Score::PosInf) => panic!(),
//             (Score::PosInf, Score::NegInf) => Score::PosInf,
//             (Score::NegInf, Score::Finite(b)) => Score::NegInf,
//             (Score::NegInf, Score::PosInf) => Score::NegInf,
//             (Score::NegInf, Score::NegInf) => panic!(),
//         }
//     }
// }
// impl Mul<i64> for Score {
//     type Output = Score;

//     fn mul(self, rhs: i64) -> Self::Output {
//         assert!(rhs > 0);
//         match self {
//             Score::Finite(a) => match a.checked_mul(rhs) {
//                 Some(ans) => Score::Finite(ans),
//                 None => match a.cmp(&0) {
//                     std::cmp::Ordering::Less => Score::NegInf,
//                     std::cmp::Ordering::Equal => Score::Finite(0),
//                     std::cmp::Ordering::Greater => Score::PosInf,
//                 },
//             },
//             Score::PosInf => Score::PosInf,
//             Score::NegInf => Score::NegInf,
//         }
//     }
// }
