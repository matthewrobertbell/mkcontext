use std::fs;
use std::process::Command;
use std::sync::Mutex;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use globset::{Glob, GlobSetBuilder};
use tiktoken_rs::cl100k_base;
use walkdir::WalkDir;

#[derive(Parser)]
#[clap(version = "0.5.1")]
struct Opt {
    /// Optional token limit
    #[clap(short = 't', long, default_value = "200000")]
    token_limit: usize,
    /// Commands to execute and include in the output
    #[clap(long = "command", short = 'c')]
    commands: Vec<String>,
    /// Globs to search / exclude
    #[clap(long = "glob", short = 'g')]
    globs: Vec<String>,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let content = Mutex::new(String::new());
    let current_token_count = Mutex::new(0);
    let bpe = cl100k_base().unwrap();

    // Process files

    let mut include_builder = GlobSetBuilder::new();
    let mut exclude_builder = GlobSetBuilder::new();

    for glob in &opt.globs {
        if let Some(stripped) = glob.strip_prefix('!') {
            exclude_builder.add(
                Glob::new(stripped).map_err(|e| anyhow!("Invalid exclude glob pattern: {}", e))?,
            );
        } else {
            include_builder
                .add(Glob::new(glob).map_err(|e| anyhow!("Invalid include glob pattern: {}", e))?);
        }
    }

    let include_set = include_builder
        .build()
        .map_err(|e| anyhow!("Failed to build include globset: {}", e))?;
    let exclude_set = exclude_builder
        .build()
        .map_err(|e| anyhow!("Failed to build exclude globset: {}", e))?;

    for entry in WalkDir::new(".") {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let path_str = path.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
        let path_str = path_str.strip_prefix("./").unwrap_or(path_str);

        if (include_set.is_match(path_str)) && !exclude_set.is_match(path_str) {
            let file_content = match fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::InvalidData {
                        eprintln!("Ignored non-UTF8 file: {}", path.display());
                    } else {
                        eprintln!("Failed to read file: {}", path.display());
                    }
                    continue;
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
            let new_token_count = add_content(
                &content,
                &current_token_count,
                &file_context,
                &bpe,
                opt.token_limit,
            )?;
            println!("Processed file: {path_str:<70} ({new_token_count:<6} tokens)");
        }
    }

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
        let new_token_count = add_content(
            &content,
            &current_token_count,
            &command_context,
            &bpe,
            opt.token_limit,
        )?;
        println!("Executed command: {cmd:<67}  ({new_token_count:<6} tokens)");
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
) -> Result<usize> {
    let new_token_count = bpe.encode_with_special_tokens(new_content).len();
    let mut current_token_count = current_token_count.lock().unwrap();

    if *current_token_count + new_token_count > token_limit {
        return Err(anyhow::anyhow!("Error: Token limit exceeded"));
    }
    if new_token_count > 0 {
        content.lock().unwrap().push_str(new_content);
        *current_token_count += new_token_count;
    }
    Ok(new_token_count)
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

    Ok(format!(
        "Exit Status: {}\n\nSTDOUT:\n{}\n\nSTDERR:\n{}",
        exit_status, stdout, stderr
    ))
}
