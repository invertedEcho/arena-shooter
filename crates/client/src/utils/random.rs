use std::ops::Range;

use rand::{distr::uniform::SampleUniform, prelude::*};

pub fn get_random_number_from_range<T>(range: Range<T>) -> T
where
    T: SampleUniform + PartialOrd,
{
    let mut rng = rand::rng();
    rng.random_range(range)
}
