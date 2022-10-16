pub use self::{animal::*, food::*, world::*};

mod animal;
mod food;
mod world;

use nalgebra as na;
use rand::{Rng, RngCore};


pub struct Simulation {
    world: World,
}

impl Simulation {
    pub fn random(rng: &mut dyn RngCore) -> Self {
        Self {
            world: World::random(rng),
        }
    }

    /// Performs a single step - a single second, so to say - of our
    /// simulation.
    pub fn step(&mut self, rng: &mut dyn RngCore) {
        self.process_collisions(rng);
        self.process_movements();
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    fn process_collisions(&mut self, rng: &mut dyn RngCore) {
        for animal in &mut self.world.animals {
            for food in &mut self.world.foods {
                let distance = na::distance(
                    &animal.position,
                    &food.position,
                );
    
                if distance <= 0.01 {
                    food.position = rng.gen();
                }
            }
        }
    }

    fn process_movements(&mut self) {
        for animal in &mut self.world.animals {
            animal.position +=
                animal.rotation * na::Vector2::new(animal.speed, 0.0);

            animal.position.x = na::wrap(animal.position.x, 0.0, 1.0);
            animal.position.y = na::wrap(animal.position.y, 0.0, 1.0);
        }
    }
}


#[derive(Debug)]
pub struct Point2 {
    x: f32,
    y: f32,
}

// impl Point2 {
//     pub fn new(...) -> Self {
//         /* ... */
//     }

//     /* ... */
// }

// impl Add<Point2> for Point2 {
//     /* ... */
// }

// impl Sub<Point2> for Point2 {
//     /* ... */
// }

// impl Mul<Point2> for f32 {
//     /* ... */
// }

// impl Mul<f32> for Point2 {
//     /* ... */
// }

#[cfg(test)]
mod tests {
    /* ... */
}