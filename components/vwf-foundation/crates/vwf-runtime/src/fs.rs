//! Filesystem-based runtime implementation.

use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::traits::{CmdOut, LlmClient, Runtime};

/// Real filesystem runtime.
pub struct FsRuntime {
    workdir: PathBuf,
    llm: Box<dyn LlmClient>,
    pub command_allowlist: BTreeSet<String>,
}

impl FsRuntime {
    pub fn new(workdir: impl Into<PathBuf>, llm: Box<dyn LlmClient>) -> Self {
        Self {
            workdir: workdir.into(),
            llm,
            command_allowlist: Default::default(),
        }
    }
}

impl Runtime for FsRuntime {
    fn workdir(&self) -> &Path {
        &self.workdir
    }

    fn ensure_dir(&mut self, rel: &str) -> Result<()> {
        let p = self.workdir.join(rel);
        std::fs::create_dir_all(&p).with_context(|| format!("create_dir_all {}", p.display()))
    }

    fn write_text(&mut self, rel: &str, content: &str) -> Result<()> {
        let p = self.workdir.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, content).with_context(|| format!("write {}", p.display()))
    }

    fn read_text(&self, rel: &str) -> Result<String> {
        let p = self.workdir.join(rel);
        let bytes = std::fs::read(&p).with_context(|| format!("read {}", p.display()))?;
        Ok(String::from_utf8(bytes)?)
    }

    fn run_command(&mut self, prog: &str, args: &[String], cwd: Option<&str>) -> Result<CmdOut> {
        if !self.command_allowlist.is_empty() && !self.command_allowlist.contains(prog) {
            anyhow::bail!("Command not allowed: `{prog}`. Add it to the allowlist.");
        }
        let mut cmd = std::process::Command::new(prog);
        cmd.args(args).current_dir(
            cwd.map(|c| self.workdir.join(c))
                .unwrap_or_else(|| self.workdir.clone()),
        );
        let output = cmd.output().with_context(|| format!("run `{prog}`"))?;
        Ok(CmdOut {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into(),
            stderr: String::from_utf8_lossy(&output.stderr).into(),
        })
    }

    fn llm(&mut self) -> &mut dyn LlmClient {
        self.llm.as_mut()
    }
}
