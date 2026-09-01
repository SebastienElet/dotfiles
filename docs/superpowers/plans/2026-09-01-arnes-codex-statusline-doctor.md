# Codex Statusline Doctor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ajouter un diagnostic en lecture seule qui compare les déclarations de status line Codex du manifeste à `tui.status_line` dans la configuration native.

**Architecture:** Le manifeste expose une projection Codex par scope avec une liste ordonnée d'identifiants opaques. Un module `statusline` dédié lit uniquement le fichier TOML du scope demandé, valide strictement le chemin `tui.status_line`, puis produit les diagnostics Doctor sans appeler Codex ni aucun shell. Claude et Cursor restent silencieusement hors périmètre, avec un unique commentaire de code sourcé qui documente cette frontière.

**Tech Stack:** Rust 2024, `serde`, `serde_yaml_ng`, `toml`, `clap`, tests d'intégration Cargo avec fixtures isolées.

**Spec:** `docs/superpowers/specs/2026-09-01-arnes-codex-statusline-doctor-design.md`

## Global Constraints

- Le diagnostic est strictement en lecture seule et n'exécute ni Codex ni shell.
- Claude et Cursor ne produisent aucun diagnostic `unsupported` ; le rendu vide commun reste inchangé.
- Les identifiants de status line sont comparés comme des chaînes opaques et ordonnées.
- Un format externe inattendu produit `error`, jamais une valeur de repli plausible.
- Les clés TOML sans rapport et leurs valeurs ne doivent jamais apparaître dans la sortie.
- `arnes doctor` n'agrège pas encore cette ressource ; l'issue #123 porte cette évolution.
- Les fichiers de production restent sous 250 lignes et les fonctions sous 50 lignes logiques, sauf justification explicite.

---

### Task 1: Déclaration normalisée des status lines Codex

**Files:**
- Create: `tooling/arnes/src/manifest/statusline.rs`
- Create: `tooling/arnes/src/manifest/validation/statusline.rs`
- Create: `tooling/arnes/tests/manifest_statusline.rs`
- Modify: `tooling/arnes/src/manifest.rs`
- Modify: `tooling/arnes/src/manifest/validation.rs`

**Interfaces:**
- Consumes: `Agent`, `Scope`, `ManifestError` et `validation::validate_target` existants.
- Produces: `Statusline<'a> { agent: Agent, scope: Scope, items: &'a [String] }` et `Manifest::statuslines() -> impl Iterator<Item = Statusline<'_>>`.

- [ ] **Step 1: Écrire les tests de parsing et de validation qui échouent**

Créer `tooling/arnes/tests/manifest_statusline.rs` avec un manifeste minimal et des attentes littérales :

```rust
use arnes::manifest::{self, Agent, Scope};

fn input(statuslines: &str) -> String {
    input_with_codex_scopes("user, project", statuslines)
}

fn input_with_codex_scopes(scopes: &str, statuslines: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\n  - id: cursor\n    scopes: [user]\n  - id: codex\n    scopes: [{scopes}]\nstatuslines:{statuslines}\nresources: []\n"
    )
}

fn error(statuslines: &str) -> String {
    manifest::parse(&input(statuslines))
        .err()
        .unwrap()
        .to_string()
}

#[test]
fn parses_ordered_codex_statusline_projections() {
    let manifest = manifest::parse(&input(
        "\n  - { agent: codex, scope: user, items: [model-with-reasoning, current-dir] }\n  - { agent: codex, scope: project, items: [context-used] }",
    ))
    .unwrap();
    let projections = manifest.statuslines().collect::<Vec<_>>();

    assert_eq!(projections.len(), 2);
    assert_eq!((projections[0].agent, projections[0].scope), (Agent::Codex, Scope::User));
    assert_eq!(projections[0].items, ["model-with-reasoning", "current-dir"]);
    assert_eq!((projections[1].agent, projections[1].scope), (Agent::Codex, Scope::Project));
}

#[test]
fn absent_statusline_declarations_remain_compatible() {
    assert_eq!(manifest::parse(&input(" []")).unwrap().statuslines().count(), 0);
}

#[test]
fn rejects_unsupported_duplicate_and_empty_statuslines() {
    for (statuslines, expected) in [
        ("\n  - { agent: claude, scope: user, items: [model] }", "statuslines[0].agent: only codex status lines are supported"),
        ("\n  - { agent: cursor, scope: user, items: [model] }", "statuslines[0].agent: only codex status lines are supported"),
        ("\n  - { agent: codex, scope: user, items: [] }", "statuslines[0].items: cannot be empty"),
        ("\n  - { agent: codex, scope: user, items: [''] }", "statuslines[0].items[0]: cannot be blank"),
        ("\n  - { agent: codex, scope: user, items: [model] }\n  - { agent: codex, scope: user, items: [dir] }", "statuslines[1].scope: duplicates statuslines[0] projection"),
    ] {
        assert_eq!(error(statuslines), expected);
    }
}

#[test]
fn statusline_target_must_be_declared() {
    assert_eq!(
        manifest::parse(&input_with_codex_scopes(
            "user",
            "\n  - { agent: codex, scope: project, items: [model] }",
        ))
        .err()
        .unwrap()
        .to_string(),
        "statuslines[0].scope: scope is not declared for this agent"
    );
}
```

