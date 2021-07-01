pub fn main() {
    // let tsp = TspBuilder::parse_path(Path::new("./examples/berlin52.tsp")).unwrap();
    // let repo = Repo::from(tsp);
    // let mut tour = TwoLevelList::new(&repo, 13);

    // cand_gen_nn(&mut tour, 10);

    // let mut best_starter = 0;
    // let mut best_tour = TourOrder::default();
    // for ii in 0..tour.len() {
    //     assert!(solve_greedy(&mut tour, Some(ii)).is_ok());

    //     let new_order = tour.tour_order();
    //     if new_order.cost() < best_tour.cost() {
    //         best_tour = new_order;
    //         best_starter = ii;
    //     }
    // }

    // println!("Best tour with starter = {}: {:?}", best_starter, best_tour)
}
