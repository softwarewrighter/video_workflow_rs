//! Default values for the web UI.

pub fn workflow() -> String {
    include_str!("../../../../../examples/workflows/shorts_narration.yaml").to_string()
}

pub fn vars() -> Vec<(String, String)> {
    vec![
        ("project_name".into(), "My Demo Project".into()),
        ("audience".into(), "curious beginners".into()),
        ("style".into(), "energetic, nerdy, no fluff".into()),
        ("max_words".into(), "220".into()),
    ]
}
