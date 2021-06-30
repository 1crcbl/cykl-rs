use crate::{
    combine_range,
    tour::{Tour, TourOrder, UpdateTourError},
    tour_order,
};

pub fn init_tour<T>(tour: &mut T) -> Result<(), UpdateTourError>
where
    T: Tour,
{
    tour.apply(&tour_order!(0..tour.len()))
}
