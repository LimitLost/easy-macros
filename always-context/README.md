# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-macros-always-context.svg)](https://crates.io/crates/easy-macros-always-context)

`#[always_context]` attribute automatically adds `.with_context(context!())` to all `?` operators that don't already have context, eliminating the need to manually add context to every fallible operation.

## Basic Usage

```rust
use always_context::always_context;
use anyhow::Result;

#[always_context]
fn process_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(path)?; // Automatically gets context
    let processed = content.trim().to_uppercase();
    Ok(processed)
}
```

## How It Works

The `#[always_context]` attribute automatically transforms:

```rust
// From this:
let result = operation()?;

// To this:
let result = operation().with_context(context!("operation()"))?;
```

Context includes:

- Function call with arguments
- File location and line number
- Formatted argument values

## Attributes

### Function-level Control

- `#[no_context]` - Disable context generation entirely
- `#[no_context_inputs]` - Add context but exclude function arguments
- `#[enable_context]` - Re-enable context (useful in macros where it's auto-disabled)

### Argument-level Control

- `#[context(display)]` - Use `Display` instead of `Debug` for argument formatting
- `#[context(.method())]` - Call method on argument before displaying
- `#[context(tokens)]` - Format as token stream (for proc-macro arguments)
- `#[context(ignore)]` - Exclude this argument from context

## Requirements

- Function must return `anyhow::Result<T>` or `Result<T, UserFriendlyError>`
- Only processes `?` operators that don't already have context methods

## Unsupported Syntax

These expressions before `?` require manual `.with_context()`:

- Blocks: `{ expr }?`
- Control flow: `if ... {}?`, `match ... {}?`, `while ... {}?`, `for ... {}?`, `loop { ... }?`
- Field access: `obj.field?`
- Macros: `macro!()?`

## Features

- `easy-sql` - Adds `#[context(not_sql)]` attribute for SQL macro integration
