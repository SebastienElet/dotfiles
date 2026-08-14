# Plan d’implémentation du refactor du setup du hook Claude

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extraire le setup du hook Claude et son test CI dans des scripts Bash lisibles, sans target Make de test ni mélange avec le hook runtime.

**Architecture:** `scripts/setup/claude_handoff_hook` possède toute la mutation atomique de `~/.claude/settings.json`; `scripts/agent_handoff` reste exclusivement runtime. `scripts/setup/claude_handoff_hook_test` exerce le setup sous des `HOME` temporaires et la CI l’appelle directement, tandis que le Makefile ne conserve qu’une façade phony.

**Tech Stack:** Bash 3.2 portable, GNU Make, jq, GitHub Actions YAML.

---

### Task 1: Écrire le test Bash du setup

**Files:**
- Create: `scripts/setup/claude_handoff_hook_test`

- [ ] **Step 1: Créer le test exécutable**

```bash
#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
setup="$here/claude_handoff_hook"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

runtime_dir="$tmp/runtime hooks"
runtime_hook="$runtime_dir/agent_handoff"
mkdir -p "$runtime_dir"
touch "$runtime_hook"
chmod +x "$runtime_hook"

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

use_home() {
  HOME="$tmp/$1"
  export HOME
  settings="$HOME/.claude/settings.json"
  mkdir -p "$HOME"
}

run_setup() {
  "$setup" "$runtime_hook"
}

settings_mode() {
  case "$(uname -s)" in
  Darwin) stat -f '%Lp' "$1" ;;
  Linux) stat -c '%a' "$1" ;;
  *) fail "unsupported platform: $(uname -s)" ;;
  esac
}

test_creates_private_settings() {
  use_home creates
  run_setup
  jq -e --arg command "$runtime_hook" \
    '.hooks.Stop == [{hooks: [{type: "command", command: $command}]}]' \
    "$settings" >/dev/null
  [ "$(settings_mode "$settings")" = 600 ] || fail "settings mode is not 0600"
}

test_preserves_and_deduplicates_settings() {
  use_home preserves
  mkdir -p "$(dirname "$settings")"
  jq -n '{
    permissions: {allow: ["Read"]},
    hooks: {Stop: [{hooks: [{type: "command", command: "/other"}]}]}
  }' >"$settings"
  run_setup
  run_setup
  jq -e --arg command "$runtime_hook" '
    .permissions.allow == ["Read"]
    and ([.hooks.Stop[]?.hooks[]?.command] | map(select(. == $command)) | length == 1)
    and ([.hooks.Stop[]?.hooks[]?.command] | index("/other") != null)
  ' "$settings" >/dev/null
}

test_refuses_invalid_settings() {
  use_home invalid
  mkdir -p "$(dirname "$settings")"
  printf '{\n' >"$settings"
  cp "$settings" "$tmp/invalid-before"
  if run_setup; then
    fail "truncated JSON was accepted"
  fi
  cmp "$tmp/invalid-before" "$settings"

  use_home empty
  mkdir -p "$(dirname "$settings")"
  : >"$settings"
  if run_setup; then
    fail "empty JSON was accepted"
  fi
  [ ! -s "$settings" ] || fail "empty settings were replaced"
}

test_refuses_missing_requirements() {
  use_home requirements
  if "$setup"; then
    fail "missing hook path was accepted"
  fi
  mkdir -p "$tmp/empty-bin"
  if PATH="$tmp/empty-bin" /bin/bash "$setup" "$runtime_hook"; then
    fail "missing jq was accepted"
  fi
}

test_creates_private_settings
test_preserves_and_deduplicates_settings
test_refuses_invalid_settings
test_refuses_missing_requirements

printf 'ok\n'
```

Run: `chmod +x scripts/setup/claude_handoff_hook_test`

- [ ] **Step 2: Vérifier l’échec RED**

Run: `scripts/setup/claude_handoff_hook_test`

Expected: FAIL parce que `scripts/setup/claude_handoff_hook` n’existe pas encore.

### Task 2: Extraire le setup et simplifier ses appelants

**Files:**
- Create: `scripts/setup/claude_handoff_hook`
- Modify: `Makefile`
- Modify: `.github/workflows/test.yml`

- [ ] **Step 1: Créer le script de setup exécutable**

