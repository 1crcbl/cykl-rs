use crate::{
    combine_range,
    tour::{Tour, TourOrder, UpdateTourError},
    tour_order,
};

pub fn init_tour<T>(tour: &mut T) -> Result<(), UpdateTourError>
where
    T: Tour,
{
    // TODO: generate a random initial tour
    tour.apply(&tour_order!(0..tour.len()))
}
