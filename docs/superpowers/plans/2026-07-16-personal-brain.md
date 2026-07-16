# Personal Brain Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Initialize the iCloud-backed personal Brain and install a safe, idempotent `~/Brain` symbolic link through `make ai`.

**Architecture:** Keep the canonical content in iCloud Drive and expose it through a stable home-directory symlink. A phony Make target validates the source and destination without modifying Brain content, while local `AGENTS.md` files define and route the operating rules.

**Tech Stack:** GNU Make, POSIX shell, Markdown, macOS iCloud Drive

---

## File Map

- Modify `Makefile`: define the canonical Brain path, add the safe `brain`
  target, and include it in `ai`.
- Modify `AGENTS.md`: route Brain-related work to the Brain's canonical
  instructions.
- Create `~/Library/Mobile Documents/com~apple~CloudDocs/Brain/AGENTS.md`:
  define the Brain's organization and maintenance rules.
- Create the directories `Inbox/`, `Projects/`, `Areas/`, `Resources/`, and
  `Archive/` inside the canonical Brain directory.

Creating the Brain structure and `~/Brain` link writes outside the dotfiles
repository and requires explicit filesystem approval during execution.

### Task 1: Add the Safe Make Target

**Files:**

- Modify: `Makefile:1-16`
- Modify: `Makefile:169-186`
- Modify: `Makefile:188`

- [ ] **Step 1: Run the target before implementation to verify the missing rule**

Run:

```bash
make brain
```

Expected: FAIL with `No rule to make target 'brain'`.

- [ ] **Step 2: Add the Brain path, phony declaration, dependency, and target**

Add after `LOCAL_BIN`:

```make
BRAIN_PATH?=$(HOME)/Library/Mobile Documents/com~apple~CloudDocs/Brain
```

Add `brain` to the existing `.PHONY` declaration.

Add `brain` to the alphabetically ordered `ai` dependencies:

```make
ai: \
	brain \
	chatgpt \
	claude \
	claude-code \
	codegraph \
	codex \
	codexbar \
	cursor \
	googleworkspace-cli \
	hermes \
	llama-cpp \
	llmfit \
	mistral-vibe \
	opencode \
	openspec \
	pi-coding-agent \
	skills \
	skill-caveman
```

Add immediately after the `ai` dependency list:

```make
brain:
	@if [ ! -d "$(BRAIN_PATH)" ]; then \
		echo "Error: Brain directory does not exist: $(BRAIN_PATH)" >&2; \
		exit 1; \
	fi; \
	if [ -L "$(HOME)/Brain" ]; then \
		if [ "$$(readlink "$(HOME)/Brain")" = "$(BRAIN_PATH)" ]; then \
			echo "Brain symlink already configured"; \
		else \
			echo "Error: $(HOME)/Brain is not the expected symbolic link" >&2; \
			exit 1; \
		fi; \
	elif [ -e "$(HOME)/Brain" ]; then \
		echo "Error: $(HOME)/Brain already exists and is not a symbolic link" >&2; \
		exit 1; \
	else \
		ln -s "$(BRAIN_PATH)" "$(HOME)/Brain"; \
		echo "Created $(HOME)/Brain symlink"; \
	fi
```

- [ ] **Step 3: Verify link creation and idempotence with temporary paths**

Run:

```bash
tmp="$(mktemp -d)" &&
mkdir -p "$tmp/home" "$tmp/icloud/Brain" &&
make brain HOME="$tmp/home" BRAIN_PATH="$tmp/icloud/Brain" &&
test "$(readlink "$tmp/home/Brain")" = "$tmp/icloud/Brain" &&
make brain HOME="$tmp/home" BRAIN_PATH="$tmp/icloud/Brain"
```

Expected: PASS, first run prints `Created .../home/Brain symlink`, and second
run prints `Brain symlink already configured`.

- [ ] **Step 4: Verify a missing source fails without creating the link**

Run:

```bash
tmp="$(mktemp -d)" &&
mkdir -p "$tmp/home" &&
! make brain HOME="$tmp/home" BRAIN_PATH="$tmp/missing" &&
test ! -e "$tmp/home/Brain" &&
test ! -L "$tmp/home/Brain"
```

Expected: PASS and Make prints `Error: Brain directory does not exist`.

- [ ] **Step 5: Verify an existing directory is preserved**

Run:

```bash
tmp="$(mktemp -d)" &&
mkdir -p "$tmp/home/Brain" "$tmp/icloud/Brain" &&
! make brain HOME="$tmp/home" BRAIN_PATH="$tmp/icloud/Brain" &&
test -d "$tmp/home/Brain" &&
test ! -L "$tmp/home/Brain"
```

Expected: PASS and Make prints `already exists and is not a symbolic link`.

- [ ] **Step 6: Verify an incorrect symbolic link is preserved**

Run:

```bash
tmp="$(mktemp -d)" &&
mkdir -p "$tmp/home" "$tmp/icloud/Brain" "$tmp/other" &&
ln -s "$tmp/other" "$tmp/home/Brain" &&
! make brain HOME="$tmp/home" BRAIN_PATH="$tmp/icloud/Brain" &&
test "$(readlink "$tmp/home/Brain")" = "$tmp/other"
```

Expected: PASS and Make prints `is not the expected symbolic link`.

