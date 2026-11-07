use anyhow::{anyhow, Context};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

pub trait Executor {
    fn run(&self, args: &[&str]) -> anyhow::Result<String>;
    fn stream(&self, args: &[&str], on_line: &mut dyn FnMut(&str)) -> anyhow::Result<()>;
}

pub struct Kubectl;

impl Executor for Kubectl {
    fn run(&self, args: &[&str]) -> anyhow::Result<String> {
        let output = Command::new("kubectl")
            .args(args)
            .output()
            .with_context(|| format!("failed to execute kubectl {}", args.join(" ")))?;

        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let message = if stderr.is_empty() { stdout } else { stderr };
            return Err(anyhow!(
                "kubectl command failed{}{}",
                if message.is_empty() { "" } else { ": " },
                message
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn stream(&self, args: &[&str], on_line: &mut dyn FnMut(&str)) -> anyhow::Result<()> {
        let mut child = Command::new("kubectl")
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("failed to start kubectl {}", args.join(" ")))?;

        let stdout = child
            .stdout
            .take()
            .context("kubectl did not provide stdout for streaming")?;
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = line
                .with_context(|| format!("failed to read kubectl output for {}", args.join(" ")))?;
            on_line(&line);
        }

        let status = child
            .wait_with_output()
            .with_context(|| format!("failed waiting for kubectl {}", args.join(" ")))?;

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&status.stdout).trim().to_string();
            let message = if stderr.is_empty() { stdout } else { stderr };
            return Err(anyhow!(
                "kubectl stream failed{}{}",
                if message.is_empty() { "" } else { ": " },
                message
            ));
        }

        Ok(())
    }
}
