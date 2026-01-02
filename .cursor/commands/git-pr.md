# Create Pull Request

## Overview

Create a well-structured pull request with proper description following the project's PR template, labels, and reviewers.

## Steps

1. **Prepare branch**
   - Ensure all changes are committed
   - Push branch to remote
   - Verify branch is up to date with main
   - Get current branch name and remote repository info
2. **Write PR description**
   - Use the PR template from `.github/pull_request_template.md`
   - Summarize changes clearly
   - Include context and motivation
   - List any breaking changes
   - Add screenshots if UI changes
3. **Set up PR**
   - Create PR with descriptive title (in English)
   - Add appropriate labels (if needed)
   - Assign reviewers (if needed)
   - Link related issues


## PR Template

**IMPORTANT**: Always read the PR template from `.github/pull_request_template.md` at runtime. Do not copy the template content here - reference the file directly to ensure we always use the latest version.

The PR description must follow the exact format from `.github/pull_request_template.md`. Read that file when creating a PR to ensure the template is up to date.

## Process

1. **Check git status**
   ```bash
   git status
   ```
   - Ensure working directory is clean (all changes committed)
   - If there are uncommitted changes, ask user if they want to commit first

2. **Get current branch**
   ```bash
   git branch --show-current
   ```
   - Extract branch name for PR creation

3. **Check if branch is pushed**
   ```bash
   BRANCH=$(git branch --show-current)
   if git rev-parse --verify origin/$BRANCH >/dev/null 2>&1; then
     # Branch exists on remote, check for unpushed commits
     UNPUSHED=$(git log origin/$BRANCH..HEAD 2>/dev/null | wc -l | tr -d ' ')
     [ "$UNPUSHED" -gt 0 ] && echo "needs_push"
   else
     # Branch doesn't exist on remote, needs to be pushed
     echo "needs_push"
   fi
   ```
   - **Important**: The original command `git log origin/$(git branch --show-current)..HEAD 2>/dev/null | wc -l` fails for new branches because when `origin/<branch>` doesn't exist, the error is suppressed and `wc -l` returns 0, incorrectly indicating no unpushed commits. Always check if the remote branch exists first.
   - If branch has unpushed commits or doesn't exist on remote, push the branch:
     ```bash
     git push -u origin $(git branch --show-current)
     ```

4. **Get remote repository info**
   ```bash
   git remote get-url origin
   ```
   - Extract owner and repo name from remote URL
   - Format: `https://github.com/owner/repo.git` or `git@github.com:owner/repo.git`
   - Handle both formats: `git@github.com:owner/repo.git` → owner: `owner`, repo: `repo`
   - Handle format: `https://github.com/owner/repo.git` → owner: `owner`, repo: `repo`

5. **Get base branch (main)**
   ```bash
   git symbolic-ref refs/remotes/origin/HEAD | sed 's@^refs/remotes/origin/@@'
   ```
   - Default to `main` as the base branch if command fails

6. **Analyze changes for PR description**
   ```bash
   BASE_BRANCH=$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@' || echo "main")
   git diff origin/$BASE_BRANCH..HEAD --stat
   git log origin/$BASE_BRANCH..HEAD --oneline
   ```
   - Review changed files and commit messages to understand the scope
   - Extract issue numbers from commit messages or branch name (e.g., `#123`, `feat/#123`)

7. **Generate PR title**
   - Based on branch name and commits
   - **IMPORTANT**: PR titles must be in English
   - Format: `<Type>: <description in English>`
   - Examples:
     - `Feat: Add OAuth support`
     - `Fix: Handle token expiration`
     - `Refactor: Optimize query performance`
   - Map commit types to English:
     - `feat` → `Feat`
     - `fix` → `Fix`
     - `refactor` → `Refactor`
     - `chore` → `Chore`
     - `docs` → `Docs`
     - `style` → `Style`
     - `test` → `Test`
     - `perf` → `Perf`
     - `ci` → `CI`
     - `build` → `Build`

8. **Fill PR template**
   - **Read the template**: First, read `.github/pull_request_template.md` to get the current template structure
   - **Useful Links**: Extract issue numbers from branch name or commits. **If no issue numbers are found, remove the entire "Useful Links" section** (do not include an empty section)
   - **Description**: Summarize changes from commits/changes or ask user
   - **How to Test**: Ask user for testing steps or infer from changes
   - **Checklist**: Pre-fill based on what can be verified (tests, migrations, etc.)

9. **Create PR using GitHub API**
   - Use MCP GitHub tools to create the PR
   - Set base branch to `main` (or detected base branch)
   - Set head branch (current branch)
   - Include the filled PR template as body

10. **Optional: Add labels and reviewers**
    - Ask user if they want to add labels
    - Ask user if they want to assign reviewers
    - Use GitHub API to update PR if needed

## Rules

- **Always use the PR template**: Follow the exact format from `.github/pull_request_template.md`
- **Descriptive title**: Should clearly indicate what the PR does (must be in English)
- **Complete description**: Fill all sections of the template, even if brief
- **Link issues**: If branch name or commits reference issues, include them in the description. **If no issue numbers are found, remove the entire "Useful Links" section** (do not include an empty section with placeholders)
- **Check for breaking changes**: Review changes to identify any breaking changes
- **Verify base branch**: Ensure PR targets the base branch (default: `main`)

## Example Workflow

```
User: Create a PR

Agent:
1. Checks git status → working directory clean
2. Gets current branch → "feat/#123-add-oauth-support"
3. Checks if pushed → needs push
4. Pushes branch → "feat/#123-add-oauth-support"
5. Gets remote info → "github.com/SebastienElet/dotfiles"
6. Analyzes changes → 5 files changed, 3 commits
7. Extracts issue number → #123
8. Generates title → "Feat: Add OAuth support (#123)"
9. Reads PR template from .github/pull_request_template.md
10. Fills PR template:
    - Useful Links: Includes #123 from branch name
    - Description: "Add OAuth2 authentication support. Implemented OAuth2 flow with Google provider..."
    - Testing: "Test login flow with Google account..."
    - Note: If no issue number found, removes "Useful Links" section entirely
11. Creates PR via GitHub API
12. Confirms PR created with link
```

## GitHub API Integration

When creating the PR, use the MCP GitHub tools:
- `mcp_Github-personal_create_pull_request`: Create the PR with title, body, head, and base
- `mcp_Github-personal_update_issue`: If labels or reviewers need to be added (PRs are issues)

## Notes

- If the user hasn't committed changes, ask if they want to commit first
- If the branch isn't pushed, offer to push it
- Always verify the base branch is correct before creating PR
- The PR template is in English - maintain that language in the description
- Extract issue numbers from branch names or commit messages when possible
- When reading the PR template, preserve the exact markdown structure and placeholders
