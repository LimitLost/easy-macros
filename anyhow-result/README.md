# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-macros-anyhow-result.svg)](https://crates.io/crates/easy-macros-anyhow-result)

Proc-macro attribute enabling `anyhow::Result<TokenStream>` return types for procedural macros.

## Error Handling

When your function returns an `Err`, `anyhow_result` automatically converts it to appropriate `compile_error!` tokens
