//! Workflow engine: executes steps based on dependencies (DAG).
//!
//! Key behaviors:
//! - Steps run as soon as their dependencies are satisfied
//! - Failed steps don't stop unrelated work
//! - Steps that depend on failed steps are marked "blocked"
//! - The workflow continues until no more progress can be made
//! - Cycles and invalid dependencies are detected before execution

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;
use uuid::Uuid;

use vwf_config::{StepConfig, WorkflowConfig};
use vwf_render::render_template;
use vwf_runtime::{output_is_valid, Runtime};
use vwf_steps::execute_step;

use super::report::{RunReport, StepReport, StepStatus};

/// Options for workflow execution.
#[derive(Default)]
pub struct RunOptions {
    /// Skip steps whose output_path already exists and is valid.
    pub resume: bool,
}

pub struct Runner;

impl Runner {
    pub fn run(
        rt: &mut dyn Runtime,
        cfg: &WorkflowConfig,
        extra: BTreeMap<String, String>,
    ) -> Result<RunReport> {
        Self::run_with_options(rt, cfg, extra, RunOptions::default())
    }

    pub fn run_with_options(
        rt: &mut dyn Runtime,
        cfg: &WorkflowConfig,
        extra: BTreeMap<String, String>,
        opts: RunOptions,
    ) -> Result<RunReport> {
        let run_id = Uuid::new_v4();
        let started_at = Utc::now();
        let mut vars = cfg.vars.clone();
        vars.extend(extra);

        // Validate the workflow DAG before execution
        validate_dag(&cfg.steps)?;

        execute_dag(rt, &vars, &cfg.steps, run_id, &cfg.name, started_at, &opts)
    }
}

/// Validate the workflow DAG for cycles and invalid dependencies.
fn validate_dag(steps: &[StepConfig]) -> Result<()> {
    let step_ids: HashSet<&str> = steps.iter().map(|s| s.id.as_str()).collect();

    // Check for invalid dependencies (references to non-existent steps)
    for step in steps {
        for dep in &step.depends_on {
            if !step_ids.contains(dep.as_str()) {
                bail!(
                    "Step `{}` depends on `{}`, but no such step exists",
                    step.id, dep
                );
            }
        }
    }

    // Check for cycles using DFS
    let mut visited = HashSet::new();
    let mut rec_stack = HashSet::new();

    // Build adjacency list (step -> steps that depend on it)
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    for step in steps {
        for dep in &step.depends_on {
            dependents.entry(dep.as_str()).or_default().push(&step.id);
        }
    }

    for step in steps {
        if !visited.contains(step.id.as_str())
            && let Some(cycle) = detect_cycle(&step.id, &dependents, &mut visited, &mut rec_stack)
        {
            bail!(
                "Cycle detected in workflow dependencies: {}",
                cycle.join(" -> ")
            );
        }
    }

    Ok(())
}

/// DFS-based cycle detection. Returns the cycle path if found.
fn detect_cycle<'a>(
    node: &'a str,
    dependents: &HashMap<&str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    rec_stack: &mut HashSet<&'a str>,
) -> Option<Vec<String>> {
    visited.insert(node);
    rec_stack.insert(node);

    if let Some(neighbors) = dependents.get(node) {
        for &neighbor in neighbors {
            if !visited.contains(neighbor) {
                if let Some(mut cycle) = detect_cycle(neighbor, dependents, visited, rec_stack) {
                    cycle.insert(0, node.to_string());
                    return Some(cycle);
                }
            } else if rec_stack.contains(neighbor) {
                // Found a cycle
                return Some(vec![node.to_string(), neighbor.to_string()]);
            }
        }
    }

    rec_stack.remove(node);
    None
}

