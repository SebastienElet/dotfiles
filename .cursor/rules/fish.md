---
description: Fish shell conventions
globs: fish/**/*.fish
---

# Fish Shell Conventions

Follow official Fish shell conventions:

- Use `snake_case` for function and variable names
- Prefix private/internal functions with `_` (e.g., `_fzf_search_directory`)
- Use `set -g` for global variables, `set -l` for local
- Prefer Fish builtins over external commands when possible
- Use `test` instead of `[` for conditionals
- Use `string` builtin for string manipulation
- Keep functions focused and single-purpose

## Example Function Structure

```fish
function my_function --description "Brief description"
    # Implementation
end
```