- [ ] **Step 2: Exécuter le test ciblé et vérifier l'échec RED**

Run: `cargo test --locked --test manifest_statusline`

Expected: FAIL à la compilation car `Manifest::statuslines` et le modèle de déclaration n'existent pas.

- [ ] **Step 3: Implémenter le modèle et sa validation minimale**

Créer `tooling/arnes/src/manifest/statusline.rs` :

```rust
use super::{Agent, Manifest, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StatuslineDeclaration {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) items: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct Statusline<'a> {
    pub agent: Agent,
    pub scope: Scope,
    pub items: &'a [String],
}

impl Manifest {
    pub fn statuslines(&self) -> impl Iterator<Item = Statusline<'_>> {
        self.statuslines.iter().map(|declaration| Statusline {
            agent: declaration.agent,
            scope: declaration.scope,
            items: &declaration.items,
        })
    }
}
```

Brancher `mod statusline`, `pub use statusline::Statusline`, le champ `#[serde(default)] statuslines`, puis `statusline::validate(...)` dans `Manifest` et sa validation. La validation dédiée doit appliquer `validate_target`, refuser tout agent autre que Codex, une liste vide, un item `trim().is_empty()` et les doublons `(agent, scope)` avec les messages testés.

- [ ] **Step 4: Exécuter les tests du manifeste et vérifier GREEN**

Run: `cargo test --locked --test manifest_statusline --test manifest --test manifest_mcp`

Expected: PASS, sans warning.

- [ ] **Step 5: Commit**

```bash
git add tooling/arnes/src/manifest.rs tooling/arnes/src/manifest/statusline.rs tooling/arnes/src/manifest/validation.rs tooling/arnes/src/manifest/validation/statusline.rs tooling/arnes/tests/manifest_statusline.rs
git commit -m "feat(arnes): declare Codex status lines"
```

---

### Task 2: Lecture TOML stricte et diagnostic Doctor

**Files:**
- Create: `tooling/arnes/src/statusline.rs`
- Create: `tooling/arnes/src/statusline/configuration.rs`
- Create: `tooling/arnes/tests/statusline.rs`
- Modify: `tooling/arnes/src/lib.rs`
- Modify: `tooling/arnes/src/doctor.rs`

**Interfaces:**
- Consumes: `Roots`, `Manifest::statuslines()`, `Diagnostic`, `State`, `Agent`, `Scope`.
- Produces: `statusline::diagnose(roots: &Roots, manifest: &Manifest, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic>` et `configuration::load(roots: &Roots, scope: Scope) -> Result<Option<Vec<String>>, ConfigurationError>`.

- [ ] **Step 1: Écrire les tests CLI de frontière et la caractérisation du silence existant**

Créer le helper suivant dans `tooling/arnes/tests/statusline.rs` :

```rust
mod support;

use support::Fixture;

fn manifest(scope: &str, items: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: claude\n    scopes: [user, project]\n  - id: cursor\n    scopes: [user, project]\n  - id: codex\n    scopes: [user, project]\nstatuslines:\n  - {{ agent: codex, scope: {scope}, items: [{items}] }}\nresources: []\n"
    )
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}
```

Ajouter quatre tests indépendants :

