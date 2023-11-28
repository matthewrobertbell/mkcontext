use anyhow::{Context, Result};
use clap::{App, Arg};
use clipboard::{ClipboardContext, ClipboardProvider};
use glob::glob;
use tiktoken_rs::cl100k_base;

use std::fs;

fn main() -> Result<()> {
    let matches = App::new("mkcontext")
        .version("1.0")
        .author("Your Name")
        .about("Concatenates file contents based on glob patterns")
        .arg(
            Arg::with_name("patterns")
                .multiple(true)
                .takes_value(true)
                .required(true)
                .help("Glob patterns to process"),
        )
        .arg(
            Arg::with_name("token_limit")
                .long("token-limit")
                .takes_value(true)
                .default_value("32000")
                .help("Optional token limit"),
        )
        .get_matches();

    let mut content = String::new();
    let token_limit: usize = matches
        .value_of("token_limit")
        .unwrap()
        .parse()
        .with_context(|| "Invalid token limit")?;
    let mut current_token_count = 0;

    let bpe = cl100k_base().unwrap();

    if let Some(patterns) = matches.values_of("patterns") {
        for pattern in patterns {
            for entry in
                glob(pattern).with_context(|| format!("Invalid glob pattern: {}", pattern))?
            {
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
                        if current_token_count + new_token_count > token_limit {
                            return Err(anyhow::anyhow!("Error: Token limit exceeded"));
                        }

                        content.push_str(&file_context);
                        current_token_count += new_token_count;
                    }
                    Err(e) => return Err(anyhow::anyhow!("Glob error: {:?}", e)),
                }
            }
        }
    }

    let mut ctx: ClipboardContext =
        ClipboardProvider::new().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    ctx.set_contents(content)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    println!("Success! Token count: {}", current_token_count);

    Ok(())
}
