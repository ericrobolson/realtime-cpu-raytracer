use rand::{distributions::Uniform, prelude::Distribution, Rng};

/// Returns a random normalized `0.0..1.0` f32
pub fn random() -> f32 {
    let between = Uniform::new(0., 1.);
    let mut rng = rand::thread_rng();
    between.sample(&mut rng)
}

/// Returns a random f32
pub fn random_range(min: f32, max: f32) -> f32 {
    let between = Uniform::new(min, max);
    let mut rng = rand::thread_rng();
    between.sample(&mut rng)
}
