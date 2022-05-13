use rand::distributions::{Distribution, Uniform};

fn main() {
    get_pi(3);
}

fn get_pi(num_points: u32) {
    let mut rng = rand::thread_rng();
    let distribution: Uniform<f64> = Uniform::new(0.0, 1.0);

    let random_number = distribution.sample(&mut rng);
    println!("{}", random_number);
}
