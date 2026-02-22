use branchy::{default_registry, interpret, Span};
use rand::rngs::StdRng;
use rand::SeedableRng;

pub fn run_with_seed(program: &branchy::Program, seed: u64) -> String {
    run_with_seed_and_input(program, seed, None).0
}

pub fn run_with_seed_and_input(
    program: &branchy::Program,
    seed: u64,
    input: Option<&str>,
) -> (String, Vec<Span>) {
    let builtins = default_registry();
    let mut rng = StdRng::seed_from_u64(seed);
    interpret(program, &builtins, &mut rng, input).unwrap()
}

/// Run with seed and return (output, trace). Use for trace tests.
pub fn run_with_seed_and_trace(
    program: &branchy::Program,
    seed: u64,
) -> (String, Vec<Span>) {
    run_with_seed_and_input(program, seed, None)
}
