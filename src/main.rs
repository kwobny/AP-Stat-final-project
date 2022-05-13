use rand::distributions::{Distribution, Uniform};
use std::io;

fn main() {
    loop {
        println!("Number of points in sample:");
        let mut points = String::new();
        io::stdin()
            .read_line(&mut points)
            .unwrap();
        let points: u32 = match points.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Input is not a number. Re-enter values.");
                continue;
            }
        };

        println!("Number of trials:");
        let mut trials = String::new();
        io::stdin()
            .read_line(&mut trials)
            .unwrap();
        let trials: u32 = match trials.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Input is not a number. Re-enter values.");
                continue;
            }
        };

        let pi_s: Vec<_> = (0..trials).map(|_| get_pi(points)).collect();
        println!("{:?}", pi_s);

        break;
    }
}

fn get_pi(num_points: u32) -> f64 {
    let mut rng = rand::thread_rng();
    let distribution: Uniform<f64> = Uniform::new(0.0, 1.0);
    let mut points_in_circle: u32 = 0;

    for _ in 0..num_points {
        let point = (
            distribution.sample(&mut rng),
            distribution.sample(&mut rng),
        );
        let is_in_circle = (point.0.powi(2) + point.1.powi(2)).sqrt() < 1.0;
        if is_in_circle {
            points_in_circle += 1;
        }
    }

    let proportion = (points_in_circle as f64)/(num_points as f64);
    let pi = proportion * 4.0;

    return pi;
}