```rust
#[test]
fn matching_user_statusline_is_healthy_without_mutation() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model-with-reasoning, current-dir"));
    fixture.write_home(
        ".codex/config.toml",
        "secret = \"not-rendered\"\n[tui]\nstatus_line = [\"model-with-reasoning\", \"current-dir\"]\n",
    );
    let before = fixture.snapshot();

    let output = fixture.command(["doctor", "statusline", "--agent", "codex", "--scope", "user", "-v"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("healthy statusline: codex user"));
    assert!(!stdout(&output).contains("not-rendered"));
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn ordered_statusline_mismatch_is_drift() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model-with-reasoning, current-dir"));
    fixture.write_home(
        ".codex/config.toml",
        "[tui]\nstatus_line = [\"current-dir\", \"model-with-reasoning\"]\n",
    );

    let output = fixture.command(["doctor", "statusline", "--agent", "codex"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("ordered items differ"));
}

#[test]
fn missing_statusline_is_drift() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));

    let output = fixture.command(["doctor", "statusline"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("configuration is missing"));
}

#[test]
fn unsupported_agents_and_undeclared_scopes_are_silent() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));

    for args in [
        ["doctor", "statusline", "--agent", "claude", "--format", "json"],
        ["doctor", "statusline", "--agent", "cursor", "--format", "json"],
        ["doctor", "statusline", "--scope", "project", "--format", "json"],
    ] {
        let output = fixture.command(args);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(stdout(&output), "[]\n");
    }
}
```

Ajouter dans le même fichier les cas de format externe invalide, de scope projet et de clé absente :

```rust
#[test]
fn malformed_or_wrongly_typed_codex_configuration_is_error() {
    for (contents, expected) in [
        ("not = [", "Codex configuration is malformed"),
        ("tui = true", "tui must be a table"),
        ("[tui]\nstatus_line = true", "tui.status_line must be an array of strings"),
        ("[tui]\nstatus_line = [1]", "tui.status_line must be an array of strings"),
    ] {
        let fixture = Fixture::new();
        fixture.write_home(".arnes.yaml", &manifest("user", "model"));
        fixture.write_home(".codex/config.toml", contents);

        let output = fixture.command(["doctor", "statusline"]);

        assert_eq!(output.status.code(), Some(2));
        assert!(stdout(&output).contains(expected));
    }
}

#[test]
fn project_scope_reads_only_project_codex_configuration() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("project", "project-item"));
    fixture.write_home(".codex/config.toml", "[tui]\nstatus_line = [\"wrong-user-item\"]\n");
    fixture.write_repository(".codex/config.toml", "[tui]\nstatus_line = [\"project-item\"]\n");

    let output = fixture.command(["doctor", "statusline", "--scope", "project", "-v"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("healthy statusline: codex project"));
}
```

Ajouter les deux frontières restantes :

```rust
#[test]
fn missing_statusline_key_is_drift() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));
    fixture.write_home(".codex/config.toml", "[tui]\nanimations = false\n");

    let output = fixture.command(["doctor", "statusline"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("configuration is missing"));
}

#[test]
fn configuration_symlink_cannot_escape_scope_root() {
    use std::fs;
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));
    let outside = tempfile::tempdir().unwrap();
    let external = outside.path().join("config.toml");
    fs::write(&external, "secret = \"outside\"\n[tui]\nstatus_line = [\"model\"]\n").unwrap();
    fs::create_dir_all(fixture.home().join(".codex")).unwrap();
    symlink(&external, fixture.home().join(".codex/config.toml")).unwrap();

    let output = fixture.command(["doctor", "statusline"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).contains("escapes its scope root"));
    assert!(!stdout(&output).contains("outside"));
}
```

- [ ] **Step 2: Exécuter les tests ciblés et vérifier l'échec RED**

Run: `cargo test --locked --test statusline`

Expected: les cas Codex échouent parce que `doctor.rs` renvoie encore un rapport vide ; le test de
silence peut déjà passer comme caractérisation du fallback existant. Confirmer que les autres
échecs viennent du comportement absent et non des fixtures.

- [ ] **Step 3: Implémenter le diagnostic et le lecteur minimal**

Dans `tooling/arnes/src/statusline.rs`, filtrer les déclarations par agent et scope, puis produire exactement un diagnostic par déclaration. Ne produire aucun diagnostic quand la sélection est vide. Ajouter près de ce filtre l'unique commentaire de production demandé :

```rust
// Claude Code status lines execute shell commands (code.claude.com/docs/en/statusline), while Cursor publishes no persistent schema; only Codex tui.status_line is diagnosed statically.
```

Le lecteur `configuration::load` choisit `~/.codex/config.toml` pour `user` et `.codex/config.toml` pour `project`, refuse les symlinks qui sortent de la racine via `canonical_within`, puis parse uniquement la valeur utile :

