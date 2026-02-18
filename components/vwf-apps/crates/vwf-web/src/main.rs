//! VWF Web UI - workflow configuration and status interface.

mod components;
mod defaults;
mod report;

use components::{RunStatusViewer, ServicePanel, VarEditor, WorkdirInput, WorkflowEditor};
use gloo::file::callbacks::FileReader;
use gloo::file::File;
use report::RunReport;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RunRequest {
    workflow_text: String,
    workdir: String,
    vars: Vec<(String, String)>,
}

/// Active tab in the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Editor,
    Status,
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let active_tab = use_state(|| Tab::Editor);
    let workflow = use_state(defaults::workflow);
    let workdir = use_state(|| "work/web-demo".to_string());
    let vars = use_state(defaults::vars);
    let run_report: UseStateHandle<Option<RunReport>> = use_state(|| None);
    let _file_reader: UseStateHandle<Option<FileReader>> = use_state(|| None);

    let set_wf = {
        let h = workflow.clone();
        Callback::from(move |v| h.set(v))
    };
    let set_wd = {
        let h = workdir.clone();
        Callback::from(move |v| h.set(v))
    };
    let add_v = {
        let v = vars.clone();
        Callback::from(move |kv| {
            let mut n = (*v).clone();
            n.push(kv);
            v.set(n);
        })
    };
    let on_export = {
        let (wf, wd, vs) = (workflow.clone(), workdir.clone(), vars.clone());
        Callback::from(move |_| {
            let req = RunRequest {
                workflow_text: (*wf).clone(),
                workdir: (*wd).clone(),
                vars: (*vs).clone(),
            };
            gloo::dialogs::alert(&format!(
                "Copy this JSON:\n\n{}",
                serde_json::to_string_pretty(&req).unwrap()
            ));
        })
    };

    let on_tab_editor = {
        let tab = active_tab.clone();
        Callback::from(move |_| tab.set(Tab::Editor))
    };
    let on_tab_status = {
        let tab = active_tab.clone();
        Callback::from(move |_| tab.set(Tab::Status))
    };

    let on_load_report = {
        let report = run_report.clone();
        let reader_handle = _file_reader.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(file) = input.files().and_then(|f| f.get(0)) {
                let file = File::from(file);
                let report = report.clone();
                let reader = gloo::file::callbacks::read_as_text(&file, move |res| {
                    if let Ok(text) = res {
                        if let Ok(parsed) = serde_json::from_str::<RunReport>(&text) {
                            report.set(Some(parsed));
                        } else {
                            gloo::dialogs::alert("Failed to parse run.json");
                        }
                    }
                });
                reader_handle.set(Some(reader));
            }
        })
    };

    // Extract required step kinds from the loaded report for service panel
    let required_kinds: Vec<String> = (*run_report)
        .as_ref()
        .map(|r| r.steps.iter().map(|s| s.kind.to_lowercase()).collect())
        .unwrap_or_default();

    html! {
        <>
            <header>
                <h1>{"VWF Web"}</h1>
                <nav class="tabs">
                    <button
                        class={if *active_tab == Tab::Editor { "tab active" } else { "tab" }}
                        onclick={on_tab_editor}
                    >{"Editor"}</button>
                    <button
                        class={if *active_tab == Tab::Status { "tab active" } else { "tab" }}
                        onclick={on_tab_status}
                    >{"Status"}</button>
                </nav>
            </header>
            <main>
                if *active_tab == Tab::Editor {
                    <WorkflowEditor value={(*workflow).clone()} onchange={set_wf} />
                    <WorkdirInput value={(*workdir).clone()} onchange={set_wd} />
                    <VarEditor vars={(*vars).clone()} on_add={add_v} />
                    <div class="card"><button onclick={on_export}>{"Export run request JSON"}</button></div>
                } else {
                    <div class="card">
                        <h3>{"Load Run Report"}</h3>
                        <p>{"Select a run.json file to view workflow execution status."}</p>
                        <input type="file" accept=".json" onchange={on_load_report} />
                    </div>
                    <ServicePanel required_kinds={required_kinds.clone()} />
                    <RunStatusViewer report={(*run_report).clone()} />
                }
            </main>
            <Footer />
        </>
    }
}

#[function_component(Footer)]
fn footer() -> Html {
    html! {
        <footer style="margin-top: 2em; padding-top: 1em; border-top: 1px solid #ddd; font-size: 0.9em; color: #666;">
            <p>{"Copyright: 2026 Software Wrighter LLC | License: MIT"}</p>
            <p>{"Repository: "}<a href="https://github.com/softwarewrighter/video_workflow_rs">{"github.com/softwarewrighter/video_workflow_rs"}</a></p>
        </footer>
    }
}
