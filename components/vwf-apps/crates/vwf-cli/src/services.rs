//! Service health checking command.
//!
//! Parses a workflow to detect required services and checks their availability.

use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};

use vwf_core::WorkflowConfig;

/// Known service endpoints and their health check URLs.
#[derive(Debug, Clone)]
struct ServiceInfo {
    name: &'static str,
    description: &'static str,
    default_url: &'static str,
    health_path: &'static str,
    step_kinds: &'static [&'static str],
}

const SERVICES: &[ServiceInfo] = &[
    ServiceInfo {
        name: "Ollama",
        description: "Local LLM (text generation & vision audit)",
        default_url: "http://localhost:11434",
        health_path: "/api/tags",
        step_kinds: &["llm_generate", "llm_audit"],
    },
    ServiceInfo {
        name: "VoxCPM",
        description: "Voice cloning TTS",
        default_url: "http://curiosity:7860",
        health_path: "/api/predict",
        step_kinds: &["tts_generate"],
    },
    ServiceInfo {
        name: "FLUX.1",
        description: "Text-to-image generation",
        default_url: "http://192.168.1.64:8570",
        health_path: "/system_stats",
        step_kinds: &["text_to_image"],
    },
    ServiceInfo {
        name: "SVD-XT",
        description: "Image-to-video animation",
        default_url: "http://192.168.1.64:8100",
        health_path: "/system_stats",
        step_kinds: &["image_to_video"],
    },
    ServiceInfo {
        name: "Wan 2.2",
        description: "Text-to-video generation",
        default_url: "http://192.168.1.64:6000",
        health_path: "/system_stats",
        step_kinds: &["text_to_video"],
    },
];

/// Check service availability for a workflow.
pub fn check_services(workflow_path: &Path) -> Result<()> {
    let text = std::fs::read_to_string(workflow_path)
        .with_context(|| format!("read {}", workflow_path.display()))?;
    let cfg = WorkflowConfig::from_yaml(&text)?;

    println!("Checking services for: {}", cfg.name);
    println!();

    // Collect required step kinds
    let step_kinds: HashSet<String> = cfg
        .steps
        .iter()
        .map(|s| format!("{:?}", s.kind).to_lowercase())
        .collect();

    // Determine which services are required
    let mut required_services = Vec::new();
    for service in SERVICES {
        let is_required = service.step_kinds.iter().any(|k| step_kinds.contains(*k));
        if is_required {
            required_services.push(service);
        }
    }

    if required_services.is_empty() {
        println!("No remote services required for this workflow.");
        println!("(Only local operations like ensure_dirs, write_file, run_command)");
        return Ok(());
    }

    println!("Required services:");
    println!();

    let mut all_ok = true;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    for service in &required_services {
        let url = format!("{}{}", service.default_url, service.health_path);
        let status = check_service_health(&client, &url);

        let status_str = if status {
            "\x1b[32m[RUNNING]\x1b[0m"
        } else {
            all_ok = false;
            "\x1b[31m[NOT RUNNING]\x1b[0m"
        };

        println!(
            "  {} {} - {} {}",
            status_str, service.name, service.description, service.default_url
        );
    }

    println!();

    if !all_ok {
        println!("Some services are not available.");
        println!();
        print_startup_instructions(&required_services);
    } else {
        println!("All required services are running.");
    }

    Ok(())
}

fn check_service_health(client: &reqwest::blocking::Client, url: &str) -> bool {
    match client.get(url).send() {
        Ok(response) => response.status().is_success() || response.status().as_u16() == 422,
        Err(_) => false,
    }
}

fn print_startup_instructions(services: &[&ServiceInfo]) {
    println!("To start missing services:");
    println!();

    // Group by host
    let mut gpu_services: Vec<&&ServiceInfo> = Vec::new();
    let mut curiosity_services: Vec<&&ServiceInfo> = Vec::new();
    let mut local_services: Vec<&&ServiceInfo> = Vec::new();

    for service in services {
        if service.default_url.contains("192.168.1.64") {
            gpu_services.push(service);
        } else if service.default_url.contains("curiosity") {
            curiosity_services.push(service);
        } else if service.default_url.contains("localhost") {
            local_services.push(service);
        }
    }

    if !local_services.is_empty() {
        println!("  Local services:");
        for service in &local_services {
            if service.name == "Ollama" {
                println!("    ollama serve");
            }
        }
    }

    if !curiosity_services.is_empty() {
        println!("  TTS server (curiosity):");
        println!("    ssh curiosity 'docker start voxcpm'");
    }

    if !gpu_services.is_empty() {
        println!("  GPU server (192.168.1.64):");
        let mut docker_services = Vec::new();
        for service in &gpu_services {
            match service.name {
                "FLUX.1" => docker_services.push("comfyui-flux"),
                "SVD-XT" => docker_services.push("comfyui-svd"),
                "Wan 2.2" => docker_services.push("comfyui-wan"),
                _ => {}
            }
        }
        if !docker_services.is_empty() {
            println!("    ssh gpu 'docker start {}'", docker_services.join(" "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_info_is_complete() {
        // Ensure all services have required fields
        for service in SERVICES {
            assert!(!service.name.is_empty());
            assert!(!service.default_url.is_empty());
            assert!(!service.health_path.is_empty());
            assert!(!service.step_kinds.is_empty());
        }
    }
}
