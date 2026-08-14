# Plan d'implémentation de l'intégration CodeGraph aux agents

> **Pour les agents d'exécution :** SOUS-SKILL REQUIS : utiliser
> `superpowers:subagent-driven-development` (recommandé) ou `superpowers:executing-plans` pour
> exécuter ce plan tâche par tâche. Les étapes utilisent des cases (`- [ ]`) pour le suivi.

**Objectif :** installer CodeGraph 1.5.0, l'exposer globalement à Codex, Claude Code et Cursor, et
faire initialiser automatiquement les grands dépôts par les agents pour les explorations
structurelles.

**Architecture :** le `Makefile` épingle le binaire et déploie les configurations globales ; une
skill partagée porte la politique, appuyée par un helper qui mesure uniquement la taille du dépôt.
CodeGraph reste le serveur MCP upstream sans wrapper, avec son index natif `.codegraph/`, son
watcher et son daemon. Une fixture publique et un client JSON-RPC exercent la fraîcheur réelle.

**Stack technique :** GNU Make, Bash 3.2 portable, `tokei`, `jq`, Node.js ESM, MCP stdio JSON-RPC,
CodeGraph 1.5.0, skills Markdown/YAML/JSON, GitHub Actions macOS.

---

## Carte des fichiers

### Commit 1 — politique d'activation

- Créer : `.agents/skills/codegraph/SKILL.md`
- Créer : `.agents/skills/codegraph/evals/trigger-queries.json`
- Créer : `.agents/skills/codegraph/scripts/measure_repository.sh`
- Créer : `.agents/skills/codegraph/scripts/measure_repository_test.sh`
- Modifier : `.agents/skills/README.md` par `skill-manager sync-index` uniquement

### Commit 2 — installation et configuration globale

- Créer : `.config/git/ignore`
- Créer : `scripts/codegraph_configure`
- Créer : `scripts/codegraph_configure_test`
- Modifier : `Makefile`

### Commit 3 — vérification MCP réelle

- Créer : `codegraph/fixtures/freshness/package.json`
- Créer : `codegraph/fixtures/freshness/tsconfig.json`
- Créer : `codegraph/fixtures/freshness/src/branch.ts`
- Créer : `codegraph/fixtures/freshness/src/entry.ts`
- Créer : `codegraph/fixtures/freshness/src/live.ts`
- Créer : `codegraph/fixtures/freshness/src/removable.ts`
- Créer : `codegraph/mcp_probe.mjs`
- Créer : `codegraph/network_canary.mjs`
- Créer : `scripts/codegraph_mcp_test`
- Créer : `scripts/codegraph_network_test`
- Créer : `.github/workflows/test-codegraph.yml`
- Modifier : `Makefile`

### Commit 4 — décision et exploitation

- Créer : `docs/adr/038-codegraph-recuperation-structurelle.md`
- Créer : `docs/codegraph.md`
- Modifier : `docs/adr/README.md`

### Commit 5 — preuves observées

- Créer : `docs/codegraph-validation.md`

## Tâche 0 : resynchroniser le worktree et confirmer le pin

- [ ] **Étape 1 : charger les skills d'implémentation requis**

Lire intégralement, dans cet ordre : `superpowers:test-driven-development`, `dotfiles`, `scripts`,
`skill-manager` avec `references/conventions.md`, puis `superpowers:writing-skills`. Ne modifier
aucun fichier avant cette lecture.

- [ ] **Étape 2 : vérifier l'état isolé**

```bash
git status --short --branch
git worktree list --porcelain
git log -2 --oneline
```

Attendu : branche `issue-97-codegraph`, worktree
`/Users/sebastien/.dotfiles/.worktrees/issue-97-codegraph`, aucun changement local, spec et plan en
tête de branche.

- [ ] **Étape 3 : rebaser sur le dernier `main` distant**

```bash
git fetch origin main
git rebase origin/main
git status --short --branch
```

Attendu : rebase réussi, worktree propre. En cas de conflit avec un changement utilisateur,
s'arrêter et le signaler ; ne jamais résoudre en écrasant le changement.

- [ ] **Étape 4 : confirmer la version upstream retenue**

```bash
gh release view v1.5.0 --repo colbymchenry/codegraph --json tagName,isPrerelease,publishedAt
codegraph --version
```

Attendu : `v1.5.0`, release non préliminaire ; le binaire local affiche `1.5.0`. Une version plus
récente ne change pas le pin sans nouvelle décision et vérification.

## Tâche 1 : mesurer les dépôts et livrer la skill partagée

**Fichiers :** tous les fichiers du commit 1.

- [ ] **Étape 1 : créer d'abord le test RED du helper**

Créer `.agents/skills/codegraph/scripts/measure_repository_test.sh` avec ce contenu :

```bash
#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
measure="$here/measure_repository.sh"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

fake_tokei="$tmp/tokei"
args_log="$tmp/args"

cat >"$fake_tokei" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' "$*" >"$CODEGRAPH_TOKEI_ARGS"
jq -n \
  --argjson loc "$CODEGRAPH_TEST_LOC" \
  --argjson files "$CODEGRAPH_TEST_FILES" \
  'range(0; $files) as $index
  | {language: "TypeScript", stats: {name: ("fixture-" + ($index | tostring) + ".ts"), stats: {code: (if $index == 0 then $loc else 0 end)}}}'
SCRIPT
chmod +x "$fake_tokei"

measure_case() {
  CODEGRAPH_TOKEI_BIN="$fake_tokei" \
    CODEGRAPH_TOKEI_ARGS="$args_log" \
    CODEGRAPH_TEST_LOC="$1" \
    CODEGRAPH_TEST_FILES="$2" \
    "$measure" "$tmp/repository"
}

mkdir -p "$tmp/repository"

low=$(measure_case 49999 499)
[ "$(printf '%s' "$low" | jq -r .initialize)" = "false" ]
[ "$(printf '%s' "$low" | jq -r .loc)" = "49999" ]
[ "$(printf '%s' "$low" | jq -r .files)" = "499" ]

loc_threshold=$(measure_case 50000 1)
[ "$(printf '%s' "$loc_threshold" | jq -r .initialize)" = "true" ]

file_threshold=$(measure_case 1 500)
[ "$(printf '%s' "$file_threshold" | jq -r .initialize)" = "true" ]

rg -F -- '--hidden' "$args_log"
rg -F -- '--streaming json' "$args_log"
rg -F -- '--exclude node_modules' "$args_log"
rg -F -- '--exclude docs' "$args_log"
rg -F -- '--exclude fixtures' "$args_log"
rg -F 'TypeScript' "$args_log"
rg -F 'Rust' "$args_log"
rg -F 'Razor' "$args_log"
rg -F 'Vue' "$args_log"

real_repository="$tmp/real-repository"
mkdir -p "$real_repository/src" "$real_repository/docs" "$real_repository/ignored"
printf '%s\n' 'export const kept = 1' >"$real_repository/src/kept.ts"
printf '%s\n' 'export const documentation = 2' >"$real_repository/docs/documentation.ts"
printf '%s\n' 'export const ignored = 3' >"$real_repository/ignored/ignored.ts"
printf '%s\n' 'ignored/' >"$real_repository/.gitignore"
git -C "$real_repository" init -q
real=$(CODEGRAPH_TOKEI_BIN=tokei "$measure" "$real_repository")
[ "$(printf '%s' "$real" | jq -r .loc)" = "1" ]
[ "$(printf '%s' "$real" | jq -r .files)" = "1" ]

if CODEGRAPH_TOKEI_BIN="$tmp/missing" "$measure" "$tmp/repository" 2>"$tmp/error"; then
  exit 1
fi
rg -F 'tokei is required' "$tmp/error"

if CODEGRAPH_TOKEI_BIN="$fake_tokei" CODEGRAPH_JQ_BIN="$tmp/missing" \
  "$measure" "$tmp/repository" 2>"$tmp/error"; then
  exit 1
fi
rg -F 'jq is required' "$tmp/error"

echo ok
```

