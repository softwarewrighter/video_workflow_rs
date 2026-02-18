//! Workflow YAML editor component.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub value: String,
    pub onchange: Callback<String>,
}

#[function_component(WorkflowEditor)]
pub fn workflow_editor(props: &Props) -> Html {
    let onchange = props.onchange.clone();
    let oninput = Callback::from(move |e: InputEvent| {
        let t = e
            .target_unchecked_into::<web_sys::HtmlTextAreaElement>()
            .value();
        onchange.emit(t);
    });

    html! {
        <div class="card">
            <h2>{"Workflow YAML"}</h2>
            <textarea value={props.value.clone()} {oninput} />
        </div>
    }
}
