//! Compare source vs bytecode: size and run speed (parse+interpret vs deserialize+interpret).

use branchy::{default_registry, deserialize_program, interpret, parse_program, serialize_program};
use rand::SeedableRng;
use std::time::Instant;

const ITER: u32 = 5000;

#[test]
fn size_and_speed_comparison() {
  let small_src = "[ a; b; 42; ]";
  let medium_src = r#"
!g(:x) = [ :x; ]
[ !g(hello); !g(world); ]
"#;

  let large_src = r#"
!greet(:who) = [ "Hello, "; :who; "!" ];
!tag(:name, :body) = [ "<"; :name; ">"; :body; "</"; :name; ">" ];
[
  42;
  "a" + "b";
  "x" * 5;
  [ "a"; "b"; "c"; ];
  prefix <one|two|three>;
  !greet("world");
  !tag("div", "content");
  [a-zA-Z:6];
  !upper("hi");
]
"#;

  let builtins = default_registry();

  let big_blocks_src = std::fs::read_to_string("examples/big_blocks.branchy")
    .unwrap_or_else(|_| String::new());

  let cases: Vec<(&str, &str)> = vec![
    ("small", small_src),
    ("medium", medium_src),
    ("large", large_src),
  ];
  let cases_with_file: Vec<(&str, String)> = if big_blocks_src.is_empty() {
    vec![]
  } else {
    vec![("big_blocks", big_blocks_src)]
  };

  for (name, src) in cases {
    run_one(name, src, &builtins);
  }
  for (name, src) in cases_with_file {
    run_one(name, src.as_str(), &builtins);
  }
}

fn run_one(name: &str, src: &str, builtins: &std::collections::HashMap<String, branchy::builtins::BuiltinFn>) {
  let program = parse_program(src).unwrap();
  let bytecode = serialize_program(&program).unwrap();

  let src_bytes = src.len();
  let bc_bytes = bytecode.len();
  let ratio = bc_bytes as f64 / src_bytes as f64;

  // Warmup
  let mut rng = rand::rngs::StdRng::seed_from_u64(0);
  let _ = interpret(&program, builtins, &mut rng, None).unwrap();

  // Time: parse + interpret (source path)
  let t0 = Instant::now();
  for _ in 0..ITER {
    let p = parse_program(src).unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let _ = interpret(&p, builtins, &mut rng, None).unwrap();
  }
  let source_ns = t0.elapsed().as_nanos() / ITER as u128;

  // Time: deserialize + interpret (bytecode path)
  let t0 = Instant::now();
  for _ in 0..ITER {
    let p = deserialize_program(&bytecode).unwrap();
    let mut rng = rand::rngs::StdRng::seed_from_u64(0);
    let _ = interpret(&p, builtins, &mut rng, None).unwrap();
  }
  let bytecode_ns = t0.elapsed().as_nanos() / ITER as u128;

  let speedup = source_ns as f64 / bytecode_ns as f64;

  eprintln!(
    "{}: source {} B, bytecode {} B (ratio {:.2}x); parse+run {:.0} µs/run, deser+run {:.0} µs/run (bytecode {:.2}x faster)",
    name,
    src_bytes,
    bc_bytes,
    ratio,
    source_ns as f64 / 1000.,
    bytecode_ns as f64 / 1000.,
    speedup
  );
}
