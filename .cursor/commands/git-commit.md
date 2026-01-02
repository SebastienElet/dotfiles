# Git Create Commit

## Overview

Create a short, focused commit message and commit staged changes.

## Steps

1. **Review changes**
    - Check the diff: `git diff --cached` (if changes are staged) or `git diff` (if unstaged)
    - Understand what changed and why
2. **Ask for GitHub issue number (optional)**
    - Check the branch name for a GitHub issue number
    - If a GitHub issue number (e.g., #123, #456) is not already available in the chat or commit context, optionally ask the user if they want to include one
    - This is optional - commits can be made without an issue number
3. **Stage changes (if not already staged)**
    - `git add -A`
4. **Create short commit message**
    - Base the message on the actual changes in the diff
    - Example: `git commit -m "fix(auth): handle expired token refresh"`
    - Example with GitHub issue: `git commit -m "fix(auth): handle expired token refresh (#123)"`

## Template

- `git commit -m "<type>(<scope>): <short summary>"`
- With GitHub issue: `git commit -m "<type>(<scope>): <short summary> (#<issue-number>)"`

## Rules

- **Length:** <= 72 characters
- **Imperative mood:** Use "fix", "add", "update" (not "fixed", "added", "updated")
- **Capitalize:** First letter of summary should be capitalized
- **No period:** Don't end the subject line with a period
- **Describe why:** Not just what - "fix stuff" is meaningless

## Common Commit Types

- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation only changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements
- `ci`: CI/CD changes
- `build`: Build system changes

## Process

1. First, check if there are any changes to commit:
   ```bash
   git status --porcelain
   ```

2. If there are unstaged changes and no staged changes, ask the user if they want to stage all changes:
   ```bash
   git diff --stat
   ```

3. Show the diff (staged or unstaged) to understand what changed:
   ```bash
   git diff --cached  # for staged changes
   git diff           # for unstaged changes
   ```

4. Extract GitHub issue number from branch name:
   ```bash
   git branch --show-current
   ```
   - Look for patterns like: `#123`, `feat/#123`, `fix/#456`, `issue-123`

5. If no GitHub issue number found in branch name, optionally ask the user if they want to include one

6. If changes are not staged, ask user if they want to stage all changes with `git add -A`

7. Based on the diff, suggest an appropriate commit type and scope:
   - Analyze the changed files to determine scope (e.g., `auth`, `api`, `ui`, `db`)
   - Determine the type based on the nature of changes

8. Create a concise summary (imperative mood, <= 72 chars total including type and scope)

9. Show the proposed commit message to the user for confirmation before executing:
   ```bash
   git commit -m "<commit-message>"
   ```

## Example Workflow

```
User: Create a commit

Agent:
1. Checks git status
2. Shows diff summary
3. Detects branch name "feat/#123"
4. Analyzes changes: "Modified auth.ts to handle expired tokens"
5. Proposes: "fix(auth): handle expired token refresh (#123)"
6. Confirms with user
7. Executes: git commit -m "fix(auth): handle expired token refresh (#123)"
```
