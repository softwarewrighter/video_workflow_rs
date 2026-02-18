//! Dry-run runtime: records intended writes without touching disk.

use anyhow::Result;
use std::path::{Path, PathBuf};

use super::traits::{CmdOut, LlmClient, Runtime};

/// Dry-run runtime: records writes without disk I/O.
pub struct DryRunRuntime {
    workdir: PathBuf,
    llm: Box<dyn LlmClient>,
    pub planned_writes: Vec<(String, String)>,
    pub planned_dirs: Vec<String>,
}

impl DryRunRuntime {
    pub fn new(workdir: impl Into<PathBuf>, llm: Box<dyn LlmClient>) -> Self {
        Self {
            workdir: workdir.into(),
            llm,
            planned_writes: vec![],
            planned_dirs: vec![],
        }
    }
}

impl Runtime for DryRunRuntime {
    fn workdir(&self) -> &Path {
        &self.workdir
    }

    fn ensure_dir(&mut self, rel: &str) -> Result<()> {
        self.planned_dirs.push(rel.to_string());
        Ok(())
    }

    fn write_text(&mut self, rel: &str, content: &str) -> Result<()> {
        self.planned_writes
            .push((rel.to_string(), content.to_string()));
        Ok(())
    }

    fn read_text(&self, rel: &str) -> Result<String> {
        self.planned_writes
            .iter()
            .rev()
            .find(|(path, _)| path == rel)
            .map(|(_, content)| content.clone())
            .ok_or_else(|| anyhow::anyhow!("DryRunRuntime: file `{rel}` not found"))
    }

    fn run_command(&mut self, prog: &str, _args: &[String], _cwd: Option<&str>) -> Result<CmdOut> {
        Ok(CmdOut {
            status: 0,
            stdout: format!("[dry-run] would run {prog}"),
            stderr: String::new(),
        })
    }

    fn llm(&mut self) -> &mut dyn LlmClient {
        self.llm.as_mut()
    }
}
