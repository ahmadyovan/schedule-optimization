use rand::Rng;

#[derive(Debug, Clone)]
pub struct Particle {
    pub position: Vec<f32>,
    pub velocity: Vec<f32>,
    pub pbest_position: Vec<f32>,
    pub pbest_fitness: f32,
    pub fitness: f32,
}

impl Particle {
    pub fn new(dimension: usize) -> Self {
        use rand::seq::SliceRandom;

        let mut rng = rand::rng();

        // Bagi domain [0,1) menjadi `dimension` sel
        let mut lhs_values: Vec<f32> = (0..dimension)
            .map(|i| {
                let step = 1.0f32 / dimension as f32;
                let min = i as f32 * step;
                let max = (i + 1) as f32 * step;
                rng.random_range(min..max)
            })
            .collect();

        lhs_values.shuffle(&mut rng); // acak urutannya

        let position = lhs_values.clone();
        let velocity: Vec<f32> = (0..dimension).map(|_| rng.random_range(-1.0f32..1.0f32)).collect();

        Particle {
            position: position.clone(),
            velocity,
            pbest_position: position,
            pbest_fitness: f32::INFINITY,
            fitness: f32::INFINITY,
        }
    }

    pub fn update_velocity(
        &mut self,
        gbest: &[f32],
        inertia_weight: f32,
        cognitive_weight: f32,
        social_weight: f32,
    ) {
        let mut rng = rand::rng();

        for i in 0..self.velocity.len() {
            let r1: f32 = rng.random();
            let r2: f32 = rng.random();

            let cognitive = cognitive_weight * r1 * (self.pbest_position[i] - self.position[i]);
            let social = social_weight * r2 * (gbest[i] - self.position[i]);

            self.velocity[i] = inertia_weight * self.velocity[i] + cognitive + social;

            // Jika kamu ingin clamp velocity:
            // self.velocity[i] = self.velocity[i].clamp(-velocity_clamp, velocity_clamp);
        }
    }

    pub fn update_position(&mut self) {
        for i in 0..self.position.len() {
            self.position[i] += self.velocity[i];
            // self.position[i] = self.round_to_n_digits(self.position[i], 3);
            // Jika kamu ingin clamp posisi:
            // self.position[i] = self.position[i].clamp(0.0f32, position_clamp);
        }
    }
}
