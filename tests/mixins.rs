mod common;

use branchy::{parse_program, resolve_includes};
use std::collections::HashSet;

use common::run_with_seed;

#[test]
fn mixin_spread_param() {
  let p = parse_program(
    r#"
!wrap(:_) = [ a; b; ...:extra; c; ]
[ wrap :_ { :extra = [ x; y; ]; }; ]
"#,
  )
  .unwrap();
  let program = resolve_includes(p, |_| Err("no includes".into())).unwrap();
  let out = run_with_seed(&program, 3);
  let allowed: HashSet<_> = ["a", "b", "x", "y", "c"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}

#[test]
fn mixin_spread_include() {
  let main_src = r#"
[ a; ...include "mix.branchy"; b; ]
"#;
  let mix_src = "[ x; y; ]";
  let p = parse_program(main_src).unwrap();
  let program = resolve_includes(p, |path| {
    if path == "mix.branchy" {
      Ok(mix_src.to_string())
    } else {
      Err("unknown".into())
    }
  })
  .unwrap();
  let out = run_with_seed(&program, 0);
  let allowed: HashSet<_> = ["a", "x", "y", "b"].into_iter().collect();
  assert!(allowed.contains(out.as_str()), "got {}", out);
}
