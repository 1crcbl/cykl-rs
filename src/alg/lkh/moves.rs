use crate::tour::{NodeRel, Tour, TourNode};

/// Executes the 2-opt move.
#[inline]
pub fn move_2_opt<T>(tour: &mut T, f1: &TourNode, t1: &TourNode, f2: &TourNode, t2: &TourNode)
where
    T: Tour,
{
    tour.flip(f1, t1, f2, t2);
}

/// Enum for for all 3-opt moves.
///
/// Assumes that the order of input nodes is [f1-t1]-[f2-t2]-[f3-t3], where ```tx``` is the direct
/// successor of ```fx```.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opt3Move {
    /// Equivalent to a single 2-opt move.
    ///
    /// Results in [f1-f2]-[t1-t2]-[f3-t3].
    Move1,
    /// Equivalent to a single 2-opt move.
    ///
    /// Results in [f1-t1]-[f2-f3]-[t2-t3].
    Move2,
    /// Equivalent to a single 2-opt move.
    ///
    /// Results in [f1-f3]-[t2-f2]-[t1-t3].
    Move3,
    /// Equivalent to two 2-opt moves. The first move is [`Opt3Move::Move1`].
    ///
    /// Results in [f1-f3]-[t2-t1]-[f2-t3].
    Move4,
    /// Equivalent to two 2-opt moves. The first move is [`Opt3Move::Move2`].
    ///
    /// Results in [f1-f2]-[t1-f3]-[t2-t3].
    Move5,
    /// Equivalent to two 2-opt moves. The first move is [`Opt3Move::Move3`].
    ///
    /// Results in [f1-t2]-[f3-f2]-[t1-t3].
    Move6,
    /// Equivalent to three 2-opt moves.
    ///
    /// Results in [f1-t2]-[f3-t1]-[f2-t3].
    Move7,
}

