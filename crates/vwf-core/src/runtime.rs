use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Runtime mediates side effects so the engine can be tested.
pub trait Runtime {
    fn workdir(&self) -> &Path;

    fn ensure_dir(&mut self, rel: &str) -> Result<()>;
    fn write_text(&mut self, rel: &str, content: &str) -> Result<()>;
    fn read_text(&self, rel: &str) -> Result<String>;

    fn run_command(
        &mut self,
        _program: &str,
        _args: &[String],
        _cwd: Option<&str>,
    ) -> Result<CommandOutput>;

    fn llm(&mut self) -> &mut dyn LlmClient;
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait LlmClient {
    fn generate(&mut self, req: LlmRequest) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub system: String,
    pub user: String,
    pub provider: String,
}

/// Real filesystem runtime.
pub struct FsRuntime {
    workdir: PathBuf,
    llm: Box<dyn LlmClient>,
    /// Allowlist of executables (exact match).
    pub command_allowlist: std::collections::BTreeSet<String>,
}

impl FsRuntime {
    pub fn new(workdir: impl Into<PathBuf>, llm: Box<dyn LlmClient>) -> Self {
        Self {
            workdir: workdir.into(),
            llm,
            command_allowlist: Default::default(),
        }
    }

    fn abs(&self, rel: &str) -> PathBuf {
        self.workdir.join(rel)
    }
}

impl Runtime for FsRuntime {
    fn workdir(&self) -> &Path {
        &self.workdir
    }

    fn ensure_dir(&mut self, rel: &str) -> Result<()> {
        let p = self.abs(rel);
        std::fs::create_dir_all(&p).with_context(|| format!("create_dir_all {}", p.display()))?;
        Ok(())
    }

    fn write_text(&mut self, rel: &str, content: &str) -> Result<()> {
        let p = self.abs(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&p, content.as_bytes()).with_context(|| format!("write {}", p.display()))?;
        Ok(())
    }

    fn read_text(&self, rel: &str) -> Result<String> {
        let p = self.abs(rel);
        let bytes = std::fs::read(&p).with_context(|| format!("read {}", p.display()))?;
        Ok(String::from_utf8(bytes)?)
    }

    fn run_command(
        &mut self,
        program: &str,
        args: &[String],
        cwd: Option<&str>,
    ) -> Result<CommandOutput> {
        if !self.command_allowlist.is_empty() && !self.command_allowlist.contains(program) {
            anyhow::bail!("Command not allowed: `{program}`. Add it to the allowlist explicitly.");
        }
        let mut cmd = std::process::Command::new(program);
        cmd.args(args);
        if let Some(cwd) = cwd {
            cmd.current_dir(self.abs(cwd));
        } else {
            cmd.current_dir(&self.workdir);
        }
        let output = cmd.output().with_context(|| format!("run `{program}`"))?;
        Ok(CommandOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    fn llm(&mut self) -> &mut dyn LlmClient {
        self.llm.as_mut()
    }
}

/// Dry-run runtime: records intended writes without touching disk.
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

    fn read_text(&self, _rel: &str) -> Result<String> {
        anyhow::bail!("DryRunRuntime cannot read files (provide fixtures via FsRuntime in tests)")
    }

    fn run_command(
        &mut self,
        program: &str,
        _args: &[String],
        _cwd: Option<&str>,
    ) -> Result<CommandOutput> {
        Ok(CommandOutput {
            status: 0,
            stdout: format!("[dry-run] would run {program}"),
            stderr: String::new(),
        })
    }

    fn llm(&mut self) -> &mut dyn LlmClient {
        self.llm.as_mut()
    }
}

/// Mock LLM: returns a configured response (or echoes the user prompt).
pub struct MockLlmClient {
    pub canned: Option<String>,
}

impl MockLlmClient {
    pub fn canned(s: impl Into<String>) -> Self {
        Self {
            canned: Some(s.into()),
        }
    }
    pub fn echo() -> Self {
        Self { canned: None }
    }
}

impl LlmClient for MockLlmClient {
    fn generate(&mut self, req: LlmRequest) -> Result<String> {
        Ok(self
            .canned
            .clone()
            .unwrap_or_else(|| format!("SYSTEM:\n{}\n\nUSER:\n{}", req.system, req.user)))
    }
}
