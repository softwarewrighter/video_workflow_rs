//! Variable editor component.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub vars: Vec<(String, String)>,
    pub on_add: Callback<(String, String)>,
}

#[function_component(VarEditor)]
pub fn var_editor(props: &Props) -> Html {
    let key = use_state(|| "key".to_string());
    let val = use_state(|| "value".to_string());

    let on_add = {
        let key = key.clone();
        let val = val.clone();
        let cb = props.on_add.clone();
        Callback::from(move |_| cb.emit(((*key).clone(), (*val).clone())))
    };

    html! {
        <div class="card">
            <h2>{"Vars (mad-lib)"}</h2>
            {render_inputs(&key, &val)}
            <button onclick={on_add}>{"Add"}</button>
            {render_var_list(&props.vars)}
        </div>
    }
}

fn render_inputs(key: &UseStateHandle<String>, val: &UseStateHandle<String>) -> Html {
    let key_cb = {
        let key = key.clone();
        Callback::from(move |e: InputEvent| {
            key.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
        })
    };
    let val_cb = {
        let val = val.clone();
        Callback::from(move |e: InputEvent| {
            val.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
        })
    };
    html! {
        <div style="display:flex; gap: 8px;">
            <input type="text" value={(**key).clone()} oninput={key_cb} />
            <input type="text" value={(**val).clone()} oninput={val_cb} />
        </div>
    }
}

fn render_var_list(vars: &[(String, String)]) -> Html {
    html! {
        <ul>
            { for vars.iter().enumerate().map(|(i, (k, v))| html!{ <li key={i}>{format!("{k}={v}")}</li> }) }
        </ul>
    }
}
