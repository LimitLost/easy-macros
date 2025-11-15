# Easy Macros

[![Crates.io](https://img.shields.io/crates/v/easy-macros.svg)](https://crates.io/crates/easy-macros)
[![Documentation](https://docs.rs/easy-macros/badge.svg)](https://docs.rs/easy-macros)
[![License](https://img.shields.io/crates/l/easy-macros.svg)](https://github.com/LimitLost/easy-macros/blob/master/LICENSE)

Toolkit for building Rust procedural macros with automatic error context generation and powerful utility macros.

## Features

For detailed feature documentation, usage examples, and API reference, please visit:

- **[crates.io/crates/easy-macros](https://crates.io/crates/easy-macros)** - Short version
- **[docs.rs/easy-macros](https://docs.rs/easy-macros)** - Complete API documentation and usage guides

### Quick Overview

**Easy Macros** provides:

1. **Automatic Error Context** - Works in any Rust project, automatically adds `.with_context()` to all `?` operators
2. **Attribute Pattern Matching** - Extract and validate attributes using intuitive patterns
3. **Exhaustive AST Traversal** - Generate recursive handlers for all `syn` types
4. **Helper Utilities** - Token stream builders, parsing helpers, crate finding, and more
5. **Result Type for Proc Macros** - Use `anyhow::Result<TokenStream>` with automatic error conversion

## Project Structure

This repository is organized as a Cargo workspace containing multiple crates:

- **`-main-crate/`** - The main `easy-macros` crate (re-exports all functionality)
- **`all-syntax-cases/`** - Exhaustive AST traversal macro
- **`always-context/`** - Automatic error context generation
- **`attributes/`** - Attribute pattern matching utilities
- **`anyhow-result/`** - Result type wrapper for proc macros
- **`helpers/`** - Helper utilities for proc macro development
- **Supporting crates** - Internal implementations and build tools

## Contributing

Contributions are welcome! Here's how you can help:

### Reporting Issues

If you find a bug or have a feature request:

1. Check the [issue tracker](https://github.com/LimitLost/easy-macros/issues) to see if it's already reported (you can upvote existing issues to increase priority)
2. If not, [open a new issue](https://github.com/LimitLost/easy-macros/issues/new) with:
   - A clear description of the problem or feature
   - Steps to reproduce (for bugs)
   - Expected vs. actual behavior
   - Relevant code snippets or error messages

### Submitting Pull Requests

1. **Fork** the repository and create a new branch for your changes
2. **Make your changes** following the existing code style
3. **Test** your changes thoroughly
4. **Update documentation** if you're adding new features or changing APIs
5. **Run the preparation script**: `./scripts/pr.sh` - To perform all necessary checks and generation before submitting a PR
6. **Commit** with clear, descriptive messages
7. **Submit a pull request** with:
   - A clear description of what you've changed and why
   - Reference to any related issues
   - Examples of the new functionality (if applicable)

### Code Style

- Follow standard Rust conventions and idioms
- Use `cargo fmt` to format your code
- Run `cargo clippy` and address any warnings
- Add doc comments for public APIs
- Include tests for new functionality

## License

This project is licensed under the **Apache License, Version 2.0**.

You may obtain a copy of the License at:

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

See the [LICENSE](LICENSE) file for the full license text.
