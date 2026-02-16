//! VWF Web UI - workflow configuration interface.

mod components;
mod defaults;

use components::{VarEditor, WorkdirInput, WorkflowEditor};
use serde::{Deserialize, Serialize};
use yew::prelude::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RunRequest {
    workflow_text: String,
    workdir: String,
    vars: Vec<(String, String)>,
}

fn main() { yew::Renderer::<App>::new().render(); }

#[function_component(App)]
fn app() -> Html {
    let workflow = use_state(defaults::workflow);
    let workdir = use_state(|| "work/web-demo".to_string());
    let vars = use_state(defaults::vars);
    let set_wf = { let h = workflow.clone(); Callback::from(move |v| h.set(v)) };
    let set_wd = { let h = workdir.clone(); Callback::from(move |v| h.set(v)) };
    let add_v = { let v = vars.clone(); Callback::from(move |kv| { let mut n = (*v).clone(); n.push(kv); v.set(n); }) };
    let on_export = { let (wf, wd, vs) = (workflow.clone(), workdir.clone(), vars.clone()); Callback::from(move |_| {
        let req = RunRequest { workflow_text: (*wf).clone(), workdir: (*wd).clone(), vars: (*vs).clone() };
        gloo::dialogs::alert(&format!("Copy this JSON:\n\n{}", serde_json::to_string_pretty(&req).unwrap()));
    })};
    html! {
        <>
            <header><h1>{"VWF Web"}</h1><p>{"Configure and export workflow run requests."}</p></header>
            <main>
                <WorkflowEditor value={(*workflow).clone()} onchange={set_wf} />
                <WorkdirInput value={(*workdir).clone()} onchange={set_wd} />
                <VarEditor vars={(*vars).clone()} on_add={add_v} />
                <div class="card"><button onclick={on_export}>{"Export run request JSON"}</button></div>
            </main>
            <Footer />
        </>
    }
}

#[function_component(Footer)]
fn footer() -> Html {
    html! {
        <footer style="margin-top: 2em; padding-top: 1em; border-top: 1px solid #ddd; font-size: 0.9em; color: #666;">
            <p>{"Copyright: 2025 Software Wrighter LLC | License: MIT"}</p>
            <p>{"Repository: "}<a href="https://github.com/softwarewrighter/video_workflow_rs">{"github.com/softwarewrighter/video_workflow_rs"}</a></p>
        </footer>
    }
}