Le test crée son faux `tokei` dans un répertoire temporaire et ajoute six limites réelles de 500
fichiers : Razor, OpenTofu et CommonJS activent ; `.hcl` générique, YAML et XML n'activent pas.
Aucun fixture volumineux n'est versionné.

- [ ] **Étape 2 : exécuter le test et observer RED**

```bash
bash .agents/skills/codegraph/scripts/measure_repository_test.sh
```

Attendu : échec `No such file or directory` sur `measure_repository.sh`.

- [ ] **Étape 3 : implémenter le helper minimal**

Créer `.agents/skills/codegraph/scripts/measure_repository.sh` avec les propriétés suivantes :

- produire les enregistrements fichier par fichier avec `tokei --streaming json` ;
- filtrer les extensions sur la carte canonique de CodeGraph 1.5.0 plutôt que sur les catégories
  plus larges de Tokei ;
- compter Razor, OpenTofu et CommonJS, sans compter `.hcl` générique ni les formats de données YAML
  et XML ;
- compléter `.tofu` par des symlinks temporaires `.tf`, Tokei ne connaissant pas cette extension ;
- respecter Git et les exclusions de dépendances, sorties générées, documentation et fixtures ;
- retourner `{loc, files, initialize}` avec `initialize` vrai à 50 000 lignes ou 500 fichiers.

Le helper ne lance jamais `codegraph init` et ne décide pas du type de tâche ; il rend uniquement
la mesure reproductible.

- [ ] **Étape 4 : vérifier GREEN et les erreurs**

```bash
bash .agents/skills/codegraph/scripts/measure_repository_test.sh
bash -n .agents/skills/codegraph/scripts/measure_repository.sh .agents/skills/codegraph/scripts/measure_repository_test.sh
shellcheck --severity=error .agents/skills/codegraph/scripts/measure_repository.sh .agents/skills/codegraph/scripts/measure_repository_test.sh
.agents/skills/codegraph/scripts/measure_repository.sh . | jq -e '.initialize == false and .loc < 50000 and .files < 500'
```

Attendu : `ok`, syntaxe et ShellCheck sans sortie, puis verdict `false` pour ce dépôt public sur
macOS. Ne pas publier les mesures des dépôts privés.

- [ ] **Étape 5 : créer la skill avec la politique exacte**

Créer `.agents/skills/codegraph/SKILL.md` :

````markdown
---
name: codegraph
description: >
  Use CodeGraph for open-ended structural exploration of large repositories. Use when locating
  architecture, call paths, dependencies, cross-package behavior, or change impact, and when an
  existing .codegraph index must be checked or synchronized. Make sure to use it before broad
  file-by-file exploration; initialize automatically only when the repository reaches 50000 source
  lines or 500 source files.
metadata:
  category: dev
---

# CodeGraph

## Routing

Use `rg` and `fd` directly for exact literals, regular expressions, known paths, and targeted
source verification. Those tasks never measure the repository and never initialize CodeGraph.

For open-ended structural exploration, architecture, call paths, dependencies, cross-package
behavior, or change impact, follow the lifecycle below.

## Lifecycle

1. If `.codegraph/` exists, run the private CLI environment from the Commands section with
   `codegraph status --json`.
2. If status is healthy, use `codegraph_explore` before broad grep, find, or file reads.
3. If status reports stale or incomplete state, run `codegraph sync` once and check status again.
4. If `.codegraph/` is absent, run `codegraph-repository-size .`.
5. If `initialize` is true, run `codegraph init`, confirm status, then use `codegraph_explore`.
6. If `initialize` is false, use `rg` and `fd` without initializing.
7. Verify important graph claims in the source before editing.

An existing index remains usable below the threshold. The threshold uses OR: 50000 source lines or
500 source files.

## Commands

Prefix every CodeGraph CLI call with:

```bash
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1
```

The MCP entry already supplies the same environment.

## Failures

If measurement, initialization, synchronization, or the second status check fails, name the
failure and fall back explicitly to `rg` and `fd`. Never describe a failed or unchecked index as
fresh.

Never run `codegraph uninit --force`, `codegraph index`, or remove `.codegraph/` automatically.
Corruption, an unexplained lock, and an incompatible index require a diagnosis and user approval
before destructive recovery.

## Boundaries

CodeGraph is retrieval-only. Use language servers for semantic rename and code actions, and a
debugger for runtime inspection. The default MCP surface is `codegraph_explore`; do not enable
hidden tools or add a query wrapper.
````

- [ ] **Étape 6 : ajouter les évaluations de déclenchement**

Créer `.agents/skills/codegraph/evals/trigger-queries.json` :

```json
{
  "skill": "codegraph",
  "version": "1.0",
  "queries": [
    {
      "query": "Explique-moi l'architecture de ce monorepo et les dépendances entre packages",
      "should_activate": true,
      "reason": "Open-ended structural exploration across packages"
    },
    {
      "query": "Quel est le blast radius si je change ce service ?",
      "should_activate": true,
      "reason": "Cross-file impact analysis"
    },
    {
      "query": "L'index CodeGraph semble périmé après mon changement de branche",
      "should_activate": true,
      "reason": "Existing index freshness and synchronization"
    },
    {
      "query": "Trouve exactement la chaîne FEATURE_FLAG_DISABLED avec rg",
      "should_activate": false,
      "reason": "Exact literal lookup must stay on rg"
    },
    {
      "query": "Ouvre src/auth/session.ts à la ligne 42",
      "should_activate": false,
      "reason": "Known path and targeted read"
    },
    {
      "query": "Renomme ce symbole avec le language server",
      "should_activate": false,
      "reason": "Semantic refactoring is outside retrieval"
    }
  ]
}
```

- [ ] **Étape 7 : valider et synchroniser la skill**

Exécuter le workflow `skill-manager` sur `codegraph`, puis :

```bash
jq empty .agents/skills/codegraph/evals/trigger-queries.json
prettier --check .agents/skills/codegraph/SKILL.md .agents/skills/codegraph/evals/trigger-queries.json
git diff -- .agents/skills/README.md
git diff --check
```

Attendu : doctor vert, index README mis à jour une seule fois et resynchronisation suivante sans
diff. Si `skills-ref` ou un doctor optionnel est absent, le signaler sans revendiquer sa couverture.

- [ ] **Étape 8 : committer la politique**

```bash
git add .agents/skills/codegraph .agents/skills/README.md
git diff --cached --check
git commit -m "feat(codegraph): add repository activation policy"
```

## Tâche 2 : épingler CodeGraph et configurer les trois agents

**Fichiers :** tous les fichiers du commit 2, plus les cibles de liens de skill dans `Makefile`.

- [ ] **Étape 1 : écrire le test RED du configurateur**

Créer `scripts/codegraph_configure_test` :

