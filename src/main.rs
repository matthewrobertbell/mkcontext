use std::fs;
use std::process::Command;
use std::sync::Mutex;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use glob::glob;
use rayon::prelude::*;
use tiktoken_rs::cl100k_base;

#[derive(Parser)]
#[clap(version = "0.4.3")]
struct Opt {
    /// Glob patterns to process
    patterns: Vec<String>,
    /// Optional token limit
    #[clap(short = 't', long, default_value = "32000")]
    token_limit: usize,
    /// Commands to execute and include in the output
    #[clap(long = "command", short = 'c')]
    commands: Vec<String>,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let content = Mutex::new(String::new());
    let current_token_count = Mutex::new(0);
    let bpe = cl100k_base().unwrap();

    // Process files
    opt.patterns.par_iter().try_for_each(|pattern| {
        glob(pattern)
            .with_context(|| format!("Invalid glob pattern: {}", pattern))?
            .par_bridge()
            .try_for_each(|entry| {
                let path = entry.with_context(|| "Glob error".to_string())?;
                if path.is_dir() {
                    return Ok(());
                }
                let file_content = match fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::InvalidData {
                            println!("Ignored non-UTF8 file: {}", path.display());
                            return Ok(());
                        } else {
                            return Err(anyhow!("Failed to read file: {}", path.display()))
                                .with_context(|| e.to_string());
                        }
                    }
                };
                let file_context = format!(
                    r#"File name: "{}"

File contents: """
{}"""
----------

"#,
                    path.display(),
                    file_content
                );
                add_content(
                    &content,
                    &current_token_count,
                    &file_context,
                    &bpe,
                    opt.token_limit,
                )?;
                Ok(())
            })
    })?;

    // Process commands
    for cmd in &opt.commands {
        let output = execute_command(cmd)?;
        let command_context = format!(
            r#"Command: "{}"

Command output: """
{}"""
----------

"#,
            cmd, output
        );
        add_content(
            &content,
            &current_token_count,
            &command_context,
            &bpe,
            opt.token_limit,
        )?;
    }

    let current_token_count = *current_token_count.lock().unwrap();
    if current_token_count == 0 {
        return Err(anyhow::anyhow!(
            "Error: No tokens found in the provided files or command outputs"
        ));
    }

    let mut ctx: ClipboardContext =
        ClipboardProvider::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let clipboard_content = content.into_inner().unwrap().replace('\n', "\r\n");
    ctx.set_contents(clipboard_content)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("Success! Token count: {}", current_token_count);
    Ok(())
}

fn add_content(
    content: &Mutex<String>,
    current_token_count: &Mutex<usize>,
    new_content: &str,
    bpe: &tiktoken_rs::CoreBPE,
    token_limit: usize,
) -> Result<()> {
    let new_token_count = bpe.encode_with_special_tokens(new_content).len();
    let mut current_token_count = current_token_count.lock().unwrap();

    if *current_token_count + new_token_count > token_limit {
        return Err(anyhow::anyhow!("Error: Token limit exceeded"));
    }
    if new_token_count > 0 {
        content.lock().unwrap().push_str(new_content);
        *current_token_count += new_token_count;
    }
    Ok(())
}

fn execute_command(cmd: &str) -> Result<String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .with_context(|| format!("Failed to execute command: {}", cmd))?;

    let exit_status = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    let result = format!(
        "Exit Status: {}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
        exit_status, stdout, stderr
    );

    Ok(result)
}
