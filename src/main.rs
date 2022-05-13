use rand::distributions::{Distribution, Uniform};

fn main() {
    println!("{}", get_pi(3_000_000));
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