```bash
#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
configure="$here/codegraph_configure"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

state="$tmp/state"
log="$tmp/calls"
mkdir -p "$state" "$tmp/bin" "$tmp/cursor"

make_fake() {
  name=$1
  path="$tmp/bin/$name"
  printf '%s\n' '#!/usr/bin/env bash' >"$path"
  printf '%s\n' 'set -euo pipefail' >>"$path"
  printf '%s\n' 'name=$(basename "$0")' >>"$path"
  printf '%s\n' 'printf "%s %s\n" "$name" "$*" >>"$CODEGRAPH_TEST_LOG"' >>"$path"
  printf '%s\n' 'marker="$CODEGRAPH_TEST_STATE/$name"' >>"$path"
  printf '%s\n' 'if [ "$name" = codegraph ]; then [ "$*" = "telemetry off" ]; exit; fi' >>"$path"
  printf '%s\n' 'if [ "${1:-} ${2:-}" = "mcp get" ]; then [ -f "$marker" ]; exit; fi' >>"$path"
  printf '%s\n' 'if [ "${1:-} ${2:-}" = "mcp remove" ]; then rm -f "$marker"; exit; fi' >>"$path"
  printf '%s\n' 'if [ "${1:-} ${2:-}" = "mcp add" ]; then touch "$marker"; exit; fi' >>"$path"
  printf '%s\n' 'exit 1' >>"$path"
  chmod +x "$path"
}

make_fake claude
make_fake codex
make_fake codegraph

cat >"$tmp/cursor/mcp.json" <<'JSON'
{
  "mcpServers": {
    "existing": {
      "command": "existing-server"
    }
  }
}
JSON

run_configure() {
  CODEGRAPH_CLAUDE_BIN="$tmp/bin/claude" \
    CODEGRAPH_CODEX_BIN="$tmp/bin/codex" \
    CODEGRAPH_BIN="$tmp/bin/codegraph" \
    CODEGRAPH_CURSOR_CONFIG="$tmp/cursor/mcp.json" \
    CODEGRAPH_TEST_LOG="$log" \
    CODEGRAPH_TEST_STATE="$state" \
    "$configure"
}

run_configure
run_configure

jq -e '.mcpServers.existing.command == "existing-server"' "$tmp/cursor/mcp.json"
jq -e '.mcpServers.codegraph.command == "codegraph"' "$tmp/cursor/mcp.json"
jq -e '.mcpServers.codegraph.args == ["serve", "--mcp", "--path", "${workspaceFolder}"]' "$tmp/cursor/mcp.json"
jq -e '.mcpServers.codegraph.env == {
  "CODEGRAPH_TELEMETRY": "0",
  "CODEGRAPH_NO_UPDATE_CHECK": "1",
  "CODEGRAPH_NO_DOWNLOAD": "1"
}' "$tmp/cursor/mcp.json"
[ "$(rg -c '^codegraph telemetry off$' "$log")" = "2" ]
[ "$(rg -c '^claude mcp add ' "$log")" = "2" ]
[ "$(rg -c '^codex mcp add ' "$log")" = "2" ]
[ "$(rg -c '^claude mcp remove ' "$log")" = "1" ]
[ "$(rg -c '^codex mcp remove ' "$log")" = "1" ]
rg -F -- '--scope user' "$log"
rg -F -- '--env CODEGRAPH_TELEMETRY=0' "$log"
rg -F -- '-e CODEGRAPH_TELEMETRY=0' "$log"

cp "$tmp/cursor/mcp.json" "$tmp/cursor/valid.json"
printf '{invalid\n' >"$tmp/cursor/mcp.json"
before=$(wc -l <"$log")
if run_configure 2>"$tmp/error"; then exit 1; fi
[ "$(wc -l <"$log")" = "$before" ]
rg -F 'invalid Cursor MCP JSON' "$tmp/error"

mv "$tmp/cursor/valid.json" "$tmp/cursor/target.json"
ln -s "$tmp/cursor/target.json" "$tmp/cursor/mcp.json"
if run_configure 2>"$tmp/error"; then exit 1; fi
rg -F 'Cursor MCP config must not be a symlink' "$tmp/error"

echo ok
```

- [ ] **Étape 2 : observer RED**

```bash
bash scripts/codegraph_configure_test
```

Attendu : échec `No such file or directory` sur `scripts/codegraph_configure`.

- [ ] **Étape 3 : implémenter le configurateur**

Créer `scripts/codegraph_configure` avec les propriétés suivantes :

- valider les trois exécutables et exactement un objet JSON Cursor avant toute mutation ;
- enregistrer le chemin du binaire CodeGraph épinglé dans les trois agents ;
- utiliser la grammaire réelle `claude mcp add --scope user codegraph -e … -- BINAIRE serve --mcp` ;
- utiliser `codex mcp add codegraph --env … -- BINAIRE serve --mcp` ;
- préserver les autres serveurs Cursor par une écriture JSON atomique ;
- refuser les symlinks normaux et cassés pour les configurations natives des trois agents ;
- distinguer une absence de serveur d'une défaillance native par égalité avec les diagnostics
  connus, avant toute mutation MCP ;
- sauvegarder les configurations Claude et Codex, puis les restaurer octet pour octet si une étape
  ultérieure échoue.

Le script configure, il ne lance ni ne relaie le serveur MCP. L'absence d'un agent, un état natif
illisible ou un JSON Cursor invalide échoue sans laisser de configuration partielle.

- [ ] **Étape 4 : vérifier GREEN**

```bash
bash scripts/codegraph_configure_test
bash -n scripts/codegraph_configure scripts/codegraph_configure_test
shellcheck --severity=error scripts/codegraph_configure scripts/codegraph_configure_test
```

Attendu : `ok`, puis aucune sortie des deux vérificateurs.

- [ ] **Étape 5 : versionner l'exclusion Git globale**

Créer `.config/git/ignore` avec exactement :

```gitignore
**/.claude/settings.local.json
.codegraph/
```

Avant toute installation réelle, constater que `~/.config/git/ignore` est actuellement un fichier
ordinaire contenant seulement la première ligne. Ne pas le remplacer depuis le worktree. Le
`Makefile` doit refuser cet état ; la migration récupérable vers le symlink sera faite depuis le
checkout canonique après intégration.

- [ ] **Étape 6 : modifier le `Makefile`**

Ajouter près des variables globales :

```make
CODEGRAPH_VERSION:=1.5.0
CODEGRAPH_GLOBAL_IGNORE?=$(HOME)/.config/git/ignore
```

Remplacer la cible CodeGraph par :

```make
.PHONY: codegraph codegraph-cli codegraph-ignore
codegraph: codegraph-cli claude-code codex cursor codegraph-ignore ~/.local/bin/codegraph-repository-size ~/.claude/skills/codegraph ~/.agents/skills/codegraph ~/.cursor/skills/codegraph
	CODEGRAPH_CLAUDE_BIN=${LOCAL_BIN}/claude CODEGRAPH_CODEX_BIN=${VOLTA_BIN}/codex CODEGRAPH_BIN=${VOLTA_BIN}/codegraph scripts/codegraph_configure

codegraph-cli: ${VOLTA_BIN}/node
	@if [ ! -x "${VOLTA_BIN}/codegraph" ] || [ "$$(${VOLTA_BIN}/codegraph --version)" != "${CODEGRAPH_VERSION}" ]; then \
		${BREW_BIN}/volta install @colbymchenry/codegraph@${CODEGRAPH_VERSION}; \
	fi

codegraph-ignore:
	@expected='${DOTFILES_PATH}/.config/git/ignore'; \
	target='${CODEGRAPH_GLOBAL_IGNORE}'; \
	if [ -L "$$target" ] && [ "$$(readlink "$$target")" = "$$expected" ]; then \
		exit 0; \
	fi; \
	if [ -e "$$target" ] || [ -L "$$target" ]; then \
		echo "Error: $$target exists and is not the expected symbolic link" >&2; \
		exit 1; \
	fi; \
	mkdir -p "$$(dirname "$$target")"; \
	ln -s "$$expected" "$$target"

~/.local/bin/codegraph-repository-size: ${DOTFILES_PATH}/.agents/skills/codegraph/scripts/measure_repository.sh | ~/.local/bin
	ln -s ${DOTFILES_PATH}/.agents/skills/codegraph/scripts/measure_repository.sh $@
```