/// Executes the 3-opt move.
pub fn move_3_opt<T>(
    tour: &mut T,
    pair_1: (&TourNode, &TourNode),
    pair_2: (&TourNode, &TourNode),
    pair_3: (&TourNode, &TourNode),
    move_case: Opt3Move,
) where
    T: Tour,
{
    let (f1, t1) = pair_1;
    let (f2, t2) = pair_2;
    let (f3, t3) = pair_3;

    match move_case {
        Opt3Move::Move1 => tour.flip(f1, t1, f2, t2),
        Opt3Move::Move2 => tour.flip(f2, t2, f3, t3),
        Opt3Move::Move3 => tour.flip(f1, t1, f3, t3),
        Opt3Move::Move4 => {
            tour.flip(f1, t1, f2, t2);

            match tour.relation(f1, f2) {
                NodeRel::Predecessor => tour.flip(f1, f2, f3, t3),
                NodeRel::Successor => tour.flip(f2, f1, t3, f3),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt3Move::Move5 => {
            tour.flip(f2, t2, f3, t3);

            match tour.relation(f2, f3) {
                NodeRel::Predecessor => tour.flip(f1, t1, f2, f3),
                NodeRel::Successor => tour.flip(t1, f1, f3, f2),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt3Move::Move6 => {
            tour.flip(f1, t1, f3, t3);

            match tour.relation(f1, f3) {
                NodeRel::Predecessor => tour.flip(f1, f3, t2, f2),
                NodeRel::Successor => tour.flip(f3, f1, f2, t2),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt3Move::Move7 => {
            tour.flip(f1, t1, f2, t2);

            match tour.relation(f1, f2) {
                NodeRel::Predecessor => tour.flip(f1, f2, f3, t3),
                NodeRel::Successor => tour.flip(f2, f1, f3, t3),
                NodeRel::None => panic!("Broken tour"),
            }

            match tour.relation(f1, f3) {
                NodeRel::Predecessor => tour.flip(f1, f3, t2, t1),
                NodeRel::Successor => tour.flip(f3, f1, t1, t2),
                NodeRel::None => panic!("Broken tour"),
            }
        }
    }
}

/// Enum for purely sequential 4-opt moves.
///
/// Assumes that the order of input nodes is [f1-t1]-[f2-t2]-[f3-t3]-[f4-t4], where ```tx``` is the
/// direct successor of ```fx```.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Opt4SeqMove {
    /// A sequential 4-opt move that results in [f1-f2]-[t1-f3]-[t2-f4]-[t3-t4].
    ///
    /// This move comprises of [`Opt3Move::Move5`] and a [`move_2_opt`] acting on the last two pairs.
    Move1,
    /// A sequential 4-opt move that results in [f1-t2]-[f3-t1]-[f2-f4]-[t3-t4].
    ///
    /// This move comprises of [`Opt3Move::Move7`] and a [`move_2_opt`] acting on the last two pairs.
    Move2,
    /// A sequential 4-opt move that results in [f1-t2]-[f3-f2]-[t1-f4]-[t3-t4].
    ///
    /// This move comprises of [`Opt3Move::Move6`] and a [`move_2_opt`] acting on the last two pairs.
    Move3,
    /// A sequential 4-opt move that results in [f1-f3]-[t2-t1]-[f2-f4]-[t3-t4].
    ///
    /// This move comprises of [`Opt3Move::Move4`] and a [`move_2_opt`] acting on the first and last pairs.
    Move4,
    /// A sequential 4-opt move that results in [f1-t3]-[f4-t1]-[f2-f3]-[t2-t4].
    ///
    /// This move comprises of [`Opt4SeqMove::Move3`] and a [`move_2_opt`] acting on the last two pairs.
    Move5,
    /// A sequential 4-opt move that results in [f1-t3]-[f4-f2]-[t1-t2]-[f3-t4].
    ///
    /// This move comprises of [`Opt4SeqMove::Move4`] and a [`move_2_opt`] acting on the last two pairs.
    Move6,
    /// A sequential 4-opt move that results in [f1-f4]-[t3-t1]-[f2-f3]-[t2-t4].
    ///
    /// This move comprises of [`Opt3Move::Move6`] and a [`move_2_opt`] acting on the first and last pairs.
    Move7,
    /// A sequential 4-opt move that results in [f1-f4]-[t3-f2]-[t1-t2]-[f3-t4].
    ///
    /// This move comprises of [`Opt3Move::Move4`] and a [`move_2_opt`] acting on the first and last pairs.
    Move8,
    /// A sequential 4-opt move that results in [f1-f4]-[t3-f2]-[t1-f3]-[t2-t4].
    ///
    /// This move comprises of [`Opt3Move::Move7`] and a [`move_2_opt`] acting on the first and last pairs.
    Move9,
    /// A sequential 4-opt move that results in [f1-f2]-[t1-t3]-[f4-t2]-[f3-t4].
    ///
    /// This move comprises of [`Opt4SeqMove::Move1`] and a [`move_2_opt`] acting on the second and last pairs.
    Move10,
    /// A sequential 4-opt move that results in [f1-f2]-[t1-t3]-[f4-f3]-[t2-t4].
    ///
    /// This move comprises of [`Opt4SeqMove::Move7`] and a [`move_2_opt`] acting on the first and third pairs.
    Move11,
    /// A sequential 4-opt move that results in [f1-f2]-[t1-f4]-[t3-t2]-[f3-t4].
    ///
    /// This move comprises of [`Opt3Move::Move5`] and a [`move_2_opt`] acting on the second and last pairs.
    Move12,
    /// A sequential 4-opt move that results in [f1-t2]-[f3-f4]-[t3-t1]-[f2-t4].
    ///
    /// This move comprises of [`Opt3Move::Move6`] and a [`move_2_opt`] acting on the second and last pairs.
    Move13,
    /// A sequential 4-opt move that results in [f1-t2]-[f3-f4]-[t3-f2]-[t1-t4].
    ///
    /// This move comprises of [`Opt3Move::Move7`] and a [`move_2_opt`] acting on the second and last pairs.
    Move14,
    /// A sequential 4-opt move that results in [f1-f3]-[t2-t3]-[f4-t1]-[f2-t4].
    ///
    /// This move comprises of [`Opt4SeqMove::Move12`] and a [`move_2_opt`] acting on the first and last pairs.
    Move15,
    /// A sequential 4-opt move that results in [f1-f3]-[t2-t3]-[f4-f2]-[t1-t4].
    ///
    /// This move has the following operations:
    /// - A 2-opt move acting on the second and third pairs, which yields [f1-t1]-[f2-f3]-[t2-t3]-[f4-t4].
    /// - A 2-opt move acting on the first and last pairs, which yields [f1-f4]-[t3-t2]-[f3-f2]-[t1-t4].
    /// - A 2-opt move acting on the first and third pairs, which yields the desired outcome.
    // Note: Move16 can be derived from Move15 but that would require in total five 2-opt moves.
    Move16,
    /// A sequential 4-opt move that results in [f1-f3]-[t2-f4]-[t3-f2]-[t1-t4].
    ///
    /// This move comprises of [`Opt3Move::Move4`] and a [`move_2_opt`] acting on the second and last pairs.
    Move17,
    /// A sequential 4-opt move that results in [f1-t3]-[f4-t2]-[f3-f2]-[t1-t4].
    ///
    /// This move has the following operations:
    /// - A 2-opt move acting on the second and third pairs, which yields [f1-t1]-[f2-f3]-[t2-t3]-[f4-t4].
    /// - A 2-opt move acting on the third and last pairs, which yields [f1-t1]-[f2-f3]-[t2-f4]-[t3-t4].
    /// - A 2-opt move acting on the first and last pairs, which yields the desired outcome.
    // Note: Move18 can be derived from Move17 but that would require in total four 2-opt moves.
    Move18,
    /// A sequential 4-opt move that results in [f1-f4]-[t3-t2]-[f3-t1]-[f2-t4].
    ///
    /// This move has the following operations:
    /// - A 2-opt move acting on the second and second pairs, which yields [f1-f2]-[t1-t2]-[f3-t3]-[f4-t4].
    /// - A 2-opt move acting on the third and last pairs, which yields [f1-f2]-[t1-t2]-[f3-f4]-[t3-t4].
    /// - A 2-opt move acting on the first and last pairs, which yields the desired outcome.
    // Note: Move18 can be derived from Move18 but that would require in total four 2-opt moves.
    Move19,
    /// A sequential 4-opt move that results in [f1-t3]-[f4-f3]-[t2-t1]-[f2-t4].
    ///
    /// This move comprises of [`Opt3Move::Move5`] and a [`move_2_opt`] acting on the first and last pairs.
    Move20,
}

/// Executes a 4-opt move.
pub fn move_4_opt<T>(
    tour: &mut T,
    pair_1: (&TourNode, &TourNode),
    pair_2: (&TourNode, &TourNode),
    pair_3: (&TourNode, &TourNode),
    pair_4: (&TourNode, &TourNode),
    move_case: Opt4SeqMove,
) where
    T: Tour,
{
    let (f1, t1) = pair_1;
    let (f2, t2) = pair_2;
    let (f3, t3) = pair_3;
    let (f4, t4) = pair_4;

    match move_case {
        Opt4SeqMove::Move1 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move5);

            match tour.relation(t2, t3) {
                NodeRel::Predecessor => tour.flip(t2, t3, f4, t4),
                NodeRel::Successor => tour.flip(t3, t2, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move2 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move7);

            match tour.relation(f2, t3) {
                NodeRel::Predecessor => tour.flip(f2, t3, f4, t4),
                NodeRel::Successor => tour.flip(t3, f2, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move3 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move6);

            match tour.relation(t1, t3) {
                NodeRel::Predecessor => tour.flip(t1, t3, f4, t4),
                NodeRel::Successor => tour.flip(f3, t1, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move4 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move4);

            match tour.relation(f2, t3) {
                NodeRel::Predecessor => tour.flip(f2, t3, f4, t4),
                NodeRel::Successor => tour.flip(t3, f2, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move5 => {
            move_4_opt(tour, pair_1, pair_2, pair_3, pair_4, Opt4SeqMove::Move3);

            match tour.relation(f1, t2) {
                NodeRel::Predecessor => tour.flip(f1, t2, t3, t4),
                NodeRel::Successor => tour.flip(t2, f1, t4, t3),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move6 => {
            move_4_opt(tour, pair_1, pair_2, pair_3, pair_4, Opt4SeqMove::Move4);

            match tour.relation(f1, f3) {
                NodeRel::Predecessor => tour.flip(f1, f3, t3, t4),
                NodeRel::Successor => tour.flip(f3, f1, t4, t3),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move7 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move6);

            match tour.relation(f1, t2) {
                NodeRel::Predecessor => tour.flip(f1, t2, f4, t4),
                NodeRel::Successor => tour.flip(t2, f1, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move8 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move4);

            match tour.relation(f1, f3) {
                NodeRel::Predecessor => tour.flip(f1, f3, f4, t4),
                NodeRel::Successor => tour.flip(f3, f1, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move9 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move7);

            match tour.relation(f1, t2) {
                NodeRel::Predecessor => tour.flip(f1, t2, f4, t4),
                NodeRel::Successor => tour.flip(t2, f1, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move10 => {
            move_4_opt(tour, pair_1, pair_2, pair_3, pair_4, Opt4SeqMove::Move1);

            match tour.relation(t1, f3) {
                NodeRel::Predecessor => tour.flip(t1, f3, t3, t4),
                NodeRel::Successor => tour.flip(f3, t1, t4, t3),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move11 => {
            move_4_opt(tour, pair_1, pair_2, pair_3, pair_4, Opt4SeqMove::Move7);

            match tour.relation(f1, f4) {
                NodeRel::Predecessor => tour.flip(f1, f4, f2, f3),
                NodeRel::Successor => tour.flip(f4, f1, f3, f2),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move12 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move5);

            match tour.relation(t1, f3) {
                NodeRel::Predecessor => tour.flip(t1, f3, f4, t4),
                NodeRel::Successor => tour.flip(f3, t1, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move13 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move6);

            match tour.relation(f3, f2) {
                NodeRel::Predecessor => tour.flip(f3, f2, f4, t4),
                NodeRel::Successor => tour.flip(f2, f3, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move14 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move7);

            match tour.relation(f3, t1) {
                NodeRel::Predecessor => tour.flip(f3, t1, f4, t4),
                NodeRel::Successor => tour.flip(t1, f3, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move15 => {
            move_4_opt(tour, pair_1, pair_2, pair_3, pair_4, Opt4SeqMove::Move12);

            match tour.relation(f1, f2) {
                NodeRel::Predecessor => tour.flip(f1, f2, f3, t4),
                NodeRel::Successor => tour.flip(f2, f1, t4, f3),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move16 => {
            tour.flip(f2, t2, f3, t3);

            match tour.relation(f1, t1) {
                NodeRel::Predecessor => tour.flip(f1, t1, f4, t4),
                NodeRel::Successor => tour.flip(t1, f1, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }

            match tour.relation(f1, f4) {
                NodeRel::Predecessor => tour.flip(f1, f4, f3, f2),
                NodeRel::Successor => tour.flip(f4, f1, f2, f3),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move17 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move4);

            match tour.relation(t2, t1) {
                NodeRel::Predecessor => tour.flip(t2, t1, f4, t4),
                NodeRel::Successor => tour.flip(t1, t2, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move18 => {
            tour.flip(f2, t2, f3, t3);

            match tour.relation(t2, t3) {
                NodeRel::Predecessor => tour.flip(t2, t3, f4, t4),
                NodeRel::Successor => tour.flip(t3, t2, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }

            match tour.relation(f1, t1) {
                NodeRel::Predecessor => tour.flip(f1, t1, t3, t4),
                NodeRel::Successor => tour.flip(t1, f1, t4, t3),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move19 => {
            tour.flip(f1, t1, f2, t2);

            match tour.relation(f3, t3) {
                NodeRel::Predecessor => tour.flip(f3, t3, f4, t4),
                NodeRel::Successor => tour.flip(t3, f3, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }

            match tour.relation(f1, f2) {
                NodeRel::Predecessor => tour.flip(f1, f2, t3, t4),
                NodeRel::Successor => tour.flip(f2, f1, t4, t3),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Opt4SeqMove::Move20 => {
            move_3_opt(tour, pair_1, pair_2, pair_3, Opt3Move::Move5);

            match tour.relation(f1, f2) {
                NodeRel::Predecessor => tour.flip(f1, f2, f4, t4),
                NodeRel::Successor => tour.flip(f2, f1, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
    }
}