/// DAG-based step execution.
///
/// Runs steps as their dependencies are satisfied. Failed steps don't block
/// unrelated work - only steps that directly or transitively depend on a
/// failed step are marked as blocked.
fn execute_dag(
    rt: &mut dyn Runtime,
    vars: &BTreeMap<String, String>,
    steps: &[StepConfig],
    run_id: Uuid,
    name: &str,
    started: DateTime<Utc>,
    opts: &RunOptions,
) -> Result<RunReport> {
    // Build step lookup and dependency info
    let step_map: HashMap<&str, &StepConfig> = steps.iter().map(|s| (s.id.as_str(), s)).collect();

    // Track state
    let mut completed: HashSet<String> = HashSet::new(); // ok or skipped
    let mut failed: HashSet<String> = HashSet::new();
    let mut blocked: HashSet<String> = HashSet::new();
    let mut reports: HashMap<String, StepReport> = HashMap::new();

    // Keep running while we can make progress
    let mut last_runnable_count = usize::MAX;
    loop {
        let runnable = find_runnable(steps, &completed, &failed, &blocked);

        if runnable.is_empty() {
            // No more runnable steps - check for infinite postponement
            let pending: Vec<&str> = steps
                .iter()
                .filter(|s| {
                    !completed.contains(&s.id)
                    && !failed.contains(&s.id)
                    && !blocked.contains(&s.id)
                    && !reports.contains_key(&s.id)
                })
                .map(|s| s.id.as_str())
                .collect();

            if !pending.is_empty() {
                // This shouldn't happen if validate_dag passed, but catch it anyway
                eprintln!("WARNING: Steps indefinitely blocked (possible bug): {:?}", pending);
            }
            break;
        }

        // Safety check: ensure we're making progress
        if runnable.len() == last_runnable_count {
            eprintln!("WARNING: No progress made in DAG execution loop");
            break;
        }
        last_runnable_count = runnable.len();

        for step_id in runnable {
            let step = step_map[step_id.as_str()];

            // Check resume skip
            if opts.resume && should_skip(rt, vars, step) {
                completed.insert(step_id.clone());
                reports.insert(step_id.clone(), skipped_report(step));
                eprintln!("  [SKIPPED] {}", step_id);
                continue;
            }

            // Run the step
            eprintln!("  [RUNNING] {} ({:?})", step_id, step.kind);
            let report = run_step(rt, vars, step);
            let status = report.status.clone();

            match &status {
                StepStatus::Ok => eprintln!("  [OK] {} ({}ms)", step_id, report.duration_ms),
                StepStatus::Failed => {
                    eprintln!("  [FAILED] {}: {}", step_id, report.error.as_deref().unwrap_or("unknown"));
                }
                _ => {}
            }

            reports.insert(step_id.clone(), report);

            match status {
                StepStatus::Ok | StepStatus::Skipped => {
                    completed.insert(step_id);
                }
                StepStatus::Failed => {
                    failed.insert(step_id.clone());
                    // Mark all transitive dependents as blocked
                    let dependents = find_all_dependents(steps, &step_id);
                    for dep in dependents {
                        if !completed.contains(&dep) && !failed.contains(&dep) {
                            blocked.insert(dep);
                        }
                    }
                }
                StepStatus::Blocked => {
                    // Shouldn't happen during execution, but handle anyway
                    blocked.insert(step_id);
                }
            }
        }
    }

    // Generate blocked reports for any steps we never ran
    for step in steps {
        if !reports.contains_key(&step.id) {
            let now = Utc::now();
            let blocking_deps: Vec<&str> = step
                .depends_on
                .iter()
                .filter(|d| failed.contains(*d) || blocked.contains(*d))
                .map(|s| s.as_str())
                .collect();

            reports.insert(
                step.id.clone(),
                StepReport {
                    id: step.id.clone(),
                    kind: format!("{:?}", step.kind),
                    status: StepStatus::Blocked,
                    started_at: now,
                    finished_at: now,
                    error: Some(format!("Blocked by: {}", blocking_deps.join(", "))),
                    duration_ms: 0,
                },
            );
        }
    }

    // Build ordered report list (preserve original step order)
    let step_reports: Vec<StepReport> = steps
        .iter()
        .filter_map(|s| reports.remove(&s.id))
        .collect();

    // Print summary
    let ok_count = step_reports.iter().filter(|r| r.status == StepStatus::Ok).count();
    let skipped_count = step_reports.iter().filter(|r| r.status == StepStatus::Skipped).count();
    let failed_count = step_reports.iter().filter(|r| r.status == StepStatus::Failed).count();
    let blocked_count = step_reports.iter().filter(|r| r.status == StepStatus::Blocked).count();

    eprintln!();
    eprintln!("Summary: {} ok, {} skipped, {} failed, {} blocked",
              ok_count, skipped_count, failed_count, blocked_count);

    if blocked_count > 0 {
        eprintln!();
        eprintln!("Blocked steps (waiting on failed dependencies):");
        for report in &step_reports {
            if report.status == StepStatus::Blocked {
                eprintln!("  - {}: {}", report.id, report.error.as_deref().unwrap_or(""));
            }
        }
        eprintln!();
        eprintln!("To unblock: fix the failed step(s), then re-run with --resume");
    }

    let has_failures = failed_count > 0 || blocked_count > 0;

    let report = RunReport {
        run_id,
        workflow_name: name.into(),
        started_at: started,
        finished_at: Utc::now(),
        steps: step_reports,
        vars: vars.clone(),
    };

    if has_failures {
        // Return error but include full report in context
        Err(anyhow::anyhow!("Workflow completed with failures"))
            .context(serde_json::to_string_pretty(&report).unwrap_or_default())
    } else {
        Ok(report)
    }
}

