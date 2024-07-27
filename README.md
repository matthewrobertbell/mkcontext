# mkcontext

`mkcontext` is a command-line tool that generates context from files and command outputs, useful for large language models.

## Features

- Process multiple files using glob patterns
- Execute shell commands and include their output
- Tokenize content using OpenAI's tiktoken
- Copy the generated context to the clipboard
- Limit the total number of tokens

## Installation

To install `mkcontext`, you need to have Rust and Cargo installed on your system. Then, you can install it using:

```
cargo install mkcontext
```

## Usage

```
mkcontext [OPTIONS] <PATTERNS>...
```

### Arguments

- `<PATTERNS>...`: Glob patterns to process (e.g., `*.rs`, `src/**/*.js`)

### Options

- `-t, --token-limit <TOKEN_LIMIT>`: Optional token limit (default: 32000)
- `-c, --command <COMMAND>`: Commands to execute and include in the output (can be used multiple times)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Examples

1. Process all Rust files in the current directory:

   ```
   mkcontext *.rs
   ```

2. Process all JavaScript files in the `src` directory and its subdirectories, with a token limit of 16000:

   ```
   mkcontext -t 16000 src/**/*.js
   ```

3. Include the output of `git status` command in the context:

   ```
   mkcontext *.rs -c "git status"
   ```

4. Process Python files and include outputs from multiple commands:

   ```
   mkcontext *.py -c "pip list" -c "python --version"
   ```

5. Combine file processing and command execution:

   ```
   mkcontext src/**/*.rs tests/**/*.rs -c "cargo test" -c "rustc --version"
   ```

## How it works

1. `mkcontext` processes the specified files matching the glob patterns.
2. It executes any specified commands and captures their output.
3. The content from files and command outputs is tokenized using the cl100k_base tokenizer.
4. If the total number of tokens exceeds the specified limit, an error is returned.
5. The generated context is copied to the clipboard.

## Note

The tool uses the system clipboard, so make sure you have the appropriate clipboard drivers installed for your operating system.

## License

This project is licensed under the MIT License.