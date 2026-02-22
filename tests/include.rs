mod common;

use branchy::{parse_program, resolve_includes};
use common::run_with_seed;

#[test]
fn include_resolved_merges_functions() {
    let main_src = r#"
include "lib.branchy";
[ !greet(); ]
"#;
    let p = parse_program(main_src).unwrap();
    assert_eq!(p.includes, ["lib.branchy"]);
    let lib_src = r#"!greet() = [ hi; ]; [ unused; ]"#;
    let program = resolve_includes(p, |path| {
        if path == "lib.branchy" {
            Ok(lib_src.to_string())
        } else {
            Err("unknown".into())
        }
    })
    .unwrap();
    assert!(program.includes.is_empty());
    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].name, "greet");
    let out = run_with_seed(&program, 0);
    assert_eq!(out, "hi");
}
