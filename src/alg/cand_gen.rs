use crate::{
    tour::{Tour, TourNode},
    Scalar,
};

/// Generates candidates for each node by using the k-nearest-neighbour method.
///
/// The operation will take O(k*N^2) time to complete.
pub fn cand_gen_nn<T>(tour: &mut T, k: usize)
where
    T: Tour,
{
    for mut base in tour.itr() {
        // Vec of candidates.
        let mut vec_c: Vec<TourNode> = vec![TourNode::default(); k];
        // Vec of distance to nearest candidates.
        let mut vec_d = vec![Scalar::MAX; k];
        let mut count = 0;

        for targ in tour.itr() {
            if base == targ {
                continue;
            }

            if count < k {
                count += 1;
            }
            let mut c_idx = count - 1;

            let d = tour.distance(&base, &targ);

            while c_idx > 0 && d < vec_d[c_idx - 1] {
                vec_c[c_idx] = vec_c[c_idx - 1];
                vec_d[c_idx] = vec_d[c_idx - 1];
                c_idx -= 1;
            }

            if d < vec_d[c_idx] {
                vec_d[c_idx] = d;
                vec_c[c_idx] = targ;
            }
        }

        debug_assert_eq!(k, vec_c.len(), "{:?}", &base);
        base.set_candidates(vec_c);
    }
}
