use crate::tour::TourImpltor;
use crate::tour::{NodeRel, Tour, TourNode};

/// Executes the 2-opt move on ```tour```.
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
/// Assumes that the order of input nodes is as follows: [12]-[34]-[56]
#[derive(Clone, Copy, Debug)]
pub enum Opt3Move {
    /// Equivalent to a single 2-opt move.
    /// Results in [13]-[24]-[56].
    Move1,
    /// Equivalent to a single 2-opt move.
    /// Results in [12]-[35]-[46].
    Move2,
    /// Equivalent to a single 2-opt move.
    /// Results in [15]-[43]-[26].
    Move3,
    /// Equivalent to two 2-opt moves. The first move is [`Opt3Move::Move1`].
    /// Results in [15]-[42]-[36].
    Move4,
    /// Equivalent to two 2-opt moves. The first move is [`Opt3Move::Move2`].
    /// Results in [13]-[25]-[46].
    Move5,
    /// Equivalent to two 2-opt moves. The first move is [`Opt3Move::Move3`].
    /// Results in [14]-[53]-[26].
    Move6,
    /// Equivalent to three 3-opt moves.
    /// Results in [14]-[52]-[36].
    Move7,
}

/// Executes the 3-opt move on ```tour```.
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