```bash
#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  printf 'usage: %s /absolute/path/to/agent_handoff\n' "$0" >&2
  exit 64
fi

runtime_hook=$1
case "$runtime_hook" in
/*) ;;
*)
  printf 'hook path must be absolute: %s\n' "$runtime_hook" >&2
  exit 64
  ;;
esac
[ -x "$runtime_hook" ] || { printf 'hook is not executable: %s\n' "$runtime_hook" >&2; exit 66; }
command -v jq >/dev/null || { printf 'jq is required\n' >&2; exit 69; }

settings_dir="$HOME/.claude"
settings="$settings_dir/settings.json"
mkdir -p "$settings_dir"
umask 077
temporary=$(mktemp "$settings.XXXXXX")
trap 'rm -f "$temporary"' EXIT HUP INT TERM

merge_settings() {
  jq -e --arg command "$runtime_hook" '
    if type == "object" then . else error("settings must contain a JSON object") end
    | .hooks.Stop //= []
    | if any(.hooks.Stop[]?.hooks[]?; .command == $command) then
        .
      else
        .hooks.Stop += [{hooks: [{type: "command", command: $command}]}]
      end
  ' "$@"
}

if [ -e "$settings" ]; then
  merge_settings "$settings" >"$temporary"
else
  printf '{}\n' | merge_settings >"$temporary"
fi

mv "$temporary" "$settings"
trap - EXIT HUP INT TERM
```

Run: `chmod +x scripts/setup/claude_handoff_hook`

- [ ] **Step 2: Réduire les targets Make à leur rôle de façade**

Ajouter `.PHONY: claude-code` immédiatement avant `claude-code`. Conserver `.PHONY: claude-handoff-hook` immédiatement avant sa target et remplacer sa recette par :

```make
claude-handoff-hook: ${BREW_BIN}/jq ${DOTFILES_PATH}/scripts/setup/claude_handoff_hook
	@"${DOTFILES_PATH}/scripts/setup/claude_handoff_hook" "${DOTFILES_PATH}/scripts/agent_handoff"
```

- [ ] **Step 3: Réduire le workflow à l’appel direct du test**

```yaml
    - name: Check Claude hook registration
      run: scripts/setup/claude_handoff_hook_test
```

- [ ] **Step 4: Vérifier GREEN**

Run: `scripts/setup/claude_handoff_hook_test`

Expected: PASS avec `ok`; les erreurs attendues des scénarios négatifs restent visibles sur stderr.

- [ ] **Step 5: Commit fonctionnel**

```bash
git add .github/workflows/test.yml Makefile scripts/setup/claude_handoff_hook scripts/setup/claude_handoff_hook_test
git commit -m "refactor(claude): extract handoff hook setup"
```

### Task 3: Vérifier les barrières réellement concernées

**Files:**
- Test: `scripts/setup/claude_handoff_hook`
- Test: `scripts/setup/claude_handoff_hook_test`
- Test: `scripts/agent_handoff`
- Test: `scripts/agent_handoff_test`
- Test: `.github/workflows/test.yml`
- Test: `Makefile`

- [ ] **Step 1: Vérifier la syntaxe et ShellCheck**

Run: `bash -n scripts/agent_handoff scripts/agent_handoff_test scripts/setup/claude_handoff_hook scripts/setup/claude_handoff_hook_test`

Expected: exit 0.

Run: `shellcheck --severity=error scripts/agent_handoff scripts/agent_handoff_test scripts/setup/claude_handoff_hook scripts/setup/claude_handoff_hook_test`

Expected: exit 0 sur macOS; la CI lint découvrira récursivement les deux nouveaux scripts par leur shebang.

- [ ] **Step 2: Exécuter les tests Bash**

Run: `scripts/agent_handoff_test && scripts/setup/claude_handoff_hook_test`

Expected: deux lignes `ok` et exit 0 sur macOS.

- [ ] **Step 3: Vérifier le câblage sans exécuter d’installation**

Run: `make -n claude-code BREW_BIN=/usr/bin`

Expected: le dry-run contient l’appel à `scripts/setup/claude_handoff_hook` et n’exécute rien.

- [ ] **Step 4: Vérifier le YAML et le diff**

Run: `ruby -e 'require "yaml"; YAML.load_file(".github/workflows/test.yml")'`

Expected: exit 0 sur macOS avec Ruby/Psych.

Run: `git diff --check && git status --short --branch`

Expected: aucune erreur d’espace; seuls les commits prévus sont en avance sur la branche distante.
