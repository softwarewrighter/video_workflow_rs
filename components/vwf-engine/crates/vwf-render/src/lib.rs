//! Template rendering for VWF workflows.

use anyhow::{Result, anyhow};
use regex::Regex;
use std::collections::BTreeMap;

/// Render a template by replacing `{{var}}` with values from vars.
pub fn render_template(input: &str, vars: &BTreeMap<String, String>) -> Result<String> {
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
