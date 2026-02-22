mod common;

use branchy::{interpret, parse_program};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashSet;

use common::run_with_seed;

#[test]
fn simple_branch() {
  let p = parse_program("[ a; b; c; ]").unwrap();
  let out = run_with_seed(&p, 42);
  let allowed: HashSet<_> = ["a", "b", "c"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn template_with_block() {
  let p = parse_program(
    r#"
[
  hello :who {
    :who = [ world; human; ];
  };
]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 123);
  let allowed: HashSet<_> = ["hello world", "hello human"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn inline_call() {
  let p = parse_program("[ hi <a|b> ]").unwrap();
  let out = run_with_seed(&p, 1);
  let allowed: HashSet<_> = ["hi a", "hi b"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn user_function() {
  let p = parse_program(
    r#"
!greet(:x) = [ hello :x; bye :x; ]
[ !greet(world); ]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 99);
  let allowed: HashSet<_> = ["hello world", "bye world"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn literal_number() {
  let p = parse_program("[ 42; 100; ]").unwrap();
  let out = run_with_seed(&p, 0);
  let allowed: HashSet<_> = ["42", "100"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn nested_three_levels() {
  let p = parse_program("[ [ [ a; b ]; c ]; d; ]").unwrap();
  let out = run_with_seed(&p, 2);
  let allowed: HashSet<_> = ["a", "b", "c", "d"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn template_single_literal_value() {
  let p = parse_program(r#"[ fix :x { :x = one; }; ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "fix one");
}

#[test]
fn template_two_vars() {
  let p = parse_program(
    r#"
[ p :a :b { :a = [ 1 ]; :b = [ 2 ]; }; ]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "p 1 2");
}

#[test]
fn inline_three_options() {
  let p = parse_program("[ x <1|2|3> ]").unwrap();
  let out = run_with_seed(&p, 5);
  let allowed: HashSet<_> = ["x 1", "x 2", "x 3"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn single_element_branch() {
  let p = parse_program("[ only; ]").unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "only");
}

#[test]
fn two_adjacent_branches_sequence() {
  // No separator: two blocks in sequence, both run and results concatenated
  let p = parse_program("[ a; b; ] [ a; b; ];").unwrap();
  let out = run_with_seed(&p, 42);
  let allowed: HashSet<_> = ["aa", "ab", "ba", "bb"].into_iter().collect();
  assert!(
    allowed.contains(out.as_str()),
    "expected one of aa|ab|ba|bb, got {:?}",
    out
  );
}

#[test]
fn call_with_block_but_no_function_errors() {
  // Block params without a template that uses them -> error
  let p = parse_program("[ api :get :users { :auth = true; }; ]").unwrap();
  let builtins = branchy::default_registry();
  let mut rng = StdRng::seed_from_u64(0);
  let r = interpret(&p, &builtins, &mut rng, None);
  let err = r.unwrap_err();
  assert!(
    err.message.contains("block parameter") && err.message.contains("api"),
    "expected error about block parameter and 'api', got: {}",
    err
  );
}

#[test]
fn two_user_functions() {
  let p = parse_program(
    r#"
!a(:x) = [ A :x; ]
!b(:x) = [ B :x; ]
[ !a(1); !b(2); ]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 0);
  let allowed: HashSet<_> = ["A 1", "B 2"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn function_two_params() {
  let p = parse_program(
    r#"
!f(:a, :b) = [ :a; and; :b; ]
[ !f(x, y); ]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 2);
  let allowed: HashSet<_> = ["x", "and", "y"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn func_call_with_branch_arg() {
  let p = parse_program(
    r#"
!f(:x) = [ val :x; ]
[ !f([ A; B; ]); ]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 3);
  let allowed: HashSet<_> = ["val A", "val B"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn call_without_block_inside_function() {
  let p = parse_program(
    r#"
!wrap(:t) = [ prefix :t :b { :b = suffix; }; ]
[ !wrap(mid); ]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "prefix mid suffix");
}

#[test]
fn nested_inline_and_branch() {
  let p = parse_program("[ [ a; b ]; z <x|y> ]").unwrap();
  let out = run_with_seed(&p, 10);
  let allowed: HashSet<_> = ["a", "b", "z x", "z y"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn run_quoted_string_literal() {
  let p = parse_program(r#"[ "hello world"; ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "hello world");
}

#[test]
fn string_escape_newline_and_tab() {
  let p = parse_program(r#"[ "a\nb"; "x\ty"; ]"#).unwrap();
  let out = run_with_seed(&p, 0);
  let allowed: HashSet<_> = ["a\nb", "x\ty"].into_iter().collect();
  assert!(
    allowed.contains(out.as_str()),
    "expected literal newline/tab in output, got {:?}",
    out
  );
  if out == "a\nb" {
    assert_eq!(out.as_bytes()[1], b'\n');
  } else {
    assert_eq!(out.as_bytes()[1], b'\t');
  }
}

#[test]
fn optional_param_coin_flip() {
  // :?var may output or skip (50/50); use + to concatenate
  let p = parse_program(
    r#"
!greet(:_) = [ "privet " + :?who; ]
[ greet :_ { :who = [ Vasya; ]; }; ]
"#,
  )
  .unwrap();
  let allowed: HashSet<_> = ["privet ", "privet Vasya"].into_iter().collect();
  let mut saw_with = false;
  let mut saw_without = false;
  for seed in 0..30u64 {
    let out = run_with_seed(&p, seed);
    assert!(
      allowed.contains(out.as_str()),
      "seed {} got {:?}",
      seed,
      out
    );
    if out == "privet Vasya" {
      saw_with = true;
    }
    if out == "privet " {
      saw_without = true;
    }
  }
  assert!(saw_with, ":?who should sometimes output");
  assert!(saw_without, ":?who should sometimes be omitted");
}

#[test]
fn optional_param_in_call_no_function() {
  let p = parse_program(
    r#"
[
  hello :var1 :?var2 {
    :var1 = [ world; ];
    :var2 = [ there; ];
  };
]
"#,
  )
  .unwrap();
  let allowed: HashSet<_> = ["hello world", "hello world there"].into_iter().collect();
  let mut saw_one = false;
  let mut saw_two = false;
  for seed in 0..40u64 {
    let out = run_with_seed(&p, seed);
    assert!(
      allowed.contains(out.as_str()),
      "seed {} got {:?}",
      seed,
      out
    );
    if out == "hello world" {
      saw_one = true;
    }
    if out == "hello world there" {
      saw_two = true;
    }
  }
  assert!(saw_one, ":?var2 in call should sometimes be omitted");
  assert!(saw_two, ":?var2 in call should sometimes be included");
}

#[test]
fn optional_params_in_template_body_undefined_ok() {
  let p = parse_program(
    r#"
!format(:_) = [ :?prefix; :name; :?suffix; ]
[ format :_ { :name = [ solo; ]; }; ]
"#,
  )
  .unwrap();
  let allowed: HashSet<_> = ["", "solo"].into_iter().collect();
  let mut saw_solo = false;
  for seed in 0..30u64 {
    let out = run_with_seed(&p, seed);
    assert!(
      allowed.contains(out.as_str()),
      "seed {}: missing :prefix/:suffix must not yield undefined param, got {:?}",
      seed,
      out
    );
    if out == "solo" {
      saw_solo = true;
    }
  }
  assert!(saw_solo, "body branch sometimes picks :name -> solo");
}
