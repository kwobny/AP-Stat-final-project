use rand::distributions::{Distribution, Uniform};
use std::io;
use core::str::FromStr;

fn prompt_and_parse<T: FromStr>(prompt_string: &str) -> T {
    loop {
        println!("{}", prompt_string);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .unwrap();
        match input.trim().parse::<T>() {
            Ok(value) => return value,
            Err(_) => println!("Invalid input. Retry.")
        };
    }
}

fn main() {
    loop {
        let points: u32 = prompt_and_parse("Number of points in sample:");
        let trials: u32 = prompt_and_parse("Number of trials:");

        let pi_s: Vec<_> = (0..trials).map(|_| get_pi_proportion(points)).collect();
        println!("{:?}", pi_s);

        break;
    }
}

fn get_pi_proportion(num_points: u32) -> f64 {
    let mut rng = rand::thread_rng();
    let distribution: Uniform<f64> = Uniform::new(0.0, 1.0);
    let mut points_in_circle: u32 = 0;

    for _ in 0..num_points {
        let point = (
            distribution.sample(&mut rng),
            distribution.sample(&mut rng),
        );
        let is_in_circle = (point.0.powi(2) + point.1.powi(2)).sqrt() <= 1.0;
        if is_in_circle {
            points_in_circle += 1;
        }
    }

    let proportion = (points_in_circle as f64)/(num_points as f64);
    
    return proportion;
}