Ajouter `codegraph` aux dépendances de skills des trois agents, sur le même modèle que
`enforcement-code` :

```make
~/.cursor/skills/codegraph: ${DOTFILES_PATH}/.agents/skills/codegraph | ~/.cursor/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/codegraph $@
~/.claude/skills/codegraph: ${DOTFILES_PATH}/.agents/skills/codegraph | ~/.claude/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/codegraph $@
~/.agents/skills/codegraph: ${DOTFILES_PATH}/.agents/skills/codegraph | ~/.agents/skills
	ln -s ${DOTFILES_PATH}/.agents/skills/codegraph $@
```

Les listes de dépendances `cursor`, `claude-code` et `codex` doivent nommer leurs nouveaux liens.
Ajouter aucun commentaire Makefile.

- [ ] **Étape 7 : tester les cibles sans effet global**

```bash
make -Bn codegraph
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
make codegraph-ignore CODEGRAPH_GLOBAL_IGNORE="$tmp/git/ignore"
test "$(readlink "$tmp/git/ignore")" = "$(pwd)/.config/git/ignore"
make codegraph-ignore CODEGRAPH_GLOBAL_IGNORE="$tmp/git/ignore"
rm "$tmp/git/ignore"
printf 'foreign\n' >"$tmp/git/ignore"
! make codegraph-ignore CODEGRAPH_GLOBAL_IGNORE="$tmp/git/ignore"
git -c core.excludesFile=.config/git/ignore check-ignore -q .codegraph/index.db
```

Attendu : dry-run montrant le pin exact et les trois configurations ; symlink temporaire idempotent ;
destination étrangère refusée ; `.codegraph/index.db` ignoré. Aucune cible d'installation réelle
n'est exécutée depuis le worktree.

- [ ] **Étape 8 : committer la configuration**

```bash
git add Makefile .config/git/ignore scripts/codegraph_configure scripts/codegraph_configure_test
git diff --cached --check
git commit -m "feat(codegraph): configure coding agents"
```

## Tâche 3 : prouver la fraîcheur avec le serveur MCP réel

**Fichiers :** tous les fichiers du commit 3.

- [ ] **Étape 1 : créer la fixture publique**

Créer les fichiers suivants :

`codegraph/fixtures/freshness/package.json`

```json
{
  "name": "codegraph-freshness-fixture",
  "private": true,
  "type": "module"
}
```

`codegraph/fixtures/freshness/tsconfig.json`

```json
{
  "compilerOptions": {
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true
  },
  "include": ["src"]
}
```

`codegraph/fixtures/freshness/src/live.ts`

```typescript
export const liveValue = 1;
export const liveSentinel = "FIXTURE_LIVE_V1";
```

`codegraph/fixtures/freshness/src/removable.ts`

```typescript
export const removableValue = 2;
export const removableSentinel = "FIXTURE_REMOVABLE";
```

`codegraph/fixtures/freshness/src/branch.ts`

```typescript
export const branchMainValue = 3;
export const branchSentinel = "FIXTURE_BRANCH_MAIN";
```

`codegraph/fixtures/freshness/src/entry.ts`

```typescript
import { branchMainValue } from "./branch.js";
import { liveValue } from "./live.js";
import { removableValue } from "./removable.js";

export const entryValue = branchMainValue + liveValue + removableValue;
```

- [ ] **Étape 2 : créer l'orchestrateur avant le probe**

Créer `scripts/codegraph_mcp_test` avec ce contenu complet :

```bash
#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
fixture="$root/codegraph/fixtures/freshness"
probe="$root/codegraph/mcp_probe.mjs"
tmp=$(mktemp -d)
repository="$tmp/repository"
trap 'CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 codegraph uninit --force "$repository" >/dev/null 2>&1 || true; rm -rf "$tmp"' EXIT

command -v codegraph >/dev/null 2>&1 || { echo 'codegraph is required' >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { echo 'jq is required' >&2; exit 2; }
command -v node >/dev/null 2>&1 || { echo 'node is required' >&2; exit 2; }

cp -R "$fixture" "$repository"
git -C "$repository" init -b main >/dev/null
git -C "$repository" config user.email codegraph-fixture@example.invalid
git -C "$repository" config user.name 'CodeGraph Fixture'
git -C "$repository" add .
git -C "$repository" commit -m baseline >/dev/null
git -C "$repository" switch -c codegraph-alt >/dev/null
printf '%s\n' \
  'export const branchAltValue = 30' \
  'export const branchSentinel = "FIXTURE_BRANCH_ALT"' >"$repository/src/branch.ts"
git -C "$repository" add src/branch.ts
git -C "$repository" commit -m alternate >/dev/null
git -C "$repository" switch main >/dev/null

SECONDS=0
time_log="$tmp/index-time"
/usr/bin/time -l -o "$time_log" env \
  CODEGRAPH_TELEMETRY=0 \
  CODEGRAPH_NO_UPDATE_CHECK=1 \
  CODEGRAPH_NO_DOWNLOAD=1 \
  codegraph init "$repository" >/dev/null
initial_index_seconds=$SECONDS
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 \
  codegraph status --json "$repository" | jq empty

probe_result=$(node "$probe" "$repository")
printf '%s' "$probe_result" | jq -e '
  .tools == ["codegraph_explore"]
  and (.scenarios | keys) == [
    "branchSwitch", "daemonStopped", "delete", "edit", "initial",
    "reconciliation", "rename", "restart", "watcherInterruption"
  ]
  and .scenarios.branchSwitch == true
  and .scenarios.daemonStopped == true
  and .scenarios.delete == true
  and .scenarios.edit == true
  and .scenarios.initial == true
  and .scenarios.reconciliation == true
  and .scenarios.rename == true
  and .scenarios.restart == true
  and (.scenarios.watcherInterruption == "fresh" or .scenarios.watcherInterruption == "alerted-stale")
' >/dev/null

disk_kib=$(du -sk "$repository/.codegraph" | awk '{print $1}')
max_rss_bytes=$(awk '/maximum resident set size/ {print $1}' "$time_log")
cpu_user_seconds=$(awk 'NR == 1 {print $3}' "$time_log")
cpu_system_seconds=$(awk 'NR == 1 {print $5}' "$time_log")
jq -n \
  --argjson probe "$probe_result" \
  --argjson initialIndexSeconds "$initial_index_seconds" \
  --argjson diskKiB "$disk_kib" \
  --argjson maxRssBytes "${max_rss_bytes:-0}" \
  --argjson cpuUserSeconds "${cpu_user_seconds:-0}" \
  --argjson cpuSystemSeconds "${cpu_system_seconds:-0}" \
  '{
    environment: {os: "macOS", linuxExercised: false},
    initialIndexSeconds: $initialIndexSeconds,
    initialIndexMaxRssBytes: $maxRssBytes,
    initialIndexCpuUserSeconds: $cpuUserSeconds,
    initialIndexCpuSystemSeconds: $cpuSystemSeconds,
    indexDiskKiB: $diskKiB,
    mcp: $probe
  }'

CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 \
  codegraph uninit --force "$repository" >/dev/null
test ! -e "$repository/.codegraph"
```

Le `trap` ne vise que le répertoire créé par ce test. `/usr/bin/time -l` rend ce test macOS ; le
document de validation doit nommer Linux comme non exercé.

- [ ] **Étape 3 : observer RED**

Sans créer `codegraph/mcp_probe.mjs`, lancer :

```bash
bash scripts/codegraph_mcp_test
```

Attendu : échec explicite `Cannot find module .../codegraph/mcp_probe.mjs` après une indexation
initiale réussie. Le `trap` doit supprimer uniquement la copie temporaire et son index.

