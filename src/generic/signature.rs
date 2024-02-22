use super::*;

#[derive(Debug, Clone)]
pub struct CastleSignature {
    pub king_from: Square,
    pub king_to: Square,
    pub rook_from: Square,
    pub rook_to: Square,
    pub not_chcked: Vec<Square>, //in-between places where there cannot be check
    pub not_occupied: Vec<Square>, //in-between places which must be empty
}

#[derive(Debug, Clone)]
pub struct Signature {
    num: usize,
    flat_slides: Vec<Vec<Vec<Square>>>,
    diag_slides: Vec<Vec<Vec<Square>>>,
    knight_moves: Vec<Vec<Square>>,
    king_moves: Vec<Vec<Square>>,
    //for each square there is a list of tuples (m1, [m2, ..., m2]) where m1 is a single pawn move and m2, ..., m2 is a list of follow up pawn moves
    white_pawn_moves: Vec<Vec<(Square, Vec<Square>)>>,
    black_pawn_moves: Vec<Vec<(Square, Vec<Square>)>>,
    white_pawn_takes: Vec<Vec<Square>>,
    black_pawn_takes: Vec<Vec<Square>>,
    white_pawn_promotions: Vec<Option<Vec<PieceKind>>>,
    black_pawn_promotions: Vec<Option<Vec<PieceKind>>>,
    white_pawn_promotion_distance: Vec<Option<usize>>,
    black_pawn_promotion_distance: Vec<Option<usize>>,
    white_castles: Vec<CastleSignature>,
    black_castles: Vec<CastleSignature>,
}

impl Signature {
    pub fn num(&self) -> usize {
        self.num
    }

    pub fn get_castles(&self) -> Vec<(Team, &CastleSignature)> {
        self.white_castles
            .iter()
            .map(|cs| (Team::White, cs))
            .chain(self.black_castles.iter().map(|cs| (Team::Black, cs)))
            .collect()
    }

    pub fn get_pawn_promotion_distance(&self, sq: Square, team: Team) -> Option<usize> {
        match team {
            Team::White => self.white_pawn_promotion_distance[sq.idx],
            Team::Black => self.black_pawn_promotion_distance[sq.idx],
        }
    }

    pub fn get_pawn_promotions(&self, sq: Square, team: Team) -> &Option<Vec<PieceKind>> {
        match team {
            Team::White => &self.white_pawn_promotions[sq.idx],
            Team::Black => &self.black_pawn_promotions[sq.idx],
        }
    }

    pub fn get_pawn_moves(&self, sq: Square, team: Team) -> &Vec<(Square, Vec<Square>)> {
        match team {
            Team::White => &self.white_pawn_moves[sq.idx],
            Team::Black => &self.black_pawn_moves[sq.idx],
        }
    }

    pub fn get_pawn_takes(&self, sq: Square, team: Team) -> &Vec<Square> {
        match team {
            Team::White => &self.white_pawn_takes[sq.idx],
            Team::Black => &self.black_pawn_takes[sq.idx],
        }
    }

    pub fn get_flat_slides(&self, sq: Square) -> &Vec<Vec<Square>> {
        &self.flat_slides[sq.idx]
    }

    pub fn get_diag_slides(&self, sq: Square) -> &Vec<Vec<Square>> {
        &self.diag_slides[sq.idx]
    }

    pub fn get_knight_moves(&self, sq: Square) -> &Vec<Square> {
        &self.knight_moves[sq.idx]
    }

    pub fn get_king_moves(&self, sq: Square) -> &Vec<Square> {
        &self.king_moves[sq.idx]
    }
}

