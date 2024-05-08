use std::fs;
use std::sync::Mutex;

use anyhow::{Context, Result};
use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use glob::glob;
use rayon::prelude::*;
use tiktoken_rs::cl100k_base;

#[derive(Parser)]
#[clap(version = "0.4.1")]
struct Opt {
    /// Glob patterns to process
    patterns: Vec<String>,
    /// Optional token limit
    #[clap(short = 't', long, default_value = "32000")]
    token_limit: usize,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let content = Mutex::new(String::new());
    let current_token_count = Mutex::new(0);
    let bpe = cl100k_base().unwrap();

    opt.patterns.par_iter().try_for_each(|pattern| {
        glob(pattern)
            .with_context(|| format!("Invalid glob pattern: {}", pattern))?
            .par_bridge()
            .try_for_each(|entry| {
                let path = entry.with_context(|| "Glob error".to_string())?;
                if path.is_dir() {
                    return Ok(());
                }
                let file_content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read file: {}", path.display()))?;
                let file_context = format!(
                    r#"File name: "{}"\n\nFile contents: """\n{}"""\n----------\n\n"#,
                    path.display(),
                    file_content
                );
                let new_token_count = bpe.encode_with_special_tokens(&file_context).len();
                let mut current_token_count = current_token_count.lock().unwrap();

                if *current_token_count + new_token_count > opt.token_limit {
                    return Err(anyhow::anyhow!("Error: Token limit exceeded"));
                }
                if *current_token_count > 0 {
                    content.lock().unwrap().push_str(&file_context);
                    *current_token_count += new_token_count;
                }

                Ok(())
            })
    })?;

    let current_token_count = *current_token_count.lock().unwrap();
    if current_token_count == 0 {
        return Err(anyhow::anyhow!(
            "Error: No tokens found in the provided files"
        ));
    }

    let mut ctx: ClipboardContext =
        ClipboardProvider::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let clipboard_content = content.into_inner().unwrap().replace("\\n", "\r\n");
    ctx.set_contents(clipboard_content)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("Success! Token count: {}", current_token_count);
    Ok(())
}
