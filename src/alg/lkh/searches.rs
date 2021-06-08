use crate::tour::TourNode;

use super::KOpt;

pub fn search<T>(tour: &mut T, kopt: KOpt, head_1: &TourNode, tail_1: &TourNode) {
    match kopt {
        KOpt::Opt2 => search_2_opt(tour, head_1, tail_1),
        KOpt::Opt3 => search_3_opt(tour, head_1, tail_1),
    }
}

pub fn search_2_opt<T>(_tour: &mut T, _head_1: &TourNode, _tail_1: &TourNode) {
    todo!()
}

pub fn search_3_opt<T>(_tour: &mut T, _head_1: &TourNode, _tail_1: &TourNode) {
    todo!()
}