- [ ] **Step 7: Check Makefile formatting and commit**

Run:

```bash
git diff --check &&
git diff -- Makefile
```

Expected: no whitespace errors and only the planned Brain changes.

Commit:

```bash
git add Makefile
git commit -m "feat(ai): add Brain symlink target"
```

### Task 2: Initialize the Canonical Brain

**Files:**

- Create: `~/Library/Mobile Documents/com~apple~CloudDocs/Brain/AGENTS.md`
- Create: `~/Library/Mobile Documents/com~apple~CloudDocs/Brain/Inbox/`
- Create: `~/Library/Mobile Documents/com~apple~CloudDocs/Brain/Projects/`
- Create: `~/Library/Mobile Documents/com~apple~CloudDocs/Brain/Areas/`
- Create: `~/Library/Mobile Documents/com~apple~CloudDocs/Brain/Resources/`
- Create: `~/Library/Mobile Documents/com~apple~CloudDocs/Brain/Archive/`

- [ ] **Step 1: Verify the required structure is initially incomplete**

Run:

```bash
brain="$HOME/Library/Mobile Documents/com~apple~CloudDocs/Brain"
test -f "$brain/AGENTS.md" &&
for directory in Inbox Projects Areas Resources Archive; do
	test -d "$brain/$directory"
done
```

Expected: FAIL because `AGENTS.md` and the PARA directories do not yet exist.

- [ ] **Step 2: Create only the five PARA directories**

Run with approved access outside the repository:

```bash
brain="$HOME/Library/Mobile Documents/com~apple~CloudDocs/Brain"
mkdir -p \
	"$brain/Inbox" \
	"$brain/Projects" \
	"$brain/Areas" \
	"$brain/Resources" \
	"$brain/Archive"
```

- [ ] **Step 3: Create the Brain instructions**

Create `~/Library/Mobile Documents/com~apple~CloudDocs/Brain/AGENTS.md` with:

```markdown
# Personal Brain Instructions

The Brain is the source of truth for everything learned, decided, and built.

## Principles

- Use Markdown for all notes.
- Always prefer simplicity and human readability.
- Never create complex structures.
- Create a directory only when necessary.
- Reuse existing directories.
- Use explicit filenames.
- Use relative paths for links between files.
- Write every note so it remains understandable months later without the original conversation.
- Prefer incremental updates over complete rewrites.

## PARA Organization

- `Inbox/` contains unprocessed information.
- `Projects/` contains work with a defined end.
- `Areas/` contains ongoing responsibilities.
- `Resources/` contains reusable knowledge.
- `Archive/` contains completed or inactive content.

## Projects

Every project has a `README.md` containing:

- Objective
- Context
- Current state
- Important decisions
- Useful links
- Next steps
```

- [ ] **Step 4: Verify the exact top-level structure**

Run:

```bash
brain="$HOME/Library/Mobile Documents/com~apple~CloudDocs/Brain"
test -f "$brain/AGENTS.md" &&
for directory in Inbox Projects Areas Resources Archive; do
	test -d "$brain/$directory"
done &&
test "$(find "$brain" -mindepth 1 -maxdepth 1 | wc -l | tr -d ' ')" = "6"
```

Expected: PASS. The Brain contains exactly `AGENTS.md` and the five PARA
directories at its top level.

### Task 3: Route Dotfiles Agents to the Brain Rules

**Files:**

- Modify: `AGENTS.md:5-10`

- [ ] **Step 1: Verify the routing instruction is absent**

Run:

```bash
! rg -q 'Before operating on content under `~/Brain`' AGENTS.md
```

Expected: PASS because the instruction has not been added.

- [ ] **Step 2: Add the minimal routing section**

Add after `## Agent behavior (scope)` and its existing bullet list:

```markdown
## Personal Brain

- Before operating on content under `~/Brain`, read and follow `~/Brain/AGENTS.md`.
```

- [ ] **Step 3: Verify the routing instruction**

Run:

```bash
rg -n 'Before operating on content under `~/Brain`, read and follow `~/Brain/AGENTS.md`' AGENTS.md
```

Expected: one matching line under `## Personal Brain`.

- [ ] **Step 4: Check formatting and commit**

Run:

```bash
git diff --check &&
git diff -- AGENTS.md
```

Expected: no whitespace errors and only the new routing section.

Commit:

```bash
git add AGENTS.md
git commit -m "docs(agents): reference Brain instructions"
```

### Task 4: Install and Verify the Real Brain Link

**Files:**

- Create: `~/Brain` symbolic link

- [ ] **Step 1: Run the real target**

Run with approved access outside the repository:

```bash
make brain
```

Expected: `Created /Users/sebastien/Brain symlink`.

- [ ] **Step 2: Verify the real destination**

Run:

```bash
test -L "$HOME/Brain" &&
test "$(readlink "$HOME/Brain")" = "$HOME/Library/Mobile Documents/com~apple~CloudDocs/Brain"
```

Expected: PASS.

- [ ] **Step 3: Verify real idempotence**

Run:

```bash
make brain
```

Expected: `Brain symlink already configured`.

- [ ] **Step 4: Run final repository checks**

Run:

```bash
git status --short &&
git log -5 --oneline &&
git diff --check
```

Expected: clean working tree, the plan and all three implementation commits
are visible, and there are no whitespace errors.
