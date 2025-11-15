# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-macros-all-syntax-cases.svg)](https://crates.io/crates/easy-macros-all-syntax-cases)

A procedural macro that creates complete recursive handler functions for all syn AST types (Item, Expr, Stmt, Pat, Type, etc.). It generates match arms for every variant and routes to your custom handlers.
