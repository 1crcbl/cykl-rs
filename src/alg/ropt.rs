use crate::tour::TourImpltor;
use crate::tour::{NodeRel, Tour, TourNode};

/// Executes the 2-opt move.
pub fn move_2_opt(
    tour: &mut TourImpltor,
    f1: &TourNode,
    t1: &TourNode,
    f2: &TourNode,
    t2: &TourNode,
) {
    tour.flip(f1, t1, f2, t2);
}

/// Describes all possible outcomes for a 3-opt move setup.
///
/// Assumes that the order of input nodes is as follows: [f1-t1]-[f2-t2]-[f3-t3]
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
pub fn move_3_opt(
    tour: &mut TourImpltor,
    f1: &TourNode,
    t1: &TourNode,
    f2: &TourNode,
    t2: &TourNode,
    f3: &TourNode,
    t3: &TourNode,
    move_case: Opt3Move,
) {
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

/// Describes purely sequential and non-sequential 4-opt moves.
///
/// Assumes that the order of input nodes is as follows: [f1-t1]-[f2-t2]-[f3-t3]-[f4-t4].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Move4Opt {
    /// Pure sequential 4-opt move that results in [f1-f2]-[t1-f3]-[t2-f4]-[t3-t4].
    ///
    /// This move comprises of [`Opt3Move::Move5`] and a [`move_2_opt`] acting on the last two pairs.
    SeqMove1,
    /// Pure sequential 4-opt move that results in [f1-t2]-[f3-t1]-[f2-f4]-[t3-t4].
    ///
    /// This move comprises of [`Opt3Move::Move7`] and a [`move_2_opt`] acting on the last two pairs.
    SeqMove2,
}

/// Executes a 4-opt move.
pub fn move_4_opt(
    tour: &mut TourImpltor,
    f1: &TourNode,
    t1: &TourNode,
    f2: &TourNode,
    t2: &TourNode,
    f3: &TourNode,
    t3: &TourNode,
    f4: &TourNode,
    t4: &TourNode,
    move_case: Move4Opt,
) {
    match move_case {
        Move4Opt::SeqMove1 => {
            move_3_opt(tour, f1, t1, f2, t2, f3, t3, Opt3Move::Move5);

            match tour.relation(t2, t3) {
                NodeRel::Predecessor => tour.flip(t2, t3, f4, t4),
                NodeRel::Successor => tour.flip(t3, t2, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
        Move4Opt::SeqMove2 => {
            move_3_opt(tour, f1, t1, f2, t2, f3, t3, Opt3Move::Move7);

            match tour.relation(f2, t3) {
                NodeRel::Predecessor => tour.flip(f2, t3, f4, t4),
                NodeRel::Successor => tour.flip(t3, f2, t4, f4),
                NodeRel::None => panic!("Broken tour"),
            }
        }
    }
}