```rust
let value = toml::from_str::<toml::Value>(input)
    .map_err(|_| ConfigurationError::new("Codex configuration is malformed"))?;
let Some(tui) = value.get("tui") else {
    return Ok(None);
};
let tui = tui
    .as_table()
    .ok_or_else(|| ConfigurationError::new("tui must be a table"))?;
let Some(status_line) = tui.get("status_line") else {
    return Ok(None);
};
status_line
    .as_array()
    .ok_or_else(|| ConfigurationError::new("tui.status_line must be an array of strings"))?
    .iter()
    .map(|item| {
        item.as_str()
            .map(str::to_owned)
            .ok_or_else(|| ConfigurationError::new("tui.status_line must be an array of strings"))
    })
    .collect::<Result<Vec<_>, _>>()
    .map(Some)
```

Pour un fichier, une table ou une clé absente, retourner `Ok(None)` afin que l'orchestrateur produise
`drift`. Pour un document illisible, non UTF-8, mal formé ou mal typé, retourner une
`ConfigurationError` sans contenu du fichier. Dans `doctor.rs`, remplacer le fallback de
`Resource::Statusline` par un branchement analogue à `Mcp`, sans modifier `diagnose_default`.
Exporter `pub mod statusline` depuis `lib.rs`.

- [ ] **Step 4: Exécuter le test ciblé et vérifier GREEN**

Run: `cargo test --locked --test statusline`

Expected: PASS, sans warning.

- [ ] **Step 5: Commit**

```bash
git add tooling/arnes/src/lib.rs tooling/arnes/src/statusline.rs tooling/arnes/src/statusline/configuration.rs tooling/arnes/src/doctor.rs tooling/arnes/tests/statusline.rs
git commit -m "feat(arnes): diagnose Codex status lines"
```

---

### Task 3: Déclaration maintenue et barrière complète

**Files:**
- Modify: `home/.arnes.yaml`

**Interfaces:**
- Consumes: le schéma `statuslines` et `arnes doctor statusline` ajoutés dans les tâches précédentes.
- Produces: la projection utilisateur Codex maintenue par le dépôt, égale à la configuration actuelle attendue.

- [ ] **Step 1: Ajouter la déclaration maintenue**

Ajouter avant `mcp:` dans `home/.arnes.yaml` :

```yaml
statuslines:
  - agent: codex
    scope: user
    items:
      - model-with-reasoning
      - current-dir
      - context-used
      - context-window-size
```

- [ ] **Step 2: Vérifier le diagnostic réel sans exécution**

Run depuis `tooling/arnes` : `cargo run --locked -- doctor statusline --agent codex --scope user --verbose`

Expected sur le macOS de développement : exit `0` et un diagnostic `healthy statusline: codex user`; si l'état local a changé depuis la discovery, conserver la dérive comme preuve réelle et ne pas modifier silencieusement la déclaration.

- [ ] **Step 3: Exécuter la barrière complète**

Run depuis `tooling/arnes` :

```bash
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo check --all-targets --locked
cargo test --locked
```

Run depuis la racine : `git diff --check`

Expected: toutes les commandes sortent avec le code `0`. Si Moon est disponible, exécuter aussi `moon run arnes:fmt arnes:clippy arnes:check arnes:test`; sinon consigner son absence et les commandes Cargo équivalentes déjà exercées.

- [ ] **Step 4: Vérifier les limites de taille et les commentaires**

Run : `wc -l tooling/arnes/src/manifest.rs tooling/arnes/src/doctor.rs tooling/arnes/src/statusline.rs tooling/arnes/src/statusline/configuration.rs tooling/arnes/tests/statusline.rs`

Expected: chaque fichier de production manuscrit reste sous 250 lignes ; inspecter toute fonction de production modifiée et confirmer qu'elle reste sous 50 lignes logiques. Vérifier que le seul commentaire de production ajouté est celui qui documente la frontière Claude/Cursor et noter son fait externe dans la livraison.

- [ ] **Step 5: Vérifier le diff sémantique et auto-relire**

Exécuter `semctx_verify_change` si le dépôt devient initialisé ; sinon consigner le no-op. Relire le diff contre la spec et appliquer les dix classes de `harness/skills/pr-verdict/references/failure-classes.md`, en particulier parsing versus assertion et claim stronger than mechanism.

- [ ] **Step 6: Commit**

```bash
git add home/.arnes.yaml
git commit -m "chore(arnes): declare Codex status line"
```

- [ ] **Step 7: Demander une revue indépendante**

Fournir au reviewer la spec, ce plan et le diff entre `origin/main` et `HEAD`. Corriger tout constat Critical ou Important, puis rejouer la barrière complète avant d'annoncer la fin.
