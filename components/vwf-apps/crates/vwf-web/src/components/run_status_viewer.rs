//! Workflow run status viewer component.

use crate::report::{RunReport, StepStatus};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub report: Option<RunReport>,
}

#[function_component(RunStatusViewer)]
pub fn run_status_viewer(props: &Props) -> Html {
    match &props.report {
        None => html! {
            <div class="card">
                <h2>{"Workflow Status"}</h2>
                <p class="status-empty">{"No run report loaded. Load a run.json file to view execution status."}</p>
            </div>
        },
        Some(report) => {
            let total = report.steps.len();
            let ok = report.steps.iter().filter(|s| s.status == StepStatus::Ok).count();
            let skipped = report.steps.iter().filter(|s| s.status == StepStatus::Skipped).count();
            let failed = report.steps.iter().filter(|s| s.status == StepStatus::Failed).count();
            let blocked = report.steps.iter().filter(|s| s.status == StepStatus::Blocked).count();

            html! {
                <div class="card status-viewer">
                    <h2>{"Workflow Status: "}{&report.workflow_name}</h2>
                    <div class="status-summary">
                        <span class="status-badge status-ok">{format!("{} OK", ok)}</span>
                        <span class="status-badge status-skipped">{format!("{} Skipped", skipped)}</span>
                        <span class="status-badge status-failed">{format!("{} Failed", failed)}</span>
                        <span class="status-badge status-blocked">{format!("{} Blocked", blocked)}</span>
                        <span class="status-total">{format!("{} total steps", total)}</span>
                    </div>

                    if failed > 0 || blocked > 0 {
                        <div class="alert alert-warning">
                            <strong>{"Action Required: "}</strong>
                            if failed > 0 {
                                {format!("{} step(s) failed. ", failed)}
                            }
                            if blocked > 0 {
                                {format!("{} step(s) blocked by failed dependencies. ", blocked)}
                            }
                            {"Check service status and re-run with --resume."}
                        </div>
                    }

                    <table class="step-table">
                        <thead>
                            <tr>
                                <th>{"Status"}</th>
                                <th>{"Step ID"}</th>
                                <th>{"Kind"}</th>
                                <th>{"Duration"}</th>
                                <th>{"Error"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            { for report.steps.iter().map(|step| {
                                let status_class = match step.status {
                                    StepStatus::Ok => "status-ok",
                                    StepStatus::Skipped => "status-skipped",
                                    StepStatus::Failed => "status-failed",
                                    StepStatus::Blocked => "status-blocked",
                                };
                                let status_icon = match step.status {
                                    StepStatus::Ok => "OK",
                                    StepStatus::Skipped => "SKIP",
                                    StepStatus::Failed => "FAIL",
                                    StepStatus::Blocked => "BLOCKED",
                                };
                                html! {
                                    <tr class={format!("step-row {}", status_class)}>
                                        <td><span class={format!("status-indicator {}", status_class)}>{status_icon}</span></td>
                                        <td class="step-id">{&step.id}</td>
                                        <td class="step-kind">{&step.kind}</td>
                                        <td class="step-duration">{format!("{}ms", step.duration_ms)}</td>
                                        <td class="step-error">{step.error.as_deref().unwrap_or("-")}</td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                </div>
            }
        }
    }
}
