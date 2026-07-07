use std::{fs, path::Path};

use anyhow::{Context, Result};

pub fn run(root: impl AsRef<Path>, max: usize) -> Result<()> {
    let mut violations = Vec::new();
    visit(root.as_ref(), max, &mut violations)?;
    report(violations)
}

fn visit(path: &Path, max: usize, violations: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
        let entry = entry?;
        inspect_path(&entry.path(), max, violations)?;
    }
    Ok(())
}

fn inspect_path(path: &Path, max: usize, violations: &mut Vec<String>) -> Result<()> {
    match path.is_dir() {
        true => visit(path, max, violations),
        false => inspect_file(path, max, violations),
    }
}

fn inspect_file(path: &Path, max: usize, violations: &mut Vec<String>) -> Result<()> {
    if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return Ok(());
    }
    let source = fs::read_to_string(path)?;
    scan_functions(path, &source, max, violations);
    Ok(())
}

fn scan_functions(path: &Path, source: &str, max: usize, violations: &mut Vec<String>) {
    for block in source.split("\nfn ").skip(1) {
        inspect_function(path, block, max, violations);
    }
}

fn inspect_function(path: &Path, block: &str, max: usize, violations: &mut Vec<String>) {
    let name = function_name(block);
    let body = function_body(block);
    let score = complexity_score(body);
    if score > max {
        violations.push(format!(
            "{}::{name} complexity {score} > {max}",
            path.display()
        ));
    }
}

fn function_name(block: &str) -> &str {
    block
        .split(['(', '<', ' '])
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown")
}

fn function_body(block: &str) -> &str {
    block.split("\nfn ").next().unwrap_or(block)
}

fn complexity_score(body: &str) -> usize {
    1 + count_token(body, " if ")
        + count_token(body, " match ")
        + count_token(body, "&&")
        + count_token(body, "||")
        + count_token(body, "?")
}

fn count_token(body: &str, token: &str) -> usize {
    body.matches(token).count()
}

fn report(violations: Vec<String>) -> Result<()> {
    match violations.is_empty() {
        true => Ok(()),
        false => anyhow::bail!("{}", violations.join("\n")),
    }
}
