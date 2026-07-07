use std::{fs, path::Path};

use anyhow::{Context, Result};

const CFG_TOKEN: &str = concat!("#[cfg(", "test)]");
const MOD_TOKEN: &str = concat!("mod ", "tests");
const ATTR_TOKEN: &str = concat!("#[", "test]");

pub fn run(root: impl AsRef<Path>) -> Result<()> {
    let mut violations = Vec::new();
    visit(root.as_ref(), &mut violations)?;
    report(violations)
}

fn visit(path: &Path, violations: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry?;
        inspect_path(&entry.path(), violations)?;
    }
    Ok(())
}

fn inspect_path(path: &Path, violations: &mut Vec<String>) -> Result<()> {
    match path.is_dir() {
        true => visit(path, violations),
        false => inspect_file(path, violations),
    }
}

fn inspect_file(path: &Path, violations: &mut Vec<String>) -> Result<()> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return Ok(());
    }
    let source = fs::read_to_string(path)?;
    push_violation(path, &source, CFG_TOKEN, violations);
    push_violation(path, &source, MOD_TOKEN, violations);
    push_violation(path, &source, ATTR_TOKEN, violations);
    Ok(())
}

fn push_violation(path: &Path, source: &str, token: &str, violations: &mut Vec<String>) {
    if source.contains(token) {
        violations.push(format!("{} contains forbidden test token", path.display()));
    }
}

fn report(violations: Vec<String>) -> Result<()> {
    match violations.is_empty() {
        true => Ok(()),
        false => anyhow::bail!("{}", violations.join("\n")),
    }
}
