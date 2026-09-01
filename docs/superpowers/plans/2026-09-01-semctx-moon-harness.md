# Semctx Moon Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Installer durablement les plugins Semctx détectés au moyen de Bun et de tâches Moon sous le namespace `harness:*`, sans modifier les profils Make.

**Architecture:** Le projet Bun racine verrouille le CLI `semctx` ; un projet Moon dédié sous `harness/` expose l'installation machine, l'initialisation de dépôt et le diagnostic comme trois effets distincts. Arnes autorise et observe les plugins et skills externes installés, sans devenir leur installateur.

**Tech Stack:** Bun 1.4.0, semctx 0.1.17, Moon 2.5.3, YAML, Arnes/Rust

**Spec:** `docs/superpowers/specs/2026-09-01-semctx-moon-harness-design.md`

## Global Constraints

- Ne modifier ni `make minimal`, ni `make optional`, ni leurs dépendances.
- `package.json` est l'unique source de version Semctx ; `bun.lock` verrouille sa résolution exacte.
- Les tâches qui lisent ou modifient l'état du poste désactivent le cache Moon et ne s'exécutent pas dans `moon ci`.
- `harness:install` installe le plugin mais n'initialise jamais `.semctx/`.
- `harness:semctx-setup` reste une action volontaire séparée et utilise la configuration v2 `--polyglot` pour un dépôt neuf.
- Aucun chemin utilisateur, secret ou retrait de marketplace n'entre dans les fichiers versionnés.
- Le worktree vérifie la configuration sans installer, retirer ou mettre à jour un plugin réel.
- L'installation réelle et la migration du canal local s'exécutent seulement depuis le checkout canonique après intégration.

---

## File Structure

- Modify: `package.json` — déclare la version exacte du CLI Semctx utilisée par Moon.
- Modify: `bun.lock` — verrouille l'artefact npm et ses dépendances.
- Modify: `.moon/workspace.yml` — enregistre le projet Moon `harness`.
- Create: `harness/moon.yml` — possède exclusivement les tâches `harness:*` et leurs contrats d'effets.
- Modify: `home/.arnes.yaml` — autorise l'inventaire externe Semctx pour Claude Code et Codex.

### Task 1: Verrouiller le CLI Semctx avec Bun

**Files:**

- Modify: `package.json`
- Modify: `bun.lock`

**Interfaces:**

- Consumes: la toolchain Bun 1.4.0 définie par `packageManager` et `.moon/toolchains.yml`.
- Produces: le binaire local `semctx` en version exacte `0.1.17`, résolu par `bun run semctx`.

- [ ] **Step 1: Établir l'absence initiale du binaire projet**

Run:

```bash
bun run semctx --version
```

Expected: FAIL avec `Could not locate executable semctx` ou un diagnostic équivalent ; un binaire global éventuel ne satisfait pas ce contrôle.

- [ ] **Step 2: Déclarer la dépendance exacte**

Dans `package.json`, ajouter `semctx` aux `devDependencies` en conservant l'ordre alphabétique :

```json
{
  "devDependencies": {
    "@types/bun": "1.4.0",
    "oxfmt": "0.64.0",
    "oxlint": "1.80.0",
    "oxlint-tsgolint": "7.0.2001",
    "prettier": "3.9.6",
    "semctx": "0.1.17",
    "typescript": "7.0.2"
  }
}
```

- [ ] **Step 3: Régénérer mécaniquement le lockfile**

Run:

```bash
bun install --ignore-scripts --config=/dev/null --no-env-file
```

Expected: `bun.lock` contient la résolution de `semctx@0.1.17`; aucun script de paquet n'est exécuté.

- [ ] **Step 4: Vérifier la résolution et le gel**

Run:

```bash
bun run semctx --version
bun install --frozen-lockfile --ignore-scripts --config=/dev/null --no-env-file
bun run prettier --check package.json
```

Expected: la première commande affiche `0.1.17`; l'installation gelée et Prettier réussissent.

- [ ] **Step 5: Committer la toolchain verrouillée**

```bash
git add package.json bun.lock
git commit -m "build(harness): pin semctx CLI"
```

### Task 2: Ajouter le projet et les tâches Moon du harnais

**Files:**

- Modify: `.moon/workspace.yml`
- Create: `harness/moon.yml`

**Interfaces:**

