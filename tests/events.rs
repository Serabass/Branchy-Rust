mod common;

use branchy::{default_registry, interpret, parse_program};
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashSet;

use common::{run_with_seed, run_with_seed_and_input};

#[test]
fn event_named() {
  let p = parse_program(
    r#"
@myEvent = [ privet; poka; 123; ]
[ default; ]
"#,
  )
  .unwrap();
  let out = run_with_seed_and_input(&p, 0, Some("myEvent")).0;
  let allowed: HashSet<_> = ["privet", "poka", "123"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn event_string() {
  let p = parse_program(
    r#"
"привет" = [ hello; bye; ]
[ fallback; ]
"#,
  )
  .unwrap();
  let out = run_with_seed_and_input(&p, 0, Some("привет")).0;
  let allowed: HashSet<_> = ["hello", "bye"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn event_regex() {
  let p = parse_program(
    r#"
~"сер[ёе]жа" = [ match; ]
[ no; ]
"#,
  )
  .unwrap();
  let out = run_with_seed_and_input(&p, 0, Some("серёжа")).0;
  assert_eq!(out, "match");
  let out2 = run_with_seed_and_input(&p, 0, Some("сережа")).0;
  assert_eq!(out2, "match");
}

#[test]
fn event_no_input_runs_main() {
  let p = parse_program(
    r#"
@e = [ from_event; ]
[ from_main; ]
"#,
  )
  .unwrap();
  let out = run_with_seed(&p, 0);
  assert_eq!(out, "from_main");
}

#[test]
fn event_no_match_errors() {
  let p = parse_program(
    r#"
@only = [ x; ]
[ main; ]
"#,
  )
  .unwrap();
  let builtins = default_registry();
  let mut rng = StdRng::seed_from_u64(0);
  let r = interpret(&p, &builtins, &mut rng, Some("nonexistent"));
  assert!(r.is_err());
  assert!(r.unwrap_err().message.contains("no event matches"));
}