- [ ] **Étape 4 : implémenter le client JSON-RPC**

Créer `codegraph/mcp_probe.mjs` avec ces responsabilités, dans cet ordre :

```javascript
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const repository = fs.realpathSync(process.argv[2]);
const source = path.join(repository, "src");
const privacyEnvironment = {
  ...process.env,
  CODEGRAPH_TELEMETRY: "0",
  CODEGRAPH_NO_UPDATE_CHECK: "1",
  CODEGRAPH_NO_DOWNLOAD: "1",
  CODEGRAPH_DAEMON_IDLE_TIMEOUT_MS: "500",
};
const timings = {};
const auditPauseMilliseconds = Number.parseInt(
  process.env.CODEGRAPH_PROBE_PAUSE_MS || "0",
  10,
);
const serverPidFile = process.env.CODEGRAPH_PROBE_SERVER_PID_FILE;
let server;
let buffer = "";
let nextId = 0;
let pending = new Map();
let stderr = "";
let synchronizationMilliseconds = 0;

const delay = (milliseconds) =>
  new Promise((resolve) => setTimeout(resolve, milliseconds));

const run = (command, args, environment = privacyEnvironment) => {
  const result = spawnSync(command, args, {
    cwd: repository,
    env: environment,
    encoding: "utf8",
  });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed: ${result.stderr}`);
  }
  return result.stdout;
};

const request = (method, params = {}) =>
  new Promise((resolve, reject) => {
    const id = ++nextId;
    const timer = setTimeout(() => {
      pending.delete(id);
      reject(new Error(`MCP request timed out: ${method}`));
    }, 30000);
    pending.set(id, { resolve, reject, timer });
    server.stdin.write(
      `${JSON.stringify({ jsonrpc: "2.0", id, method, params })}\n`,
    );
  });

const notify = (method, params = {}) => {
  server.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", method, params })}\n`);
};

const consume = (chunk) => {
  buffer += chunk.toString();
  for (;;) {
    const newline = buffer.indexOf("\n");
    if (newline < 0) return;
    const line = buffer.slice(0, newline).trim();
    buffer = buffer.slice(newline + 1);
    if (!line.startsWith("{")) continue;
    const message = JSON.parse(line);
    if (message.id === undefined || !pending.has(message.id)) continue;
    const waiter = pending.get(message.id);
    pending.delete(message.id);
    clearTimeout(waiter.timer);
    if (message.error) waiter.reject(new Error(JSON.stringify(message.error)));
    else waiter.resolve(message.result);
  }
};

const startServer = async (extraArguments = []) => {
  buffer = "";
  stderr = "";
  pending = new Map();
  server = spawn(
    "codegraph",
    ["serve", "--mcp", "--path", repository, ...extraArguments],
    {
      cwd: repository,
      env: privacyEnvironment,
      stdio: ["pipe", "pipe", "pipe"],
    },
  );
  if (serverPidFile) fs.writeFileSync(serverPidFile, `${server.pid}\n`);
  server.stdout.on("data", consume);
  server.stderr.on("data", (chunk) => {
    stderr += chunk.toString();
  });
  await request("initialize", {
    protocolVersion: "2025-06-18",
    capabilities: {},
    clientInfo: { name: "dotfiles-codegraph-probe", version: "1" },
  });
  notify("notifications/initialized");
  const listed = await request("tools/list");
  const names = (listed.tools || []).map((tool) => tool.name).sort();
  if (JSON.stringify(names) !== JSON.stringify(["codegraph_explore"])) {
    throw new Error(`unexpected MCP tools: ${names.join(",")}`);
  }
};

const stopServer = async () => {
  if (!server) return;
  const closing = new Promise((resolve) => server.once("close", resolve));
  server.stdin.end();
  server.kill("SIGTERM");
  await Promise.race([closing, delay(3000)]);
  server = undefined;
};

const resultText = (result) =>
  (result?.content || [])
    .filter((item) => item?.type === "text")
    .map((item) => item.text || "")
    .join("\n");

const explore = async (query, allowError = false) => {
  const started = Date.now();
  const result = await request("tools/call", {
    name: "codegraph_explore",
    arguments: { query },
  });
  timings[query] = Date.now() - started;
  if (result?.isError && !allowError) {
    throw new Error(`codegraph_explore failed: ${resultText(result)}`);
  }
  return resultText(result);
};

const waitFresh = async (query, current, stale) => {
  let last = "";
  for (let attempt = 0; attempt < 30; attempt += 1) {
    try {
      last = await explore(query);
      if (
        current.every((term) => last.includes(term)) &&
        stale.every((term) => !last.includes(term))
      ) {
        return;
      }
    } catch (error) {
      last = String(error);
    }
    await delay(300);
  }
  throw new Error(`freshness timeout for ${query}: ${last}\n${stderr}`);
};

const replace = (file, before, after) => {
  const current = fs.readFileSync(file, "utf8");
  if (!current.includes(before))
    throw new Error(`missing fixture text: ${before}`);
  fs.writeFileSync(file, current.replaceAll(before, after));
};

const branchFile = path.join(source, "branch.ts");
const entryFile = path.join(source, "entry.ts");
const liveFile = path.join(source, "live.ts");
const renamedFile = path.join(source, "renamed-live.ts");
const removableFile = path.join(source, "removable.ts");