- Consumes: `bun run semctx` produit par Task 1 et le projet racine Moon existant.
- Produces: `harness:install`, `harness:semctx-install`, `harness:semctx-setup` et `harness:semctx-status`.

- [ ] **Step 1: Prouver que le namespace n'existe pas encore**

Run:

```bash
moon project harness --json
```

Expected: FAIL avec un projet `harness` inconnu.

- [ ] **Step 2: Enregistrer le projet dans le workspace**

Dans `.moon/workspace.yml`, ajouter l'entrée suivante à `projects`, entre `repository` et les projets `tooling/*` :

```yaml
projects:
  repository: .
  harness: harness
  tooling/agent-handoff: tooling/agent-handoff
  tooling/arnes: tooling/arnes
```

- [ ] **Step 3: Créer les quatre tâches avec leurs frontières d'effets**

Créer `harness/moon.yml` avec ce contenu :

```yaml
language: typescript
layer: configuration

taskOptions:
  cache: false
  runFromWorkspaceRoot: true
  runInCI: false

tasks:
  install:
    description: Install every declared harness capability
    command: noop
    deps:
      - ~:semctx-install
  semctx-install:
    description: Install or update Semctx plugins for detected agent hosts
    command: bun run semctx install --host auto --skip-setup --json
    options:
      mutex: harness-mutation
      outputStyle: stream
  semctx-setup:
    description: Initialize Semctx in the current Git repository
    command: bun run semctx setup --polyglot --json
    options:
      mutex: harness-mutation
      outputStyle: stream
  semctx-status:
    description: Inspect local Semctx plugin delivery without network attestation
    command: bun run semctx plugin-status --host auto --json
```

Le mutex commun empêche l'installation machine et l'initialisation du dépôt de muter simultanément des états Semctx ; le diagnostic reste indépendant et en lecture seule.

- [ ] **Step 4: Vérifier le schéma, les options et le graphe sans exécuter les tâches**

Run:

```bash
moon project harness --json
moon task harness:install --json
moon task harness:semctx-install --json
moon task harness:semctx-setup --json
moon task harness:semctx-status --json
moon task-graph harness:install --json
```

Expected:

- les six commandes réussissent ;
- les quatre tâches ont `cache: false`, `runInCI: false` et le workspace root comme répertoire d'exécution ;
- `harness:install` dépend uniquement de `harness:semctx-install` ;
- seules les deux tâches mutantes portent le mutex `harness-mutation`.

- [ ] **Step 5: Vérifier le format YAML**

Run:

```bash
bun run prettier --check .moon/workspace.yml harness/moon.yml
```

Expected: PASS.

- [ ] **Step 6: Committer le namespace Moon**

```bash
git add .moon/workspace.yml harness/moon.yml
git commit -m "feat(harness): add Semctx Moon tasks"
```

### Task 3: Déclarer Semctx dans la politique externe Arnes

**Files:**

- Modify: `home/.arnes.yaml`
- Test: `tooling/arnes/tests/manifest.rs`

**Interfaces:**

- Consumes: les identifiants publiés `semctx@semctx-stable` pour Claude Code et `semctx-control@semctx-stable` pour Codex.
- Produces: une politique Arnes qui autorise explicitement chaque plugin et chaque skill exposé, sans en exiger la présence.

- [ ] **Step 1: Établir que le manifeste courant reste valide avant modification**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --locked --test manifest repository_manifest_is_valid -- --exact
```

Expected: PASS ; ce test exerce le vrai parseur sur `home/.arnes.yaml`.

- [ ] **Step 2: Autoriser les deux plugins externes**

Dans `home/.arnes.yaml`, ajouter à `external.plugins` :

```yaml
- { agent: claude, scope: user, id: semctx@semctx-stable }
- { agent: codex, scope: user, id: semctx-control@semctx-stable }
```

Conserver les entrées regroupées par agent, puis par identifiant.

- [ ] **Step 3: Autoriser chaque skill publiée par ces plugins**

Dans `external.skills`, ajouter exactement :

```yaml
- {
    agent: claude,
    scope: user,
    origin: plugin,
    plugin: semctx@semctx-stable,
    slug: semctx-control,
  }
- {
    agent: claude,
    scope: user,
    origin: plugin,
    plugin: semctx@semctx-stable,
    slug: semctx-semantic,
  }
- {
    agent: claude,
    scope: user,
    origin: plugin,
    plugin: semctx@semctx-stable,
    slug: semctx-verify,
  }
