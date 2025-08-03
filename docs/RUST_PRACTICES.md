# Rust Development Practices

## String Formatting
- Always use inline format arguments (Rust 1.58+)
- ❌ `format!("Hello {}", name)` 
- ✅ `format!("Hello {name}")`
- Apply to: `format!`, `println!`, `eprintln!`, `write!`, etc.

## Common Clippy Fixes
- `uninlined_format_args`: Use `{variable}` instead of `{}, variable`
- Include context in error messages: `eprintln!("Warning: {repo_name}: {error}")`

## Code Quality Workflow
1. Write with inline formatting from start
2. `cargo fmt` → `cargo clippy` → `cargo test`
3. Commit only when all checks pass clean