try {
  await startServer();
  if (auditPauseMilliseconds > 0) await delay(auditPauseMilliseconds);
  await waitFresh(
    "How does entryValue depend on liveValue, removableValue, and branchMainValue?",
    ["entryValue", "liveValue", "removableValue", "branchMainValue"],
    [],
  );

  run("git", ["switch", "codegraph-alt"]);
  await waitFresh(
    "Where is branchAltValue defined?",
    ["branchAltValue", "FIXTURE_BRANCH_ALT"],
    ["branchMainValue", "FIXTURE_BRANCH_MAIN"],
  );
  run("git", ["switch", "main"]);
  await waitFresh(
    "Where is branchMainValue defined?",
    ["branchMainValue", "FIXTURE_BRANCH_MAIN"],
    ["branchAltValue", "FIXTURE_BRANCH_ALT"],
  );

  replace(liveFile, "FIXTURE_LIVE_V1", "FIXTURE_LIVE_V2");
  await waitFresh(
    "Where is liveSentinel defined?",
    ["FIXTURE_LIVE_V2"],
    ["FIXTURE_LIVE_V1"],
  );

  fs.renameSync(liveFile, renamedFile);
  replace(renamedFile, "liveValue", "renamedLiveValue");
  replace(entryFile, "./live.js", "./renamed-live.js");
  replace(entryFile, "liveValue", "renamedLiveValue");
  await waitFresh(
    "How does entryValue use renamedLiveValue?",
    ["src/renamed-live.ts", "renamedLiveValue"],
    ["src/live.ts", "liveValue"],
  );

  fs.unlinkSync(removableFile);
  replace(entryFile, 'import { removableValue } from "./removable.js"\n', "");
  replace(entryFile, " + removableValue", "");
  await waitFresh(
    "How is entryValue computed?",
    ["entryValue", "renamedLiveValue"],
    ["removableValue", "FIXTURE_REMOVABLE"],
  );

  await stopServer();
  await startServer();
  await waitFresh(
    "How is entryValue computed after restart?",
    ["entryValue", "renamedLiveValue"],
    ["removableValue", "liveValue"],
  );

  await stopServer();
  await startServer(["--no-watch"]);
  replace(branchFile, "FIXTURE_BRANCH_MAIN", "FIXTURE_WATCHER_INTERRUPTED");
  const immediate = await explore("Where is branchSentinel defined?", true);
  const immediateFresh = immediate.includes("FIXTURE_WATCHER_INTERRUPTED");
  const explicitStale = /stale|out[- ]of[- ]date|sync|refresh|reindex/i.test(
    immediate,
  );
  if (!immediateFresh && !explicitStale) {
    throw new Error(
      `watcher interruption returned silent stale output: ${immediate}`,
    );
  }
  await stopServer();
  const synchronizationStarted = Date.now();
  run("codegraph", ["sync", repository]);
  synchronizationMilliseconds = Date.now() - synchronizationStarted;
  await startServer();
  await waitFresh(
    "Where is branchSentinel defined after reconciliation?",
    ["FIXTURE_WATCHER_INTERRUPTED"],
    ["FIXTURE_BRANCH_MAIN"],
  );
  await stopServer();
  await delay(2000);

  const pidFile = path.join(repository, ".codegraph", "daemon.pid");
  if (fs.existsSync(pidFile)) {
    const record = JSON.parse(fs.readFileSync(pidFile, "utf8"));
    try {
      process.kill(record.pid, 0);
      throw new Error(`CodeGraph daemon still running: ${record.pid}`);
    } catch (error) {
      if (error.code !== "ESRCH") throw error;
    }
  }

  process.stdout.write(
    `${JSON.stringify({
      tools: ["codegraph_explore"],
      scenarios: {
        initial: true,
        branchSwitch: true,
        edit: true,
        rename: true,
        delete: true,
        restart: true,
        watcherInterruption: immediateFresh ? "fresh" : "alerted-stale",
        reconciliation: true,
        daemonStopped: true,
      },
      queryLatencyMs: timings,
      synchronizationMs: synchronizationMilliseconds,
    })}\n`,
  );
} finally {
  await stopServer();
}
```

- [ ] **Étape 5 : formater et vérifier le probe avant exécution**

```bash
node --check codegraph/mcp_probe.mjs
prettier --write codegraph/mcp_probe.mjs codegraph/fixtures/freshness/*.json
prettier --check codegraph/mcp_probe.mjs codegraph/fixtures/freshness/*.json
```

Attendu : syntaxe et format verts. Le formatage mécanique est autorisé ; aucun commentaire n'est
ajouté.

- [ ] **Étape 6 : exécuter le test MCP réel**

```bash
bash scripts/codegraph_mcp_test | tee /tmp/issue-97-codegraph-mcp.json
jq -e '.mcp.scenarios.daemonStopped == true' /tmp/issue-97-codegraph-mcp.json
bash -n scripts/codegraph_mcp_test
shellcheck --severity=error scripts/codegraph_mcp_test
```

Attendu : JSON valide, tous les scénarios vrais ou `alerted-stale` pour l'interruption, daemon
arrêté, syntaxe et ShellCheck verts. Si le test révèle du stale silencieux, appliquer
`superpowers:systematic-debugging` et corriger la cause ou ouvrir un ticket upstream ; ne pas
mémoriser une synchronisation supplémentaire.

- [ ] **Étape 7 : ajouter une barrière CI qui couvre les nouveaux tests**

Créer `.github/workflows/test-codegraph.yml` :

```yaml
name: CodeGraph tests

on:
  push:
    paths:
      - .agents/skills/codegraph/**
      - .config/git/ignore
      - codegraph/**
      - scripts/codegraph_*
      - Makefile
      - .github/workflows/test-codegraph.yml
  pull_request:
    paths:
      - .agents/skills/codegraph/**
      - .config/git/ignore
      - codegraph/**
      - scripts/codegraph_*
      - Makefile
      - .github/workflows/test-codegraph.yml

jobs:
  codegraph:
    runs-on: macos-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v5
      - run: make jq tokei codegraph-cli "$HOME/.local/bin/claude" "$HOME/.volta/bin/codex"
      - run: bash .agents/skills/codegraph/scripts/measure_repository_test.sh
      - run: bash scripts/codegraph_configure_test
      - run: bash scripts/codegraph_mcp_test
      - run: bash scripts/codegraph_network_test
```

Ajouter au `Makefile` :

```make
.PHONY: codegraph-test
codegraph-test:
	bash .agents/skills/codegraph/scripts/measure_repository_test.sh
	bash scripts/codegraph_configure_test
	bash scripts/codegraph_mcp_test
	bash scripts/codegraph_network_test
```

- [ ] **Étape 8 : vérifier puis committer la barrière**

```bash
make codegraph-test
node --check codegraph/mcp_probe.mjs
jq empty codegraph/fixtures/freshness/package.json codegraph/fixtures/freshness/tsconfig.json
prettier --check codegraph/mcp_probe.mjs codegraph/fixtures/freshness/*.json .github/workflows/test-codegraph.yml
git diff --check
git add codegraph scripts/codegraph_mcp_test scripts/codegraph_network_test .github/workflows/test-codegraph.yml Makefile
git commit -m "test(codegraph): cover MCP freshness"
```

## Tâche 4 : enregistrer la décision et l'exploitation

**Fichiers :** tous les fichiers du commit 4.

- [ ] **Étape 1 : écrire l'ADR-038**

Créer `docs/adr/038-codegraph-recuperation-structurelle.md` au format MADR du dépôt, avec les
sections et décisions exactes suivantes :

```markdown
# ADR-038 — CodeGraph à la demande pour la récupération structurelle

- **Statut** : accepté
- **Date** : 2026-08

## Contexte

L'issue #86 n'a pas produit de benchmark comparatif valide. L'issue #97 vise désormais une
intégration opérationnelle simple de CodeGraph 1.5.0 dans plusieurs dépôts et trois agents. Les
résultats upstream montrent un coût fixe visible autour de 110 fichiers et des gains plus réguliers
à partir d'environ 640 fichiers ; ils motivent un seuil pragmatique, pas une preuve locale de
supériorité.

## Décision

Installer CodeGraph 1.5.0 globalement et exposer son serveur MCP stdio upstream à Codex, Claude Code
et Cursor. Utiliser `codegraph_explore` pour l'exploration structurelle ; conserver `rg` et `fd`
pour les littéraux, expressions régulières, chemins connus et vérifications ciblées.

Lorsqu'une exploration structurelle rencontre un dépôt sans index, initialiser automatiquement à
partir de 50 000 lignes de source ou 500 fichiers source. Sous les deux seuils, ne pas initialiser.
Un index existant reste utilisable sous le seuil après contrôle de fraîcheur.

Conserver l'état natif `.codegraph/`, ignoré globalement par Git. Ne pas ajouter de wrapper MCP,
proxy, répertoire externe par symlink, daemon maison ni règle permanente dans `ai/AGENTS.md`.
Distribuer la politique conditionnelle par la skill partagée `codegraph`.

## Conséquences

- Chaque dépôt ou worktree indexé paie son propre espace disque.
- La télémétrie, le contrôle de version et le téléchargement de secours sont désactivés à
  l'exécution ; l'installation épinglée reste le seul téléchargement attendu.
- Une panne ou un état périmé produit un repli explicite vers `rg` et `fd`, jamais une réponse
  silencieusement obsolète.
- CodeGraph reste une couche de récupération. Les refactorings sémantiques et le débogage relèvent
  respectivement de LSP et DAP.
- L'intégration complète les ADR-015, ADR-028, ADR-033 et ADR-036 sans les remplacer.

## Alternatives écartées

- Indexer chaque dépôt : coût inutile sur les petits dépôts.
- Laisser l'utilisateur lancer `codegraph init` manuellement : état implicite facile à oublier.
- Stocker les index hors dépôt par symlink : contournement d'une capacité upstream absente.
- Réactiver les outils MCP cachés ou ajouter une façade : surface et maintenance sans besoin établi.
- Ajouter la règle à `ai/AGENTS.md` : instruction permanente sans ablation marginale.
```

- [ ] **Étape 2 : indexer l'ADR**

Ajouter à `docs/adr/README.md` :

```markdown
| [038](038-codegraph-recuperation-structurelle.md) | CodeGraph à la demande pour la récupération structurelle | 2026-08 |
```

- [ ] **Étape 3 : écrire le guide d'exploitation**

Créer `docs/codegraph.md` en français avec ces sections et commandes exactes :

````markdown
# CodeGraph

## Installation

Depuis le checkout canonique uniquement :

```bash
make codegraph
```

La cible épingle CodeGraph 1.5.0, configure Codex, Claude Code et Cursor, désactive les sorties
réseau automatiques et distribue la skill.

## Activation

Les agents utilisent `codegraph-repository-size .` uniquement avant une exploration structurelle
sans index. `initialize: true` autorise `codegraph init`. Une recherche exacte ne déclenche jamais
la mesure.

## Santé et fraîcheur

```bash
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 codegraph status --json
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 codegraph sync
```

Une seule synchronisation est tentée. Un second échec impose un repli explicite vers `rg` et `fd`.

## Cycle de vie

```bash
codegraph daemon
codegraph uninit
codegraph uninstall --target=codex,claude,cursor --location=global --yes --keep-cli
```

`codegraph daemon` permet d'arrêter un daemon identifié. `codegraph uninit` supprime l'index du
dépôt courant après confirmation. La désinstallation retire les configurations agent mais conserve
le CLI ; la suppression du CLI reste gérée par Volta.

## État et confidentialité

L'index réside dans `.codegraph/` et n'est pas versionné. CodeGraph ne nécessite ni Ollama, ni
serveur de modèle, ni embedding distant. Les exclusions du dépôt et les exclusions upstream
écartent les dépendances et sorties générées ; un répertoire sensible suivi doit être exclu par le
dépôt avant initialisation.

## Limites

La surface MCP par défaut contient `codegraph_explore`. `trace` n'existe plus séparément et
`impact` n'est pas réactivé. CodeGraph ne remplace ni LSP ni DAP.
````

- [ ] **Étape 4 : valider et committer la décision**

```bash
prettier --check docs/adr/038-codegraph-recuperation-structurelle.md docs/adr/README.md docs/codegraph.md
! rg -n 'TBD|TODO|FIXME|XXX' docs/adr/038-codegraph-recuperation-structurelle.md docs/codegraph.md
git diff --check
git add docs/adr/038-codegraph-recuperation-structurelle.md docs/adr/README.md docs/codegraph.md
git commit -m "docs(codegraph): record retrieval architecture"
```

Attendu : Prettier sans diff, aucun placeholder, ADR indexée.

## Tâche 5 : smokes des agents, audit réseau et barrière finale

- [ ] **Étape 1 : préparer une copie publique indexée pour les smokes**

```bash
smoke_root=$(mktemp -d)
cleanup_smoke() {
  [ -n "${smoke_root:-}" ] && [ -d "$smoke_root" ] || return
  CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 \
    codegraph uninit --force "$smoke_root/repository" >/dev/null 2>&1 || true
  rm -rf "$smoke_root"
}
trap cleanup_smoke EXIT
cp -R codegraph/fixtures/freshness "$smoke_root/repository"
git -C "$smoke_root/repository" init -b main
git -C "$smoke_root/repository" config user.email codegraph-fixture@example.invalid
git -C "$smoke_root/repository" config user.name 'CodeGraph Fixture'
git -C "$smoke_root/repository" add .
git -C "$smoke_root/repository" commit -m baseline
git -C "$smoke_root/repository" switch -c codegraph-alt
printf '%s\n' \
  'export const branchAltValue = 30' \
  'export const branchSentinel = "FIXTURE_BRANCH_ALT"' >"$smoke_root/repository/src/branch.ts"
git -C "$smoke_root/repository" add src/branch.ts
git -C "$smoke_root/repository" commit -m alternate
git -C "$smoke_root/repository" switch main
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 codegraph init "$smoke_root/repository"
mkdir -p "$smoke_root/repository/.agents/skills" "$smoke_root/repository/.claude/skills" "$smoke_root/repository/.cursor/skills"
ln -s "$(pwd)/.agents/skills/codegraph" "$smoke_root/repository/.agents/skills/codegraph"
ln -s "$(pwd)/.agents/skills/codegraph" "$smoke_root/repository/.claude/skills/codegraph"
ln -s "$(pwd)/.agents/skills/codegraph" "$smoke_root/repository/.cursor/skills/codegraph"
```

Le corpus est la fixture publique ; aucune donnée privée n'atteint les modèles distants.

- [ ] **Étape 2 : exécuter le smoke Claude Code**

Créer le MCP JSON temporaire et exécuter les deux routages :

```bash
jq -n --arg repository "$smoke_root/repository" '
  {
    mcpServers: {
      codegraph: {
        type: "stdio",
        command: "codegraph",
        args: ["serve", "--mcp", "--path", $repository],
        env: {
          CODEGRAPH_TELEMETRY: "0",
          CODEGRAPH_NO_UPDATE_CHECK: "1",
          CODEGRAPH_NO_DOWNLOAD: "1"
        }
      }
    }
  }
' >"$smoke_root/claude-mcp.json"

claude -p --verbose --output-format stream-json --strict-mcp-config --mcp-config "$smoke_root/claude-mcp.json" \
  'Explain how entryValue is assembled and name every dependency. Investigate the repository structure before answering.' \
  >"$smoke_root/claude-positive.jsonl"
rg -q 'codegraph_explore' "$smoke_root/claude-positive.jsonl"
for symbol in liveValue removableValue branchMainValue; do
  rg -q "$symbol" "$smoke_root/claude-positive.jsonl"
done

claude -p --verbose --output-format stream-json --strict-mcp-config --mcp-config "$smoke_root/claude-mcp.json" \
  'Find the exact literal FIXTURE_REMOVABLE with rg and report its file and line.' \
  >"$smoke_root/claude-negative.jsonl"
! rg -q 'codegraph_explore' "$smoke_root/claude-negative.jsonl"
```

- [ ] **Étape 3 : exécuter le smoke Codex**

```bash
codex exec --ephemeral --ignore-user-config --json -C "$smoke_root/repository" \
  -c 'mcp_servers.codegraph.command="codegraph"' \
  -c "mcp_servers.codegraph.args=['serve','--mcp','--path','$smoke_root/repository']" \
  -c 'mcp_servers.codegraph.env={CODEGRAPH_TELEMETRY="0",CODEGRAPH_NO_UPDATE_CHECK="1",CODEGRAPH_NO_DOWNLOAD="1"}' \
  'Explain how entryValue is assembled and name every dependency. Investigate the repository structure before answering.' \
  >"$smoke_root/codex-positive.jsonl"
rg -q 'codegraph_explore' "$smoke_root/codex-positive.jsonl"
for symbol in liveValue removableValue branchMainValue; do
  rg -q "$symbol" "$smoke_root/codex-positive.jsonl"
done

codex exec --ephemeral --ignore-user-config --json -C "$smoke_root/repository" \
  -c 'mcp_servers.codegraph.command="codegraph"' \
  -c "mcp_servers.codegraph.args=['serve','--mcp','--path','$smoke_root/repository']" \
  -c 'mcp_servers.codegraph.env={CODEGRAPH_TELEMETRY="0",CODEGRAPH_NO_UPDATE_CHECK="1",CODEGRAPH_NO_DOWNLOAD="1"}' \
  'Find the exact literal FIXTURE_REMOVABLE with rg and report its file and line.' \
  >"$smoke_root/codex-negative.jsonl"
! rg -q 'codegraph_explore' "$smoke_root/codex-negative.jsonl"
```

- [ ] **Étape 4 : exécuter le smoke Cursor**

Créer la même entrée dans la configuration du projet, puis exécuter les deux routages :

```bash
cat >"$smoke_root/repository/.cursor/mcp.json" <<'JSON'
{
  "mcpServers": {
    "codegraph": {
      "type": "stdio",
      "command": "codegraph",
      "args": ["serve", "--mcp", "--path", "${workspaceFolder}"],
      "env": {
        "CODEGRAPH_TELEMETRY": "0",
        "CODEGRAPH_NO_UPDATE_CHECK": "1",
        "CODEGRAPH_NO_DOWNLOAD": "1"
      }
    }
  }
}
JSON

cursor-agent --print --output-format stream-json --mode ask --approve-mcps --workspace "$smoke_root/repository" \
  'Explain how entryValue is assembled and name every dependency. Investigate the repository structure before answering.' \
  >"$smoke_root/cursor-positive.jsonl"
rg -q 'codegraph_explore' "$smoke_root/cursor-positive.jsonl"
for symbol in liveValue removableValue branchMainValue; do
  rg -q "$symbol" "$smoke_root/cursor-positive.jsonl"
done

cursor-agent --print --output-format stream-json --mode ask --approve-mcps --workspace "$smoke_root/repository" \
  'Find the exact literal FIXTURE_REMOVABLE with rg and report its file and line.' \
  >"$smoke_root/cursor-negative.jsonl"
! rg -q 'codegraph_explore' "$smoke_root/cursor-negative.jsonl"
```

Si un agent encode autrement ses appels dans le JSONL, examiner son schéma et adapter les deux
assertions aux événements réels `codegraph_explore` et `rg` ; ne remplacer aucune preuve par la
seule qualité de la réponse.

- [ ] **Étape 5 : auditer le réseau des processus CodeGraph**

Vérifier le canari récursif, puis échantillonner l'initialisation, la synchronisation, le serveur
MCP, tous leurs descendants et le daemon :

```bash
bash scripts/codegraph_network_test
```

Attendu sur macOS : le canari ouvert par un petit-enfant est détecté, puis aucune socket réseau
n'est observée pendant `init`, `sync` et les requêtes MCP CodeGraph. La socket Unix locale n'est pas
une sortie réseau. Cette preuve ne couvre pas une connexion plus brève que 50 ms et aucun PID n'est
tué par l'audit.

- [ ] **Étape 6 : publier uniquement les preuves observées**

Collecter les valeurs publiques et les versions de l'environnement exercé :

```bash
sw_vers
uname -m
codegraph --version
codex --version
claude --version
cursor-agent --version
jq '{environment, initialIndexSeconds, initialIndexMaxRssBytes, initialIndexCpuUserSeconds, initialIndexCpuSystemSeconds, indexDiskKiB, mcp}' /tmp/issue-97-codegraph-mcp.json
for agent in claude codex cursor; do
  rg -n 'codegraph_explore' "$smoke_root/${agent}-positive.jsonl"
  ! rg -n 'codegraph_explore' "$smoke_root/${agent}-negative.jsonl"
done
```

Créer `docs/codegraph-validation.md` en français, sans chemin ni symbole privé, avec exactement :

- les versions observées de macOS, architecture, CodeGraph, Codex, Claude Code et Cursor ;
- les commandes `make -Bn codegraph`, `make codegraph-test` et l'audit réseau ;
- les valeurs de `/tmp/issue-97-codegraph-mcp.json` pour temps initial, temps CPU, RSS, disque,
  synchronisation et latences ;
- la matrice edit/rename/delete/branch/restart/watcher/reconciliation ;
- le résultat positif et négatif des trois smokes, fondé sur les événements d'appel réels ;
- « Linux non exercé » sans généralisation à cette cible ;
- les écarts avec l'issue : `.codegraph/` local, outil MCP unique, absence de plafond maison ;
- aucune valeur provisoire et aucune donnée provenant d'un dépôt privé.

Valider puis committer ce seul document :

```bash
prettier --check docs/codegraph-validation.md
! rg -n 'TBD|TODO|FIXME|XXX' docs/codegraph-validation.md
git diff --check
git add docs/codegraph-validation.md
git commit -m "docs(codegraph): record local validation"
```

- [ ] **Étape 7 : nettoyer uniquement la fixture**

```bash
cleanup_smoke
trap - EXIT
```

Attendu : seul le répertoire créé par `mktemp -d` est supprimé. Ne jamais utiliser un chemin
résolu depuis une variable vide ou un répertoire large.

- [ ] **Étape 8 : exécuter la barrière finale locale**

```bash
make -Bn codegraph
make codegraph-test
bash -n scripts/codegraph_configure scripts/codegraph_configure_test scripts/codegraph_mcp_test scripts/codegraph_network_test .agents/skills/codegraph/scripts/measure_repository.sh .agents/skills/codegraph/scripts/measure_repository_test.sh
shellcheck --severity=error scripts/codegraph_configure scripts/codegraph_configure_test scripts/codegraph_mcp_test scripts/codegraph_network_test .agents/skills/codegraph/scripts/measure_repository.sh .agents/skills/codegraph/scripts/measure_repository_test.sh
node --check codegraph/mcp_probe.mjs
node --check codegraph/network_canary.mjs
jq empty .agents/skills/codegraph/evals/trigger-queries.json codegraph/fixtures/freshness/package.json codegraph/fixtures/freshness/tsconfig.json
prettier --check .agents/skills/codegraph/SKILL.md .agents/skills/codegraph/evals/trigger-queries.json codegraph/mcp_probe.mjs codegraph/network_canary.mjs codegraph/fixtures/freshness/*.json .github/workflows/test-codegraph.yml docs/adr/038-codegraph-recuperation-structurelle.md docs/adr/README.md docs/codegraph.md docs/codegraph-validation.md
git -c core.excludesFile=.config/git/ignore check-ignore -q .codegraph/index.db
git diff --check
git status --short --branch
```

Attendu : tout vert sur macOS, worktree propre après les commits. Ne pas déclarer Linux vert : la
CI et `/usr/bin/time -l` de ce plan n'exercent que macOS.

- [ ] **Étape 9 : demander la revue et préparer l'intégration**

Invoquer `superpowers:requesting-code-review`, traiter toute remarque bloquante avec
`superpowers:receiving-code-review`, puis réexécuter la barrière concernée. Invoquer ensuite
`superpowers:verification-before-completion` et enfin
`superpowers:finishing-a-development-branch`.

Ne pousser, fusionner ou supprimer le worktree qu'après le choix d'intégration de l'utilisateur. Un
échec intermittent est un bug à diagnostiquer, jamais un motif de retry ou de skip.

## Critères de livraison

- Le pin est exactement CodeGraph 1.5.0 et toute installation passe par le `Makefile`.
- Les trois agents exposent sémantiquement le même serveur local et les mêmes variables de
  confidentialité.
- La skill mesure seulement les explorations structurelles sans index et utilise le seuil
  `50 000 LOC OR 500 fichiers`.
- Les recherches exactes restent sur `rg`/`fd` ; les grands dépôts peuvent être initialisés par les
  agents sans préparation manuelle.
- Les mutations MCP, le redémarrage, le watcher interrompu, la réconciliation et l'arrêt du daemon
  sont prouvés sur une fixture publique.
- Les preuves publiées ne contiennent aucune donnée de dépôt privé.
- Aucun commentaire nouveau n'est attendu ; le compte rendu final liste explicitement
  « commentaires ajoutés : aucun ».
