use rand::Rng;

pub trait ActivationFunction:
    std::marker::Sync + std::marker::Send + std::fmt::Debug + std::marker::Copy + Clone
{
    fn activation(x: f32) -> f32;
    fn inverse_activation(x: f32) -> f32;
    fn activation_derivative(x: f32) -> f32;
}

#[derive(Copy, Clone, Debug)]
pub struct Tanh {}
impl ActivationFunction for Tanh {
    fn activation(x: f32) -> f32 {
        x.tanh()
    }
    fn inverse_activation(x: f32) -> f32 {
        let result = if x.abs() > 0.9999 {
            (0.9999 as f32).atanh().copysign(x)
        } else {
            x.atanh()
        };
        assert!(result.is_finite(), "x: {}, result: {}", x, result);
        result
    }
    fn activation_derivative(x: f32) -> f32 {
        1.0 - x.tanh().powi(2)
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct NeuralNet<A: ActivationFunction> {
    network: Vec<f32>,
    input_size: usize,
    phantom: std::marker::PhantomData<A>, //rng: rand::rngs::thread::ThreadRng
}

impl<A: ActivationFunction> NeuralNet<A> {
    pub fn new(input_size: usize) -> Self {
        Self {
            network: vec![1.0; input_size + 1],
            input_size,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn create_random(input_size: usize, rng: &mut rand::rngs::ThreadRng) -> Self {
        let mut network = Vec::with_capacity(input_size + 1);
        for _ in 0..(input_size + 1) {
            network.push(rng.gen());
        }
        Self {
            network,
            input_size,
            phantom: std::marker::PhantomData,
        }
    }

    fn get_unactivated(&self, inputs: &[f32]) -> f32 {
        self.network
            .iter()
            .zip(
                inputs
                    .iter()
                    .cloned()
                    .take(self.input_size)
                    .chain(std::iter::once(1.0)), // Add bias
            )
            .map(|(weight, input)| weight * input)
            .sum()
    }

    pub fn predict(&self, inputs: &[f32]) -> f32 {
        A::activation(self.get_unactivated(inputs))
    }

    pub fn get_action_gradients(&self, inputs: &[f32], target_score: f32) -> Vec<f32> {
        let with_weights: f32 = self.get_unactivated(inputs);
        let output = A::activation(with_weights);
        // overall_score = (target_score - output).powi(2);

        // d(overall)/d(output) = 2.0 * (target_score - output);
        // d(output)/d(with_weights) = activation_derivative(with_weights);

        // d(overall)/d(with_weights) = d(output)/d(with_weights) * d(overall)/d(output)
        // d(overall)/d(with_weights) = activation_derivative(with_weights) * 2.0 * (target_score - output)

        // d(overall)/d(weights) = d(with_weights)/d(weights) * d(overall)/d(with_weights)
        // d(with_weights)/d(weights) = inputs
        // d(overall)/d(weights) = weights * (activation_derivative(result) * 2.0 * (target_score - output))
        let d_overall_d_with_weights =
            A::activation_derivative(with_weights) * 2.0 * (target_score - output);
        let mut gradients = inputs.to_vec();
        gradients.push(1.0); // Add bias
        for ptr in gradients.iter_mut() {
            *ptr *= d_overall_d_with_weights;
        }
        gradients
    }

    pub fn learn<T: AsRef<[f32]>>(
        &mut self,
        training_data: &[(f32, T)],
        iterations: usize,
        step_size: f32,
    ) {
        debug_assert!({
            println!("Learning from {} actions", training_data.len());
            true
        });
        fn get_overall_score<A: ActivationFunction, T: AsRef<[f32]>>(
            ai: &NeuralNet<A>,
            training_data: &[(f32, T)],
        ) -> f64 {
            training_data
                .iter()
                .map(|(target_score, inputs)| {
                    (*target_score as f64 - ai.predict(inputs.as_ref()) as f64).powi(2)
                })
                .sum()
        }
        let mut total_score_before: f64 = get_overall_score(self, &training_data);
        let old_score = total_score_before;
        for i in 0..iterations {
            let mut overall_gradient = vec![0.0; self.network.len()];
            for (target_score, inputs) in training_data.iter() {
                for (ptr, new) in overall_gradient.iter_mut().zip(
                    self.get_action_gradients(inputs.as_ref(), *target_score)
                        .iter(),
                ) {
                    *ptr += new;
                }
            }
            let mut new = self.clone();
            for (ptr, gradient) in new.network.iter_mut().zip(overall_gradient.iter()) {
                *ptr += gradient * step_size;
            }
            let new_score = get_overall_score(&new, &training_data);
            if new_score < total_score_before {
                *self = new;
                /*println!(
                    "{}: Successfully trained from {} to {}",
                    i, total_score_before, new_score
                );*/
                total_score_before = new_score
            } else {
                /*println!(
                    "{}: Failed training from {} up to {}",
                    i, total_score_before, new_score
                );*/
                //println!("Gradient was: {:?}", overall_gradient);
                println!(
                    "{}: Successfully trained from {} to {}",
                    i, old_score, total_score_before
                );
                break;
            }
        }
    }
}
