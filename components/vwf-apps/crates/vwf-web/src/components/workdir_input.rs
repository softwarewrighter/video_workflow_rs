//! Workdir input component.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub value: String,
    pub onchange: Callback<String>,
}

#[function_component(WorkdirInput)]
pub fn workdir_input(props: &Props) -> Html {
    let onchange = props.onchange.clone();
    let oninput = Callback::from(move |e: InputEvent| {
        let t = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
        onchange.emit(t);
    });

    html! {
        <div class="card">
            <h2>{"Workdir"}</h2>
            <input type="text" value={props.value.clone()} {oninput} />
        </div>
    }
}
