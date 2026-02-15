use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::BTreeMap;

/// Very small template renderer: replaces `{{var}}` with `vars["var"]`.
/// Deliberately tiny to keep behavior predictable.
pub fn render_template(input: &str, vars: &BTreeMap<String, String>) -> Result<String> {
    // Matches {{ key }} with optional spaces
    let re = Regex::new(r#"\{\{\s*([a-zA-Z0-9_\-\.]+)\s*\}\}"#).unwrap();
    let mut out = String::with_capacity(input.len());
    let mut last = 0usize;

    for cap in re.captures_iter(input) {
        let m = cap.get(0).unwrap();
        let key = cap.get(1).unwrap().as_str();
        out.push_str(&input[last..m.start()]);
        match vars.get(key) {
            Some(v) => out.push_str(v),
            None => return Err(anyhow!("Missing template var: `{key}`")),
        }
        last = m.end();
    }
    out.push_str(&input[last..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let err = render_template("hi {{who}}", &vars).unwrap_err().to_string();
        assert!(err.contains("Missing template var"));
    }
}
