use std::fs;

use anyhow::{Context, Result};
use clap::Parser;
use clipboard::{ClipboardContext, ClipboardProvider};
use glob::glob;
use tiktoken_rs::cl100k_base;

#[derive(Parser)]
#[clap(version = "0.2.1")]
struct Opt {
    /// Glob patterns to process
    patterns: Vec<String>,

    /// Optional token limit
    #[clap(short = 't', long, default_value = "32000")]
    token_limit: usize,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let mut content = String::new();
    let mut current_token_count = 0;

    let bpe = cl100k_base().unwrap();

    for pattern in &opt.patterns {
        for entry in glob(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))? {
            match entry {
                Ok(path) => {
                    if path.is_dir() {
                        continue;
                    }
                    let file_content = fs::read_to_string(&path)
                        .with_context(|| format!("Failed to read file: {}", path.display()))?;

                    let file_context = format!(
                        "File name: \"{}\"\n\nFile contents: \"\"\"\n{}\"\"\"\n----------\n\n",
                        path.display(),
                        file_content
                    );

                    let new_token_count = bpe.encode_with_special_tokens(&file_context).len();
                    if current_token_count + new_token_count > opt.token_limit {
                        return Err(anyhow::anyhow!("Error: Token limit exceeded"));
                    }

                    content.push_str(&file_context);
                    current_token_count += new_token_count;
                }
                Err(e) => return Err(anyhow::anyhow!("Glob error: {:?}", e)),
            }
        }
    }

    if current_token_count == 0 {
        return Err(anyhow::anyhow!(
            "Error: No tokens found in the provided files"
        ));
    }

    let mut ctx: ClipboardContext =
        ClipboardProvider::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    ctx.set_contents(content)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("Success! Token count: {}", current_token_count);

    Ok(())
}