/// Find steps that can be run right now.
/// A step is runnable if:
/// - It hasn't been completed, failed, or blocked
/// - All its dependencies have completed successfully
fn find_runnable(
    steps: &[StepConfig],
    completed: &HashSet<String>,
    failed: &HashSet<String>,
    blocked: &HashSet<String>,
) -> Vec<String> {
    steps
        .iter()
        .filter(|step| {
            // Not already processed
            !completed.contains(&step.id)
                && !failed.contains(&step.id)
                && !blocked.contains(&step.id)
                // All dependencies satisfied
                && step.depends_on.iter().all(|dep| completed.contains(dep))
        })
        .map(|s| s.id.clone())
        .collect()
}

/// Find all steps that directly or transitively depend on the given step.
fn find_all_dependents(steps: &[StepConfig], step_id: &str) -> HashSet<String> {
    let mut dependents = HashSet::new();
    let mut to_check = vec![step_id.to_string()];

    while let Some(current) = to_check.pop() {
        for step in steps {
            if step.depends_on.contains(&current) && !dependents.contains(&step.id) {
                dependents.insert(step.id.clone());
                to_check.push(step.id.clone());
            }
        }
    }

    dependents
}

fn should_skip(rt: &dyn Runtime, vars: &BTreeMap<String, String>, step: &StepConfig) -> bool {
    let Some(ref output) = step.resume_output else {
        return false;
    };
    let Ok(path) = render_template(output, vars) else {
        return false;
    };
    let full_path = rt.workdir().join(&path);
    output_is_valid(&full_path)
}

fn skipped_report(step: &StepConfig) -> StepReport {
    let now = Utc::now();
    StepReport {
        id: step.id.clone(),
        kind: format!("{:?}", step.kind),
        status: StepStatus::Skipped,
        started_at: now,
        finished_at: now,
        error: None,
        duration_ms: 0,
    }
}

fn run_step(rt: &mut dyn Runtime, vars: &BTreeMap<String, String>, step: &StepConfig) -> StepReport {
    let started = Utc::now();
    let t0 = Instant::now();
    let result = execute_step(rt, vars, step);
    StepReport {
        id: step.id.clone(),
        kind: format!("{:?}", step.kind),
        status: if result.is_ok() {
            StepStatus::Ok
        } else {
            StepStatus::Failed
        },
        started_at: started,
        finished_at: Utc::now(),
        error: result.err().map(|e| e.to_string()),
        duration_ms: t0.elapsed().as_millis(),
    }
}
