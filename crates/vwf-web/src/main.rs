use yew::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct RunRequest {
    workflow_text: String,
    workdir: String,
    vars: Vec<(String,String)>,
}

#[function_component(App)]
fn app() -> Html {
    let workflow = use_state(|| include_str!("../../../examples/workflows/shorts_narration.yaml").to_string());
    let workdir = use_state(|| "work/web-demo".to_string());
    let var_key = use_state(|| "project_name".to_string());
    let var_val = use_state(|| "My Demo Project".to_string());
    let vars = use_state(|| vec![
        ("project_name".to_string(), "My Demo Project".to_string()),
        ("audience".to_string(), "curious beginners".to_string()),
        ("style".to_string(), "energetic, nerdy, no fluff".to_string()),
        ("max_words".to_string(), "220".to_string()),
    ]);

    let on_add = {
        let vars = vars.clone();
        let k = var_key.clone();
        let v = var_val.clone();
        Callback::from(move |_| {
            let mut next = (*vars).clone();
            next.push(((*k).clone(), (*v).clone()));
            vars.set(next);
        })
    };

    let on_export = {
        let workflow = workflow.clone();
        let workdir = workdir.clone();
        let vars = vars.clone();
        Callback::from(move |_| {
            let req = RunRequest{
                workflow_text: (*workflow).clone(),
                workdir: (*workdir).clone(),
                vars: (*vars).clone(),
            };
            let json = serde_json::to_string_pretty(&req).unwrap();
            gloo::dialogs::alert(&format!("Copy this JSON into your CLI runner (future: auto-run):\n\n{json}"));
        })
    };

    html! {
        <>
          <h1>{"VWF Web (skeleton)"}</h1>
          <p>
            {"This UI is intentionally simple: it helps you fill in vars and export a run request. "}
            {"Later, you can wire it to a local HTTP runner or spawn the CLI."}
          </p>

          <div class="card">
            <h2>{"Workflow YAML"}</h2>
            <textarea
                value={(*workflow).clone()}
                oninput={{
                    let workflow = workflow.clone();
                    Callback::from(move |e: InputEvent| {
                        let t = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                        workflow.set(t);
                    })
                }}
            />
          </div>

          <div class="card">
            <h2>{"Workdir"}</h2>
            <input type="text"
                value={(*workdir).clone()}
                oninput={{
                    let workdir = workdir.clone();
                    Callback::from(move |e: InputEvent| {
                        let t = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                        workdir.set(t);
                    })
                }}
            />
          </div>

          <div class="card">
            <h2>{"Vars (mad-lib)"}</h2>
            <div style="display:flex; gap: 8px;">
              <input type="text" value={(*var_key).clone()}
                oninput={{
                    let var_key = var_key.clone();
                    Callback::from(move |e: InputEvent| {
                        var_key.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
                    })
                }}
              />
              <input type="text" value={(*var_val).clone()}
                oninput={{
                    let var_val = var_val.clone();
                    Callback::from(move |e: InputEvent| {
                        var_val.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
                    })
                }}
              />
              <button onclick={on_add}>{"Add"}</button>
            </div>

            <ul>
              { for (*vars).iter().enumerate().map(|(i,(k,v))| html!{ <li key={i}>{format!("{k}={v}")}</li> }) }
            </ul>
          </div>

          <div class="card">
            <button onclick={on_export}>{"Export run request JSON"}</button>
          </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
