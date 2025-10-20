use rand::prelude::*;

pub fn get_random_number_from_range_i32(start: i32, end: i32) -> i32 {
    let mut rng = rand::rng();

    let mut nums: Vec<i32> = (start..end).collect();
    nums.shuffle(&mut rng);
    *nums.choose(&mut rng).unwrap()
}

pub fn get_random_number_from_range_i32_to_f32_with_fixed_step(
    start: i32,
    end: i32,
    // step: f32,
) -> f32 {
    let mut rng = rand::rng();

    // TODO: i think this factor has to be adjusted depending on the step
    // -> so for now we disable step feature and always assume 0.1
    let fixed_start = start * 10;
    let fixed_end = end * 10;

    let mut nums: Vec<f32> = Vec::new();
    for i in fixed_start..fixed_end {
        let i = i as f32 * 0.1;
        nums.push(i);
    }

    nums.shuffle(&mut rng);
    *nums.choose(&mut rng).unwrap()
}
