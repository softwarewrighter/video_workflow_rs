//! VWF Web UI - workflow configuration interface.

mod components;

use components::{VarEditor, WorkdirInput, WorkflowEditor};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RunRequest {
    workflow_text: String,
    workdir: String,
    vars: Vec<(String, String)>,
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let workflow = use_state(|| default_workflow());
    let workdir = use_state(|| "work/web-demo".to_string());
    let vars = use_state(default_vars);

    html! {
        <>
            <Header />
            <WorkflowEditor value={(*workflow).clone()} onchange={set_state(&workflow)} />
            <WorkdirInput value={(*workdir).clone()} onchange={set_state(&workdir)} />
            <VarEditor vars={(*vars).clone()} on_add={add_var(&vars)} />
            <ExportButton workflow={workflow} workdir={workdir} vars={vars} />
        </>
    }
}

fn set_state(handle: &UseStateHandle<String>) -> Callback<String> {
    let h = handle.clone();
    Callback::from(move |v| h.set(v))
}

fn add_var(vars: &UseStateHandle<Vec<(String, String)>>) -> Callback<(String, String)> {
    let vars = vars.clone();
    Callback::from(move |kv| {
        let mut next = (*vars).clone();
        next.push(kv);
        vars.set(next);
    })
}

fn default_workflow() -> String {
    include_str!("../../../examples/workflows/shorts_narration.yaml").to_string()
}

fn default_vars() -> Vec<(String, String)> {
    vec![
        ("project_name".into(), "My Demo Project".into()),
        ("audience".into(), "curious beginners".into()),
        ("style".into(), "energetic, nerdy, no fluff".into()),
        ("max_words".into(), "220".into()),
    ]
}

#[function_component(Header)]
fn header() -> Html {
    html! {
        <>
            <h1>{"VWF Web (skeleton)"}</h1>
            <p>{"This UI helps you fill in vars and export a run request."}</p>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct ExportProps {
    workflow: UseStateHandle<String>,
    workdir: UseStateHandle<String>,
    vars: UseStateHandle<Vec<(String, String)>>,
}

#[function_component(ExportButton)]
fn export_button(props: &ExportProps) -> Html {
    let workflow = props.workflow.clone();
    let workdir = props.workdir.clone();
    let vars = props.vars.clone();
    let onclick = Callback::from(move |_| export_request(&workflow, &workdir, &vars));
    html! {
        <div class="card">
            <button {onclick}>{"Export run request JSON"}</button>
        </div>
    }
}

fn export_request(workflow: &str, workdir: &str, vars: &[(String, String)]) {
    let req = RunRequest { workflow_text: workflow.into(), workdir: workdir.into(), vars: vars.to_vec() };
    let json = serde_json::to_string_pretty(&req).unwrap();
    gloo::dialogs::alert(&format!("Copy this JSON:\n\n{json}"));
}
