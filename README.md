# mkcontext

`mkcontext` is a command-line tool that generates context from files and command outputs, useful for large language models.

## Features

- Process multiple files using glob patterns
- Execute shell commands and include their output
- Tokenize content using OpenAI's tiktoken
- Copy the generated context to the clipboard
- Limit the total number of tokens
- Include or exclude files based on glob patterns

## Installation

To install `mkcontext`, you need to have Rust and Cargo installed on your system. Then, you can install it using:

```
cargo install mkcontext
```

## Usage

```
mkcontext [OPTIONS]
```

### Options

- `-t, --token-limit <TOKEN_LIMIT>`: Optional token limit (default: 200000)
- `-c, --command <COMMAND>`: Commands to execute and include in the output (can be used multiple times)
- `-g, --glob <GLOB>`: Glob patterns to include or exclude files (can be used multiple times)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Examples

1. Process all Rust files in the current directory:

   ```
   mkcontext -g "*.rs"
   ```

2. Process all JavaScript files in the `src` directory and its subdirectories, with a token limit of 16000:

   ```
   mkcontext -t 16000 -g "src/**/*.js"
   ```

3. Include the output of `git status` command in the context:

   ```
   mkcontext -g "*.rs" -c "git status"
   ```

4. Process Python files and include outputs from multiple commands:

   ```
   mkcontext -g "*.py" -c "pip list" -c "python --version"
   ```

5. Combine file processing and command execution:

   ```
   mkcontext -g "src/**/*.rs" -g "tests/**/*.rs" -c "cargo test" -c "rustc --version"
   ```

6. Exclude specific directories or file patterns:

   ```
   mkcontext -g "src/**/*.js" -g "!node_modules/**" -g "!*.log"
   ```

## How it works

1. `mkcontext` processes the specified files matching the include glob patterns, excluding any paths or patterns specified by exclude globs.
2. It executes any specified commands and captures their output.
3. The content from files and command outputs is tokenized using the cl100k_base tokenizer.
4. If the total number of tokens exceeds the specified limit, an error is returned.
5. The generated context is copied to the clipboard.

## Glob Patterns

The `-g` or `--glob` option allows you to specify patterns to include or exclude files during processing. This is useful for selecting specific file types or excluding certain directories or files.

- To include files, use a regular glob pattern: `-g "src/**/*.rs"`
- To exclude files or directories, prefix the glob with `!`: `-g "!node_modules/**"`

You can use multiple `-g` options to specify several include and exclude patterns:

```
mkcontext -g "src/**/*.js" -g "!node_modules/**" -g "!*.log" -g "!build/**"
```

This will process all JavaScript files in the `src` directory and its subdirectories, while ignoring the `node_modules` directory, any `.log` files, and the `build/` directory.

## Note

The tool uses the system clipboard, so make sure you have the appropriate clipboard drivers installed for your operating system.

## License

This project is licensed under the MIT License.