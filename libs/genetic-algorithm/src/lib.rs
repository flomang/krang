// TODO implement rank selection roulette wheel selection is fine now
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;

// TRAITS
pub trait CrossoverMethod {
    fn crossover(
        &self,
        rng: &mut dyn RngCore,
        parent_a: &Chromosome,
        parent_b: &Chromosome,
    ) -> Chromosome;
}

pub trait MutationMethod {
    fn mutate(&self, rng: &mut dyn RngCore, child: &mut Chromosome);
}

pub trait Individual {
    fn create(chromosome: Chromosome) -> Self;
    fn chromosome(&self) -> &Chromosome;
    fn fitness(&self) -> f32;
}

pub trait SelectionMethod {
    fn select<'a, I>(&self, rng: &mut dyn RngCore, population: &'a [I]) -> &'a I
    where
        I: Individual;
}

// STRUCTS
#[derive(Clone, Debug)]
pub struct Chromosome {
    genes: Vec<f32>,
}

impl Chromosome {
    pub fn len(&self) -> usize {
        self.genes.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &f32> {
        self.genes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.genes.iter_mut()
    }
}

impl std::ops::Index<usize> for Chromosome {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.genes[index]
    }
}

impl std::iter::FromIterator<f32> for Chromosome {
    fn from_iter<T: IntoIterator<Item = f32>>(iter: T) -> Self {
        Self {
            genes: iter.into_iter().collect(),
        }
    }
}

impl std::iter::IntoIterator for Chromosome {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    // type IntoIter = impl Iterator<Item = f32>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

pub struct RouletteWheelSelection;

impl RouletteWheelSelection {
    pub fn new() -> Self {
        Self
    }
}

impl SelectionMethod for RouletteWheelSelection {
    fn select<'a, I>(&self, rng: &mut dyn RngCore, population: &'a [I]) -> &'a I
    where
        I: Individual,
    {
        population
            .choose_weighted(rng, |individual| individual.fitness())
            .expect("got an empty population")
    }
}

pub struct GeneticAlgorithm<S> {
    selection_method: S,
    crossover_method: Box<dyn CrossoverMethod>,
    mutation_method: Box<dyn MutationMethod>,
}

impl<S> GeneticAlgorithm<S>
where
    S: SelectionMethod,
{
    pub fn new(
        selection_method: S,
        crossover_method: impl CrossoverMethod + 'static,
        mutation_method: impl MutationMethod + 'static,
    ) -> Self {
        Self {
            selection_method,
            crossover_method: Box::new(crossover_method),
            mutation_method: Box::new(mutation_method),
        }
    }

    pub fn evolve<I>(&self, rng: &mut dyn RngCore, population: &[I]) -> (Vec<I>, Statistics)
    where
        I: Individual,
    {
        assert!(!population.is_empty());

        let new_population = (0..population.len())
            .map(|_| {
                let parent_a = self.selection_method.select(rng, population).chromosome();

                let parent_b = self.selection_method.select(rng, population).chromosome();

                let mut child = self.crossover_method.crossover(rng, parent_a, parent_b);

                self.mutation_method.mutate(rng, &mut child);

                I::create(child)
            })
            .collect();

        let stats = Statistics::new(population);

        (new_population, stats)
    }
}

#[derive(Clone, Debug)]
pub struct UniformCrossover;

impl UniformCrossover {
    pub fn new() -> Self {
        Self
    }
}

