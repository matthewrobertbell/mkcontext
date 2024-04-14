
# mkcontext - Context Builder for ChatGPT

## Overview

`mkcontext` is a command-line tool written in Rust, designed to concatenate the contents of multiple files into a single string, primarily for building context to be used with OpenAI's ChatGPT. It supports glob patterns for file selection and maintains a token count to avoid exceeding a specified limit. The final concatenated string is copied to the clipboard for easy pasting.

## Features

- **Glob Pattern Support**: Accepts one or more glob patterns to specify the files to process.
- **Token Count Limit**: Supports an optional token count limit with a default of 32,000 tokens.
- **Clipboard Integration**: Automatically copies the final concatenated string to the clipboard.
- **Error Handling**: Utilizes `anyhow` for robust error management.

## Installation

To use `mkcontext`, you need to have Rust and Cargo installed. Follow these steps to install and build the tool:

1. Clone the repository:
   ```bash
   git clone https://github.com/matthewrobertbell/mkcontext.git
   cd mkcontext
   ```

2. Build the project using Cargo:
   ```bash
   cargo build --release
   ```

3. The executable will be available in `target/release/mkcontext`.

## Cargo Install Method

Alternatively, you can install `mkcontext` directly using Cargo. This method will download and compile the source code automatically and place the executable in your Cargo bin path.

1. Install directly from the repository:
   ```bash
   cargo install --git https://github.com/matthewrobertbell/mkcontext.git
   ```

2. Once installed, `mkcontext` can be run from anywhere on your system if your Cargo bin path is in your system's PATH.

3. To update `mkcontext` in the future, simply rerun the install command. Cargo will replace the old version with the new one.

Note: Ensure that you have Rust and Cargo installed and updated to the latest version for this method to work seamlessly.

## Usage

Run `mkcontext` with the required glob patterns. Optionally, specify a custom token limit using `--token-limit`.

```bash
mkcontext "path/to/files/*.txt" "another/path/*.md" --token-limit 50000
```

If the token limit is exceeded, `mkcontext` will output an error and terminate. Upon successful execution, it prints the token count and copies the content to the clipboard.

## Error Handling

The tool uses the `anyhow` crate for error handling, providing clear error messages for issues such as:

- Invalid glob patterns.
- File read errors.
- Exceeding the token limit.

## Contributing

Contributions to `mkcontext` are welcome. Please ensure that your code adheres to the project's standards and includes appropriate tests.

## License

`mkcontext` is licensed under the MIT License. See the LICENSE file for more details.

## Contact

For questions or suggestions regarding `mkcontext`, please open an issue on the GitHub repository.