- {
    agent: codex,
    scope: user,
    origin: plugin,
    plugin: semctx-control@semctx-stable,
    slug: semctx-control,
  }
```

Ces autorisations ne rendent pas les plugins obligatoires et n'adoptent pas leurs fichiers dans `harness/skills/`.

- [ ] **Step 4: Vérifier le manifeste et les gates Arnes affectées**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --locked --test manifest repository_manifest_is_valid -- --exact
moon run tooling/arnes:fmt tooling/arnes:clippy tooling/arnes:test
bun run prettier --check home/.arnes.yaml
```

Expected: tous les contrôles passent sur macOS ; aucun diagnostic de duplication plugin/skill n'apparaît.

- [ ] **Step 5: Committer la politique d'observation**

```bash
git add home/.arnes.yaml
git commit -m "chore(arnes): allow Semctx plugins"
```

### Task 4: Vérifier la tranche sans muter le poste

**Files:**

- Verify: `package.json`
- Verify: `bun.lock`
- Verify: `.moon/workspace.yml`
- Verify: `harness/moon.yml`
- Verify: `home/.arnes.yaml`

**Interfaces:**

- Consumes: les livrables des Tasks 1 à 3.
- Produces: les preuves locales de résolution, formatage, typage et graphe ; aucune preuve d'installation effective sur le poste.

- [ ] **Step 1: Vérifier le diff et les fichiers manuscrits**

Run:

```bash
git diff --check HEAD~3..HEAD
bun run prettier --check package.json .moon/workspace.yml harness/moon.yml home/.arnes.yaml
```

Expected: PASS ; `harness/moon.yml` reste largement sous le seuil de 250 lignes et ne contient aucun commentaire.

- [ ] **Step 2: Exécuter les gates racine affectées par le manifeste Bun et Moon**

Run:

```bash
moon run repository:typescript-lint repository:typescript-typecheck repository:typescript-format-check repository:prettier-check
bun test
```

Expected: PASS sur macOS avec Moon 2.5.3 et Bun 1.4.0.

- [ ] **Step 3: Réinspecter le projet et son graphe final**

Run:

```bash
moon project harness --json
moon task-graph harness:install --json
```

Expected: le projet expose exactement les quatre tâches prévues et l'agrégateur ne contient que l'arête vers `harness:semctx-install`.

- [ ] **Step 4: Prévisualiser l'installateur Semctx directement**

Run:

```bash
bun run semctx install --host auto --skip-setup --dry-run --json
```

Expected sur le poste actuel: FAIL fermé signalant que `semctx-stable` pointe vers une source locale non publique. Conserver le code de sortie et le reason code exacts ; ce refus prouve la protection contre l'écrasement, pas la convergence du poste.

- [ ] **Step 5: Documenter la limite de preuve dans la livraison**

La livraison doit distinguer :

```text
Repository configuration: verified in the Codex worktree on macOS
Plugin installation: not run from the worktree
Local marketplace migration: pending explicit authorization in the canonical checkout
Repository setup: not run; harness:semctx-setup remains explicit
```

Ne pas déclarer Semctx installé ou actif tant que le chemin opérateur ci-dessous n'a pas produit son propre rapport vert.

## Canonical-checkout Operator Handoff

Après intégration des commits dans le checkout canonique :

1. Inspecter `codex plugin marketplace list --json`, `codex plugin list --json` et l'inventaire Claude Code sans mutation.
2. Présenter les identifiants et sources exacts de la marketplace locale à retirer, puis demander une autorisation explicite.
3. Après autorisation, retirer uniquement le plugin et la marketplace Semctx locaux observés ; ne jamais coder ces commandes dans Moon.
4. Exécuter `moon run harness:install` depuis le checkout canonique.
5. Exiger `ok: true` et, pour chaque hôte détecté, un plugin installé, activé et à la version `0.1.17` avant de déclarer l'installation convergée.
6. Ouvrir une nouvelle tâche Codex ou recharger les plugins Claude Code selon les actions retournées par Semctx.
7. Exécuter `moon run harness:semctx-status`; rapporter séparément delivery, activation de session et toute preuve absente.
8. Exécuter `moon run harness:semctx-setup` seulement sur demande explicite d'initialiser le dépôt courant, puis vérifier séparément binding, freshness, coverage et diagnostics workspace.