impl Signature {
    //flat_nbs : all the immediate neighbours of idx in horz and vert directions
    //diag_nbs : all the immediate neighbours of idx in diagonal directions
    //flat_opp and diag_opp : assuming i and j represent a set from one square to anther, either horz, vert, or diag
    //return all possible (might be multiple e.g. on a wormhole board) next steps k so that i,j,k are "evenly spaced"
    //pawn_moves : return all the pawn moves. regular moves in the first list, and double moves in the second list
    pub fn new(
        num: usize,
        flat_nbs: &dyn Fn(Square) -> Vec<Square>,
        diag_nbs: &dyn Fn(Square) -> Vec<Square>,
        flat_opp: &dyn Fn(Square, Square) -> Vec<Square>,
        diag_opp: &dyn Fn(Square, Square) -> Vec<Square>,
        pawn_moves: &dyn Fn(Team, Square) -> Vec<(Square, Vec<Square>)>,
        white_pawn_promotions: HashMap<Square, Vec<PieceKind>>,
        black_pawn_promotions: HashMap<Square, Vec<PieceKind>>,
        white_castles: Vec<CastleSignature>,
        black_castles: Vec<CastleSignature>,
    ) -> Self {
        //given a flat move from i to j, what are the possible following orthogonal flat moves
        let flat_nopp = |i: Square, j: Square| -> Vec<Square> {
            let opps = flat_opp(i, j);
            let mut ans: Vec<Square> = vec![];
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
        let gen_slides = |idx: Square,
                          nbs: &dyn Fn(Square) -> Vec<Square>,
                          opp: &dyn Fn(Square, Square) -> Vec<Square>|
         -> Vec<Vec<Square>> {
            struct RestSlide<'a> {
                f: &'a dyn Fn(&RestSlide, Vec<Square>, Square, Square) -> Vec<Vec<Square>>,
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
                                    let mut tmp: Vec<Square> = vec![j];
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

        let flat_slides: Vec<Vec<Vec<Square>>> = (0..num)
            .map(|idx| gen_slides(Square { idx }, flat_nbs, flat_opp))
            .collect();

        let diag_slides: Vec<Vec<Vec<Square>>> = (0..num)
            .map(|idx| gen_slides(Square { idx }, diag_nbs, diag_opp))
            .collect();

        let knight_moves: Vec<Vec<Square>> = (0..num)
            .map(|a_idx| {
                let a = Square { idx: a_idx };
                let mut ans: HashSet<Square> = HashSet::new();

                //flat 2 side 1
                for b in flat_nbs(a) {
                    for c in flat_opp(a, b) {
                        for d in flat_nopp(b, c) {
                            ans.insert(d);
                        }
                    }
                }

                //side 1 flat 2
                for b in flat_nbs(a) {
                    for c in flat_nopp(a, b) {
                        for d in flat_opp(b, c) {
                            ans.insert(d);
                        }
                    }
                }
                ans.into_iter().collect()
            })
            .collect();

        let king_moves: Vec<Vec<Square>> = (0..num)
            .map(|a_idx| {
                let a = Square { idx: a_idx };
                let mut ans: HashSet<Square> = HashSet::new();
                for b in flat_nbs(a) {
                    ans.insert(b);
                }
                for b in diag_nbs(a) {
                    ans.insert(b);
                }
                ans.into_iter().collect()
            })
            .collect();

        let pawn_attacks = |team, sq| -> Vec<Square> {
            let mut sqs = vec![];
            for (m1, m2s) in pawn_moves(team, sq) {
                for a in flat_nopp(sq, m1) {
                    sqs.push(a);
                }
            }
            sqs
        };

        let compute_pawn_promotion_distance = |pawn_moves: HashMap<Square, Vec<Square>>,
                                               promotion_squares: HashSet<Square>|
         -> Vec<Option<usize>> {
            (0..num)
                .map(|idx| Square { idx })
                .map(|sq| {
                    let mut frontier = HashSet::from([sq]);
                    let mut distance = 0;
                    while !frontier
                        .iter()
                        .any(|frontier_sq| promotion_squares.contains(frontier_sq))
                    {
                        if frontier.is_empty() {
                            return None;
                        }
                        let mut new_frontier = HashSet::new();
                        for frontier_sq in frontier {
                            for new_frontier_sq in pawn_moves.get(&frontier_sq).unwrap() {
                                new_frontier.insert(*new_frontier_sq);
                            }
                        }
                        frontier = new_frontier;
                        distance += 1;
                    }
                    Some(distance)
                })
                .collect()
        };

        let white_pawn_promotion_distance = compute_pawn_promotion_distance(
            (0..num)
                .map(|idx| Square { idx })
                .map(|sq| {
                    (sq, {
                        pawn_moves(Team::White, sq)
                            .into_iter()
                            .map(|(first, seconds)| first)
                            .collect::<Vec<_>>()
                    })
                })
                .collect(),
            white_pawn_promotions.keys().cloned().collect(),
        );
        let black_pawn_promotion_distance = compute_pawn_promotion_distance(
            (0..num)
                .map(|idx| Square { idx })
                .map(|sq| {
                    (sq, {
                        pawn_moves(Team::Black, sq)
                            .into_iter()
                            .map(|(first, seconds)| first)
                            .collect::<Vec<_>>()
                    })
                })
                .collect(),
            black_pawn_promotions.keys().cloned().collect(),
        );

        Self {
            num,
            flat_slides,
            diag_slides,
            knight_moves,
            king_moves,
            white_pawn_moves: (0..num)
                .map(|idx| pawn_moves(Team::White, Square { idx }))
                .collect(),
            black_pawn_moves: (0..num)
                .map(|idx| pawn_moves(Team::Black, Square { idx }))
                .collect(),
            white_pawn_takes: (0..num)
                .map(|idx| pawn_attacks(Team::White, Square { idx }))
                .collect(),
            black_pawn_takes: (0..num)
                .map(|idx| pawn_attacks(Team::Black, Square { idx }))
                .collect(),
            white_pawn_promotions: (0..num)
                .map(|idx| {
                    let ans = white_pawn_promotions.get(&Square { idx }).cloned();
                    ans
                })
                .collect(),
            black_pawn_promotions: (0..num)
                .map(|idx| {
                    let ans = black_pawn_promotions.get(&Square { idx }).cloned();
                    ans
                })
                .collect(),
            white_pawn_promotion_distance,
            black_pawn_promotion_distance,
            white_castles,
            black_castles,
        }
    }

    pub fn get_num(&self) -> usize {
        self.num
    }
}