impl CrossoverMethod for UniformCrossover {
    fn crossover(
        &self,
        rng: &mut dyn RngCore,
        parent_a: &Chromosome,
        parent_b: &Chromosome,
    ) -> Chromosome {
        assert_eq!(parent_a.len(), parent_b.len());

        let parent_a = parent_a.iter();
        let parent_b = parent_b.iter();

        parent_a
            .zip(parent_b)
            .map(|(&a, &b)| if rng.gen_bool(0.5) { a } else { b })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct GaussianMutation {
    /// Probability of changing a gene:
    /// - 0.0 = no genes will be touched
    /// - 1.0 = all genes will be touched
    chance: f32,

    /// Magnitude of that change:
    /// - 0.0 = touched genes will not be modified
    /// - 3.0 = touched genes will be += or -= by at most 3.0
    coeff: f32,
}

impl GaussianMutation {
    pub fn new(chance: f32, coeff: f32) -> Self {
        assert!(chance >= 0.0 && chance <= 1.0);

        Self { chance, coeff }
    }
}

impl MutationMethod for GaussianMutation {
    fn mutate(&self, rng: &mut dyn RngCore, child: &mut Chromosome) {
        for gene in child.iter_mut() {
            let sign = if rng.gen_bool(0.5) { -1.0 } else { 1.0 };

            if rng.gen_bool(self.chance as _) {
                *gene += sign * self.coeff * rng.gen::<f32>();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Statistics {
    min_fitness: f32,
    max_fitness: f32,
    avg_fitness: f32,
}

impl Statistics {
    fn new<I>(population: &[I]) -> Self
    where
        I: Individual,
    {
        assert!(!population.is_empty());

        let mut min_fitness = population[0].fitness();
        let mut max_fitness = min_fitness;
        let mut sum_fitness = 0.0;

        for individual in population {
            let fitness = individual.fitness();

            min_fitness = min_fitness.min(fitness);
            max_fitness = max_fitness.max(fitness);
            sum_fitness += fitness;
        }

        Self {
            min_fitness,
            max_fitness,
            avg_fitness: sum_fitness / (population.len() as f32),
        }
    }

    pub fn min_fitness(&self) -> f32 {
        self.min_fitness
    }

    pub fn max_fitness(&self) -> f32 {
        self.max_fitness
    }

    pub fn avg_fitness(&self) -> f32 {
        self.avg_fitness
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use std::collections::BTreeMap;

    fn actual(chance: f32, coeff: f32) -> Vec<f32> {
        let mut child = vec![1.0, 2.0, 3.0, 4.0, 5.0].into_iter().collect();

        let mut rng = ChaCha8Rng::from_seed(Default::default());

        GaussianMutation::new(chance, coeff).mutate(&mut rng, &mut child);

        child.into_iter().collect()
    }

    //#[cfg(test)]
    //#[derive(Clone, Debug)]
    //pub struct TestIndividual {
    //    fitness: f32,
    //}

    // #[cfg(test)]
    // impl TestIndividual {
    //     pub fn new(fitness: f32) -> Self {
    //         Self { fitness }
    //     }
    // }

    // #[cfg(test)]
    // impl Individual for TestIndividual {
    //     fn fitness(&self) -> f32 {
    //         self.fitness
    //     }

    //     fn chromosome(&self) -> &Chromosome {
    //         panic!("not supported for TestIndividual")
    //     }

    //     fn create(chromosome: Chromosome) -> Self {
    //         todo!()
    //     }
    // }
    #[cfg(test)]
    #[derive(Clone, Debug, PartialEq)]
    pub enum TestIndividual {
        /// For tests that require access to chromosome
        WithChromosome { chromosome: Chromosome },

        /// For tests that don't require access to chromosome
        WithFitness { fitness: f32 },
    }

    #[cfg(test)]
    impl PartialEq for Chromosome {
        fn eq(&self, other: &Self) -> bool {
            approx::relative_eq!(self.genes.as_slice(), other.genes.as_slice(),)
        }
    }

    #[cfg(test)]
    impl TestIndividual {
        pub fn new(fitness: f32) -> Self {
            Self::WithFitness { fitness }
        }
    }

    #[cfg(test)]
    impl Individual for TestIndividual {
        fn create(chromosome: Chromosome) -> Self {
            Self::WithChromosome { chromosome }
        }

        fn chromosome(&self) -> &Chromosome {
            match self {
                Self::WithChromosome { chromosome } => chromosome,

                Self::WithFitness { .. } => {
                    panic!("not supported for TestIndividual::WithFitness")
                }
            }
        }

        fn fitness(&self) -> f32 {
            match self {
                Self::WithChromosome { chromosome } => {
                    chromosome.iter().sum()

                    // ^ the simplest fitness function ever - we're just
                    // summing all the genes together
                }

                Self::WithFitness { fitness } => *fitness,
            }
        }
    }

    mod selection {
        use super::*;

        #[test]
        fn wheel() {
            let method = RouletteWheelSelection::new();
            let mut rng = ChaCha8Rng::from_seed(Default::default());

            let population = vec![
                TestIndividual::new(2.0),
                TestIndividual::new(1.0),
                TestIndividual::new(4.0),
                TestIndividual::new(3.0),
            ];

            let actual_histogram: BTreeMap<i32, _> = (0..1000)
                .map(|_| method.select(&mut rng, &population))
                .fold(Default::default(), |mut histogram, individual| {
                    // as _ means "compiler, pretty please infer what type is required and cast this value into it".
                    // since we've declared the btreemap key to be i32 the type is i32
                    *histogram.entry(individual.fitness() as _).or_default() += 1;

                    histogram
                });

            let expected_histogram = maplit::btreemap! {
                // fitness => how many times this fitness has been chosen
                1 => 98,
                2 => 202,
                3 => 278,
                4 => 422,
            };

            assert_eq!(actual_histogram, expected_histogram);
        }
    }

    fn chromosome() -> Chromosome {
        Chromosome {
            genes: vec![3.0, 1.0, 2.0],
        }
    }

    mod len {
        use super::*;

        #[test]
        fn test() {
            assert_eq!(chromosome().len(), 3);
        }
    }

    mod iter {
        use super::*;

        #[test]
        fn test() {
            let chromosome = chromosome();
            let genes: Vec<_> = chromosome.iter().collect();

            assert_eq!(genes.len(), 3);
            assert_eq!(genes[0], &3.0);
            assert_eq!(genes[1], &1.0);
            assert_eq!(genes[2], &2.0);
        }
    }

    mod iter_mut {
        use super::*;

        #[test]
        fn test() {
            let mut chromosome = chromosome();

            chromosome.iter_mut().for_each(|gene| {
                *gene *= 10.0;
            });

            let genes: Vec<_> = chromosome.iter().collect();

            assert_eq!(genes.len(), 3);
            assert_eq!(genes[0], &30.0);
            assert_eq!(genes[1], &10.0);
            assert_eq!(genes[2], &20.0);
        }
    }

    mod index {
        use super::*;

        #[test]
        fn test() {
            let chromosome = chromosome();

            assert_eq!(chromosome[0], 3.0);
            assert_eq!(chromosome[1], 1.0);
            assert_eq!(chromosome[2], 2.0);
        }
    }

    mod from_iterator {
        use super::*;

        #[test]
        fn test() {
            let chromosome: Chromosome = vec![3.0, 1.0, 2.0].into_iter().collect();

            assert_eq!(chromosome[0], 3.0);
            assert_eq!(chromosome[1], 1.0);
            assert_eq!(chromosome[2], 2.0);
        }
    }

    mod into_iterator {
        use super::*;

        #[test]
        fn test() {
            let chromosome = Chromosome {
                genes: vec![3.0, 1.0, 2.0],
            };

            let genes: Vec<_> = chromosome.into_iter().collect();

            assert_eq!(genes.len(), 3);
            assert_eq!(genes[0], 3.0);
            assert_eq!(genes[1], 1.0);
            assert_eq!(genes[2], 2.0);
        }
    }

    mod crossover {
        use super::*;

        #[test]
        fn test() {
            let mut rng = ChaCha8Rng::from_seed(Default::default());
            let parent_a: Chromosome = (1..=100).map(|n| n as f32).collect();

            let parent_b: Chromosome = (1..=100).map(|n| -n as f32).collect();

            let child = UniformCrossover::new().crossover(&mut rng, &parent_a, &parent_b);

            // Number of genes different between `child` and `parent_a`
            let diff_a = child.iter().zip(parent_a).filter(|(c, p)| *c != p).count();

            // Number of genes different between `child` and `parent_b`
            let diff_b = child.iter().zip(parent_b).filter(|(c, p)| *c != p).count();

            assert_eq!(diff_a, 49);
            assert_eq!(diff_b, 51);
        }
    }

    mod given_zero_chance {
        fn actual(coeff: f32) -> Vec<f32> {
            super::actual(0.0, coeff)
        }

        mod and_zero_coefficient {
            use super::*;

            #[test]
            fn does_not_change_the_original_chromosome() {
                let actual = actual(0.0);
                let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];

                approx::assert_relative_eq!(actual.as_slice(), expected.as_slice(),);
            }
        }

        mod and_nonzero_coefficient {
            use super::*;

            #[test]
            fn does_not_change_the_original_chromosome() {
                let actual = actual(0.5);
                let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];

                approx::assert_relative_eq!(actual.as_slice(), expected.as_slice(),);
            }
        }
    }

    mod given_fifty_fifty_chance {
        fn actual(coeff: f32) -> Vec<f32> {
            super::actual(0.5, coeff)
        }

        mod and_zero_coefficient {
            use super::*;

            #[test]
            fn does_not_change_the_original_chromosome() {
                let actual = actual(0.0);
                let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];

                approx::assert_relative_eq!(actual.as_slice(), expected.as_slice(),);
            }
        }

        mod and_nonzero_coefficient {
            use super::*;

            #[test]
            fn slightly_changes_the_original_chromosome() {
                let actual = actual(0.5);
                let expected = vec![1.0, 1.7756249, 3.0, 4.1596804, 5.0];

                approx::assert_relative_eq!(actual.as_slice(), expected.as_slice(),);
            }
        }
    }

    mod given_max_chance {
        fn actual(coeff: f32) -> Vec<f32> {
            super::actual(1.0, coeff)
        }

        mod and_zero_coefficient {
            use super::*;

            #[test]
            fn does_not_change_the_original_chromosome() {
                let actual = actual(0.0);
                let expected = vec![1.0, 2.0, 3.0, 4.0, 5.0];

                approx::assert_relative_eq!(actual.as_slice(), expected.as_slice(),);
            }
        }

        mod and_nonzero_coefficient {
            use super::*;

            #[test]
            fn entirely_changes_the_original_chromosome() {
                let actual = actual(0.5);

                let expected = vec![1.4545316, 2.1162078, 2.7756248, 3.9505124, 4.638691];

                approx::assert_relative_eq!(actual.as_slice(), expected.as_slice(),);
            }
        }
    }

    #[cfg(test)]
    mod genetic_algo {
        use super::*;
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        fn individual(genes: &[f32]) -> TestIndividual {
            let chromosome = genes.iter().cloned().collect();

            TestIndividual::create(chromosome)
        }

        #[test]
        fn test() {
            let mut rng = ChaCha8Rng::from_seed(Default::default());

            let ga = GeneticAlgorithm::new(
                RouletteWheelSelection::new(),
                UniformCrossover::new(),
                GaussianMutation::new(0.5, 0.5),
            );

            let mut population = vec![
                individual(&[0.0, 0.0, 0.0]), // fitness = 0.0
                individual(&[1.0, 1.0, 1.0]), // fitness = 3.0
                individual(&[1.0, 2.0, 1.0]), // fitness = 4.0
                individual(&[1.0, 2.0, 4.0]), // fitness = 7.0
            ];

            // We're running `.evolve()` a few times, so that the
            // differences between initial and output population are
            // easier to spot.
            //
            // No particular reason for a number of 10 - this test would
            // be fine for 5, 20 or even 1000 generations; the only thing
            // that'd change is the *magnitude* of difference between
            // initial and output population.
            for _ in 0..10 {
                (population, _) = ga.evolve(&mut rng, &population);
            }

            let expected_population = vec![
                individual(&[0.44769490, 2.0648358, 4.3058133]),
                individual(&[1.21268670, 1.5538777, 2.8869110]),
                individual(&[1.06176780, 2.2657390, 4.4287640]),
                individual(&[0.95909685, 2.4618788, 4.0247330]),
            ];

            assert_eq!(population, expected_population);
        }
    }
}
