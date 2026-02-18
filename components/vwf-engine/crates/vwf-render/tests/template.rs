//! Template rendering tests.

use std::collections::BTreeMap;
use vwf_render::render_template;

#[test]
fn replaces_vars() {
    let mut vars = BTreeMap::new();
    vars.insert("name".into(), "Mike".into());
    let s = render_template("hi {{name}}", &vars).unwrap();
    assert_eq!(s, "hi Mike");
}

#[test]
fn missing_var_errors() {
    let vars = BTreeMap::new();
    let err = render_template("hi {{who}}", &vars)
        .unwrap_err()
        .to_string();
    assert!(err.contains("Missing template var"));
}
