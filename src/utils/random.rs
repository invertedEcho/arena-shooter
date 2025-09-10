use rand::prelude::*;

pub fn get_random_number_from_range_i32(start: i32, end: i32) -> i32 {
    let mut rng = rand::rng();

    let mut nums: Vec<i32> = (start..end).collect();
    nums.shuffle(&mut rng);
    *nums.choose(&mut rng).unwrap()
}
