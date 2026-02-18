//! Service information panel component.
//!
//! Shows known services and their endpoints. Due to CORS restrictions,
//! actual health checking must be done via CLI.

use yew::prelude::*;

/// Known VWF services
#[derive(Clone, PartialEq)]
struct ServiceInfo {
    name: &'static str,
    description: &'static str,
    url: &'static str,
    step_kinds: &'static [&'static str],
    start_cmd: &'static str,
}

const SERVICES: &[ServiceInfo] = &[
    ServiceInfo {
        name: "Ollama",
        description: "Local LLM for text generation",
        url: "http://localhost:11434",
        step_kinds: &["llm_generate", "llm_audit"],
        start_cmd: "ollama serve",
    },
    ServiceInfo {
        name: "VoxCPM",
        description: "Voice cloning TTS",
        url: "http://curiosity:7860",
        step_kinds: &["tts_generate"],
        start_cmd: "ssh curiosity 'docker start voxcpm'",
    },
    ServiceInfo {
        name: "FLUX.1",
        description: "Text-to-image generation",
        url: "http://192.168.1.64:8570",
        step_kinds: &["text_to_image"],
        start_cmd: "ssh gpu 'docker start comfyui-flux'",
    },
    ServiceInfo {
        name: "SVD-XT",
        description: "Image-to-video animation",
        url: "http://192.168.1.64:8100",
        step_kinds: &["image_to_video"],
        start_cmd: "ssh gpu 'docker start comfyui-svd'",
    },
    ServiceInfo {
        name: "Wan 2.2",
        description: "Text-to-video generation",
        url: "http://192.168.1.64:6000",
        step_kinds: &["text_to_video"],
        start_cmd: "ssh gpu 'docker start comfyui-wan'",
    },
];

#[derive(Properties, PartialEq)]
pub struct Props {
    /// Step kinds from the loaded workflow (to highlight required services)
    #[prop_or_default]
    pub required_kinds: Vec<String>,
}

#[function_component(ServicePanel)]
pub fn service_panel(props: &Props) -> Html {
    let expanded = use_state(|| false);

    let toggle = {
        let expanded = expanded.clone();
        Callback::from(move |_| expanded.set(!*expanded))
    };

    // Determine which services are required based on step kinds
    let required_services: Vec<&ServiceInfo> = if props.required_kinds.is_empty() {
        vec![] // No workflow loaded
    } else {
        SERVICES
            .iter()
            .filter(|s| {
                s.step_kinds
                    .iter()
                    .any(|k| props.required_kinds.contains(&k.to_string()))
            })
            .collect()
    };

    html! {
        <div class="card service-panel">
            <div class="service-header" onclick={toggle.clone()}>
                <h3>
                    {"Services "}
                    if !required_services.is_empty() {
                        <span class="service-count">{format!("({} required)", required_services.len())}</span>
                    }
                    <span class="expand-icon">{if *expanded { "▼" } else { "▶" }}</span>
                </h3>
            </div>

            if *expanded {
                <div class="service-content">
                    <p class="service-note">
                        {"Check service status with: "}
                        <code>{"vwf services workflow.yaml"}</code>
                    </p>

                    <table class="service-table">
                        <thead>
                            <tr>
                                <th>{"Service"}</th>
                                <th>{"URL"}</th>
                                <th>{"Step Types"}</th>
                                <th>{"Start Command"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for SERVICES.iter().map(|service| {
                                let is_required = required_services.contains(&service);
                                let row_class = if is_required { "service-row required" } else { "service-row" };
                                html! {
                                    <tr class={row_class}>
                                        <td class="service-name">
                                            {service.name}
                                            if is_required {
                                                <span class="required-badge">{"needed"}</span>
                                            }
                                        </td>
                                        <td class="service-url"><code>{service.url}</code></td>
                                        <td class="service-kinds">{service.step_kinds.join(", ")}</td>
                                        <td class="service-cmd"><code>{service.start_cmd}</code></td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>

                    <div class="service-tips">
                        <h4>{"Tips"}</h4>
                        <ul>
                            <li>{"GPU services share one GPU - run one at a time"}</li>
                            <li>{"Use "}<code>{"--resume"}</code>{" to skip completed steps after starting services"}</li>
                            <li>{"Check "}<code>{"run.json"}</code>{" for blocked steps waiting on services"}</li>
                        </ul>
                    </div>
                </div>
            }
        </div>
    }
}
