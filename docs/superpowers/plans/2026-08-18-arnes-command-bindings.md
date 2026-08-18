# Arnes Command Binding Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implémenter `arnes doctor commands` pour valider les commandes logiques et leurs bindings
Claude, tout en signalant Cursor et Codex `unsupported`.

**Architecture:** Ajouter `commands[]` au manifeste sous forme de commandes logiques à bindings
imbriqués. L'orchestrateur filtre les bindings avant toute I/O, réutilise la validation ciblée des
projections de prompts, puis contrôle uniquement la destination et la description propres au
binding. Parsing, validation statique, capacité, metadata et orchestration restent séparés.

**Tech Stack:** Rust 2024, Clap, Serde, `serde_yaml_ng`, tests d'intégration Cargo, fixtures
`tempfile`, macOS local et CI Ubuntu existante.

---

## Cartographie des fichiers

- Créer `tooling/arnes/src/manifest/commands.rs` : déclarations YAML et vues empruntées.
- Créer `tooling/arnes/src/manifest/validation/commands.rs` : invariants statiques purs.
- Modifier `tooling/arnes/src/manifest.rs` et `tooling/arnes/src/manifest/validation.rs` : câblage du
  schéma normalisé.
- Créer `tooling/arnes/src/commands.rs` : filtres, ordre et diagnostics.
- Créer `tooling/arnes/src/commands/capability.rs` : matrice agent/scope et destination dérivée.
- Créer `tooling/arnes/src/commands/binding.rs` : parsing du frontmatter et description.
- Modifier `tooling/arnes/src/prompts.rs` et `tooling/arnes/src/prompts/projection.rs` : primitive
  interne réutilisable et réexport du tracker existant, sans changement CLI.
- Modifier `tooling/arnes/src/lib.rs` et `tooling/arnes/src/main.rs` : exposition et routage.
- Créer `tooling/arnes/tests/manifest_commands.rs`, `tooling/arnes/tests/support/commands.rs`,
  `tooling/arnes/tests/commands.rs`, `tooling/arnes/tests/command_failures.rs` et
  `tooling/arnes/tests/command_topology.rs` : couverture séparée sous le seuil de 250 lignes.

### Tâche 1: Normaliser et valider `commands[]`

**Fichiers :**
- Créer : `tooling/arnes/src/manifest/commands.rs`
- Créer : `tooling/arnes/src/manifest/validation/commands.rs`
- Créer : `tooling/arnes/tests/manifest_commands.rs`
- Modifier : `tooling/arnes/src/manifest.rs`
- Modifier : `tooling/arnes/src/manifest/validation.rs`

- [ ] **Étape 1: Écrire les tests RED du modèle emprunté et de sa compatibilité v1**

Commencer `tests/manifest_commands.rs` par ce helper autonome :

```rust
use arnes::manifest::{self, Agent, Scope};

fn input(commands: &str) -> String {
    format!(
        "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: cursor
    scopes: [project]
  - id: codex
    scopes: [user, project]
prompts:
  - id: deploy
    source: {{ root: repository, path: harness/prompts/deploy.md }}
    includes: []
    variables: []
    projections: []
commands:{commands}
resources: []
"
    )
}

fn error(commands: &str) -> String {
    manifest::parse(&input(commands)).err().unwrap().to_string()
}
```

Ajouter ensuite :

```rust
#[test]
fn command_getters_preserve_command_and_binding_order() {
    let manifest = manifest::parse(&input(
        "\n  - name: deploy\n    description: Deploy safely\n    prompt: deploy\n    bindings:\n      - { agent: claude, scope: user }\n      - { agent: cursor, scope: project }",
    ))
    .unwrap();
    let commands = manifest.commands().collect::<Vec<_>>();
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].name(), "deploy");
    assert_eq!(commands[0].description(), "Deploy safely");
    assert_eq!(commands[0].prompt(), "deploy");
    let bindings = commands[0].bindings().collect::<Vec<_>>();
    assert_eq!((bindings[0].agent, bindings[0].scope), (Agent::Claude, Scope::User));
    assert_eq!((bindings[1].agent, bindings[1].scope), (Agent::Cursor, Scope::Project));
    assert_eq!(bindings[1].name(), "deploy");
    assert_eq!(bindings[1].description(), "Deploy safely");
    assert_eq!(bindings[1].prompt(), "deploy");
}

#[test]
fn absent_commands_remain_compatible_with_manifest_v1() {
    let manifest = manifest::parse(&input(" []")).unwrap();
    assert_eq!(manifest.commands().count(), 0);
}
```

- [ ] **Étape 2: Exécuter les tests et constater l'échec attendu**

```bash
cd tooling/arnes
cargo test --test manifest_commands
```

Attendu : FAIL à la compilation, `Manifest::commands` et les types de commande n'existent pas.

- [ ] **Étape 3: Ajouter le modèle et ses vues aplaties sans dupliquer les chaînes**

Implémenter dans `manifest/commands.rs` :

```rust
use super::{Agent, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CommandDeclaration {
    pub(super) name: String,
    pub(super) description: String,
    pub(super) prompt: String,
    pub(super) bindings: Vec<CommandBindingDeclaration>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CommandBindingDeclaration {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
}

#[derive(Clone, Copy)]
pub struct Command<'a> {
    declaration: &'a CommandDeclaration,
}

#[derive(Clone, Copy)]
pub struct CommandBinding<'a> {
    command: &'a CommandDeclaration,
    pub agent: Agent,
    pub scope: Scope,
}

impl<'a> From<&'a CommandDeclaration> for Command<'a> {
    fn from(declaration: &'a CommandDeclaration) -> Self { Self { declaration } }
}

impl<'a> Command<'a> {
    pub fn name(self) -> &'a str { &self.declaration.name }
    pub fn description(self) -> &'a str { &self.declaration.description }
    pub fn prompt(self) -> &'a str { &self.declaration.prompt }
    pub fn bindings(self) -> impl ExactSizeIterator<Item = CommandBinding<'a>> {
        self.declaration.bindings.iter().map(move |binding| CommandBinding {
            command: self.declaration,
            agent: binding.agent,
            scope: binding.scope,
        })
    }
}

impl<'a> CommandBinding<'a> {
    pub fn name(self) -> &'a str { &self.command.name }
    pub fn description(self) -> &'a str { &self.command.description }
    pub fn prompt(self) -> &'a str { &self.command.prompt }
}
```

Dans `manifest.rs`, déclarer/réexporter le module, ajouter
`#[serde(default)] commands: Vec<commands::CommandDeclaration>` à `Manifest`, puis exposer
`commands()` par `self.commands.iter().map(Command::from)`.

- [ ] **Étape 4: Ajouter les tests RED de validation statique**

Ajouter ces tests table-driven ; les chaînes sont complètes et ne dépendent d'aucune fixture externe :

```rust
#[test]
fn command_fields_are_normalized() {
    for (commands, expected) in [
        ("\n  - name: ''\n    description: Deploy safely\n    prompt: deploy\n    bindings: [{ agent: claude, scope: user }]", "commands[0].name: must be lowercase ASCII kebab-case"),
        ("\n  - name: Deploy\n    description: Deploy safely\n    prompt: deploy\n    bindings: [{ agent: claude, scope: user }]", "commands[0].name: must be lowercase ASCII kebab-case"),
        ("\n  - name: deploy_now\n    description: Deploy safely\n    prompt: deploy\n    bindings: [{ agent: claude, scope: user }]", "commands[0].name: must be lowercase ASCII kebab-case"),
        ("\n  - name: deploy--now\n    description: Deploy safely\n    prompt: deploy\n    bindings: [{ agent: claude, scope: user }]", "commands[0].name: must be lowercase ASCII kebab-case"),
        ("\n  - name: deploy\n    description: '  '\n    prompt: deploy\n    bindings: [{ agent: claude, scope: user }]", "commands[0].description: description cannot be blank"),
        ("\n  - name: deploy\n    description: Deploy safely\n    prompt: missing\n    bindings: [{ agent: claude, scope: user }]", "commands[0].prompt: referenced prompt is not declared"),
        ("\n  - name: deploy\n    description: Deploy safely\n    prompt: deploy\n    bindings: []", "commands[0].bindings: at least one binding is required"),
        ("\n  - name: deploy\n    description: Deploy safely\n    prompt: deploy\n    bindings:\n      - { agent: claude, scope: user }\n      - { agent: claude, scope: user }", "commands[0].bindings[1].agent: duplicates commands[0].bindings[0]"),
        ("\n  - name: deploy\n    description: One\n    prompt: deploy\n    bindings: [{ agent: claude, scope: user }]\n  - name: deploy\n    description: Two\n    prompt: deploy\n    bindings: [{ agent: claude, scope: user }]", "commands[1].bindings[0].agent: duplicates commands[0].bindings[0]"),
    ] {
        assert_eq!(error(commands), expected);
    }
}

#[test]
fn command_targets_must_be_declared() {
    let command = "\n  - name: deploy\n    description: Deploy safely\n    prompt: deploy\n    bindings: [{ agent: cursor, scope: project }]";
    let without_cursor = input(command).replace(
        "  - id: cursor\n    scopes: [project]\n",
        "",
    );
    assert_eq!(
        manifest::parse(&without_cursor).err().unwrap().to_string(),
        "commands[0].bindings[0].agent: agent is not declared"
    );
    let wrong_scope = command.replace("scope: project", "scope: user");
    assert_eq!(
        error(&wrong_scope),
        "commands[0].bindings[0].scope: scope is not declared for this agent"
    );
}
```

Ajouter séparément ces assertions :

```rust
#[test]
fn the_same_name_is_allowed_across_agents_and_scopes() {
    manifest::parse(&input(
        "\n  - name: deploy\n    description: Deploy safely\n    prompt: deploy\n    bindings:\n      - { agent: claude, scope: user }\n      - { agent: claude, scope: project }\n      - { agent: cursor, scope: project }",
    ))
    .unwrap();
}

#[test]
fn legacy_command_resources_are_rejected() {
    let error = manifest::parse(&input(" []").replace(
        "resources: []",
        "resources:\n  - id: deploy\n    kind: commands\n    agent: claude\n    scope: project\n    source: { root: repository, path: prompt.md }\n    destination: { root: repository, path: .claude/commands/deploy.md }",
    ))
    .err()
    .unwrap();
    assert_eq!(error.to_string(), "resources[0].kind: commands must use normalized top-level declarations");
}
```

Ajouter enfin :

```rust
#[test]
fn unknown_command_and_binding_fields_are_rejected() {
    let command = "\n  - name: deploy\n    description: Deploy safely\n    prompt: deploy\n    extra: value\n    bindings: [{ agent: claude, scope: user }]";
    assert!(error(command).contains("unknown field `extra`"));
    let binding = "\n  - name: deploy\n    description: Deploy safely\n    prompt: deploy\n    bindings: [{ agent: claude, scope: user, extra: value }]";
    assert!(error(binding).contains("unknown field `extra`"));
}
```

- [ ] **Étape 5: Implémenter la validation pure minimale**

Créer `manifest/validation/commands.rs` avec :

```rust
use super::super::commands::CommandDeclaration;
use super::super::prompts::PromptDeclaration;
use super::super::{Agent, ManifestError, Scope};
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    commands: &[CommandDeclaration],
    prompts: &[PromptDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let prompts = prompts.iter().map(|prompt| prompt.id.as_str()).collect::<HashSet<_>>();
    let mut identities = HashMap::new();
    for (command_index, command) in commands.iter().enumerate() {
        let field = |name: &str| format!("commands[{command_index}].{name}");
        if !valid_name(&command.name) {
            return Err(ManifestError::new(field("name"), "must be lowercase ASCII kebab-case"));
        }
        if command.description.trim().is_empty() {
            return Err(ManifestError::new(field("description"), "description cannot be blank"));
        }
        if !prompts.contains(command.prompt.as_str()) {
            return Err(ManifestError::new(field("prompt"), "referenced prompt is not declared"));
        }
        if command.bindings.is_empty() {
            return Err(ManifestError::new(field("bindings"), "at least one binding is required"));
        }
        for (binding_index, binding) in command.bindings.iter().enumerate() {
            let binding_field = |name: &str| {
                format!("commands[{command_index}].bindings[{binding_index}].{name}")
            };
            super::validate_target(&binding_field, binding.agent, binding.scope, agents)?;
            let identity = (binding.agent, binding.scope, command.name.as_str());
            if let Some((previous_command, previous_binding)) =
                identities.insert(identity, (command_index, binding_index))
            {
                return Err(ManifestError::new(
                    binding_field("agent"),
                    format!("duplicates commands[{previous_command}].bindings[{previous_binding}]"),
                ));
            }
        }
    }
    Ok(())
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|segment| {
            !segment.is_empty()
                && segment.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}
```

Rendre `PromptDeclaration::id` accessible à ce module avec `pub(super)` déjà présent. Dans
`validation.rs`, déclarer `mod commands`, appeler cette validation après `prompts::validate`, et
refuser `ResourceKind::Commands` au même endroit que `ResourceKind::Prompts`.

- [ ] **Étape 6: Vérifier la tranche et les régressions du manifeste**

```bash
cargo test --test manifest_commands
cargo test --test manifest --test manifest_prompts
```

Attendu : PASS pour les trois binaires de test.

- [ ] **Étape 7: Committer le schéma normalisé**

```bash
git add tooling/arnes/src/manifest.rs tooling/arnes/src/manifest/commands.rs tooling/arnes/src/manifest/validation.rs tooling/arnes/src/manifest/validation/commands.rs tooling/arnes/tests/manifest_commands.rs
git diff --cached --check
git commit -m "feat(arnes): declare command bindings"
```

### Tâche 2: Exposer une validation de projection réutilisable

**Fichiers :**
- Modifier : `tooling/arnes/src/prompts.rs`
- Modifier : `tooling/arnes/src/prompts/projection.rs`

- [ ] **Étape 1: Établir la caractérisation GREEN de #109**

```bash
cd tooling/arnes
cargo test --test prompts --test prompt_failures --test prompt_topology --test prompt_adversarial
```

Attendu : PASS avant le refactor ; conserver la sortie dans le journal d'exécution.

- [ ] **Étape 2: Faire retourner le contenu effectivement comparé sans seconde lecture**

Changer `projection::validate` en `Result<String, Failure>` et terminer par :

```rust
if actual == *expected {
    Ok(actual)
} else {
    Err(stale(projection))
}
```

Dans `prompts.rs`, extraire la logique actuelle dans :

```rust
pub(crate) fn validate_projection(
    roots: &Roots,
    prompt: Prompt<'_>,
    projection: PromptProjection<'_>,
) -> Result<String, Failure> {
    if projection.representation == PromptRepresentation::Symlink {
        return Err(Failure::new(
            State::Unsupported,
            "symlink projections have no stable agent contract",
            "symlink projection unsupported",
        ));
    }
    let expected = source::validate(roots, prompt)?;
    projection::validate(roots, projection, &expected)
}
```

`diagnose_projection` appelle cette fonction et transforme `Ok(_)` en son diagnostic `healthy`
actuel. Rendre `Failure`, ses trois champs et `Failure::new` `pub(crate)`. Réexporter
`pub(crate) use topology::Tracker as ProjectionTracker`; conserver `Tracker::new` et
`Tracker::validate` publics dans le crate uniquement.

- [ ] **Étape 3: Relancer strictement la caractérisation**

```bash
cargo test --test prompts --test prompt_failures --test prompt_topology --test prompt_adversarial
```

Attendu : mêmes tests PASS, mêmes états et messages ; aucun changement fonctionnel du CLI prompts.

- [ ] **Étape 4: Committer le refactor interne isolé**

```bash
git add tooling/arnes/src/prompts.rs tooling/arnes/src/prompts/projection.rs
git diff --cached --check
git commit -m "refactor(arnes): expose prompt projection validation"
```

### Tâche 3: Livrer la tranche verticale Claude et le routage CLI

**Fichiers :**
- Créer : `tooling/arnes/src/commands.rs`
- Créer : `tooling/arnes/src/commands/capability.rs`
- Créer : `tooling/arnes/src/commands/binding.rs`
- Créer : `tooling/arnes/tests/support/commands.rs`
- Créer : `tooling/arnes/tests/commands.rs`
- Modifier : `tooling/arnes/src/lib.rs`
- Modifier : `tooling/arnes/src/main.rs`

- [ ] **Étape 1: Écrire une fixture read-only à deux scopes**

Créer ce support complet dans `tests/support/commands.rs` :

```rust
use crate::support::Fixture;
use std::process::Output;

pub const DESCRIPTION: &str = "Deploy safely";
pub const CONTENTS: &str = "---\ndescription: Deploy safely\n---\nDeploy now\n";

pub fn manifest(prompts: &str, commands: &str) -> String {
    format!(
        "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: cursor
    scopes: [user, project]
  - id: codex
    scopes: [user, project]
prompts:
{prompts}commands:
{commands}resources: []
"
    )
}

pub fn prompt(id: &str, agent: &str, scope: &str, representation: &str, destination: &str) -> String {
    let root = if scope == "user" { "home" } else { "repository" };
    format!(
        "  - id: {id}\n    source: {{ root: repository, path: harness/prompts/{id}.md }}\n    includes: []\n    variables: []\n    projections:\n      - agent: {agent}\n        scope: {scope}\n        representation: {representation}\n        destination: {{ root: {root}, path: {destination} }}\n"
    )
}

pub fn command(name: &str, prompt: &str, bindings: &str) -> String {
    format!(
        "  - name: {name}\n    description: {DESCRIPTION}\n    prompt: {prompt}\n    bindings:\n{bindings}"
    )
}

pub fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    let prompts = "  - id: deploy
    source: { root: repository, path: harness/prompts/deploy.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: rendered
        destination: { root: home, path: .claude/commands/deploy.md }
      - agent: claude
        scope: project
        representation: file
        destination: { root: repository, path: .claude/commands/deploy.md }
";
    let commands = command(
        "deploy",
        "deploy",
        "      - { agent: claude, scope: user }\n      - { agent: claude, scope: project }\n",
    );
    fixture.write_home(".arnes.yaml", &manifest(prompts, &commands));
    fixture.write_repository("harness/prompts/deploy.md", CONTENTS);
    fixture.write_home(".claude/commands/deploy.md", CONTENTS);
    fixture.write_repository(".claude/commands/deploy.md", CONTENTS);
    fixture
}

pub fn run(fixture: &Fixture, args: &[&str]) -> (i32, String, String) {
    let before = fixture.snapshot();
    let output = fixture.command(args);
    assert_eq!(fixture.snapshot(), before);
    output_tuple(output)
}

pub fn output_tuple(output: Output) -> (i32, String, String) {
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
```

- [ ] **Étape 2: Écrire les tests CLI RED de la tranche saine**

Ajouter dans `tests/commands.rs` :

```rust
#[path = "support/commands.rs"]
pub mod command_support;
pub mod support;

use command_support::{configured_fixture, output_tuple, run};
use std::process::Command;

#[test]
fn claude_user_and_project_bindings_are_healthy() {
    for scope in ["user", "project"] {
        let fixture = configured_fixture();
        let (code, stdout, stderr) = run(
            &fixture,
            &["doctor", "commands", "--agent", "claude", "--scope", scope],
        );
        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains(&format!("claude {scope} commands")));
        assert!(stdout.contains("healthy     deploy · current"));
        assert!(stderr.is_empty());
    }
}

#[test]
fn command_diagnostics_are_json_and_read_only() {
    let fixture = configured_fixture();
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "commands", "--agent", "claude", "--scope", "project", "--format", "json"],
    );
    let diagnostics: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();
    assert_eq!(code, 0);
    assert_eq!(diagnostics[0]["resource"], "commands");
    assert_eq!(diagnostics[0]["state"], "healthy");
    assert!(stderr.is_empty());
}
```

Ajouter dans le même fichier :

```rust
#[test]
fn doctor_commands_routes_root_errors_to_commands() {
    let fixture = configured_fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["doctor", "commands"])
        .current_dir(fixture.repository())
        .env_clear()
        .output()
        .unwrap();
    let (code, stdout, stderr) = output_tuple(output);
    assert_eq!(code, 2);
    assert!(stdout.starts_with("error commands:"));
    assert!(stderr.is_empty());
}
```

- [ ] **Étape 3: Exécuter la tranche et observer RED**

```bash
cargo test --test commands
```

Attendu : FAIL, sortie vide pour `doctor commands` et exit code incorrect sans `HOME`.

- [ ] **Étape 4: Implémenter la capacité et le parseur de metadata**

Dans `commands/capability.rs` :

```rust
use crate::manifest::{Agent, Scope};
use std::path::{Path, PathBuf};

pub(super) fn destination(agent: Agent, scope: Scope, name: &str) -> Option<PathBuf> {
    match (agent, scope) {
        (Agent::Claude, Scope::User | Scope::Project) => {
            Some(Path::new(".claude/commands").join(format!("{name}.md")))
        }
        _ => None,
    }
}
```

Dans `commands/binding.rs`, normaliser CRLF vers LF, exiger deux lignes délimitrices `---`, puis
désérialiser sans `deny_unknown_fields` :

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct Metadata {
    description: Option<String>,
}

pub(super) fn validate(contents: &str, expected: &str) -> Result<(), &'static str> {
    let normalized = contents.replace("\r\n", "\n");
    let frontmatter = normalized
        .strip_prefix("---\n")
        .and_then(|contents| contents.split_once("\n---\n").map(|(yaml, _)| yaml))
        .ok_or("frontmatter missing or malformed")?;
    let metadata: Metadata =
        serde_yaml_ng::from_str(frontmatter).map_err(|_| "frontmatter missing or malformed")?;
    match metadata.description.as_deref() {
        Some(description) if description == expected => Ok(()),
        Some(_) => Err("description differs from manifest"),
        None => Err("description is missing"),
    }
}
```

- [ ] **Étape 5: Implémenter l'orchestrateur avec les frontières approuvées**

Dans `commands.rs`, déclarer `mod binding; mod capability;` et exposer :

```rust
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, CommandBinding, Manifest, Prompt, PromptProjection, Scope};
use crate::prompts::{self, Failure};
use std::path::Path;

mod binding;
mod capability;

pub fn diagnose(
    roots: &Roots,
    manifest: &Manifest,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    let selected = manifest
        .commands()
        .flat_map(|command| command.bindings())
        .filter(|binding| agent.is_none_or(|agent| agent == binding.agent))
        .filter(|binding| scope.is_none_or(|scope| scope == binding.scope))
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return vec![unsupported(agent, scope, None)];
    }
    let prompts = manifest.prompts().collect::<Vec<_>>();
    selected
        .into_iter()
        .map(|binding| diagnose_binding(roots, binding, &prompts))
        .collect()
}

fn diagnose_binding(
    roots: &Roots,
    command: CommandBinding<'_>,
    prompts: &[Prompt<'_>],
) -> Diagnostic {
    let Some(expected_destination) =
        capability::destination(command.agent, command.scope, command.name())
    else {
        return unsupported(Some(command.agent), Some(command.scope), Some(command.name()));
    };
    let (prompt, projection) = match resolve_projection(command, prompts, &expected_destination) {
        Ok(binding) => binding,
        Err(diagnostic) => return diagnostic,
    };
    match prompts::validate_projection(roots, prompt, projection) {
        Err(failure) => broken(command, failure),
        Ok(contents) => match binding::validate(&contents, command.description()) {
            Ok(()) => diagnostic(command, State::Healthy, "binding is current", "current"),
            Err(message) => diagnostic(command, State::Drift, message, "binding stale"),
        },
    }
}

fn resolve_projection<'a>(
    command: CommandBinding<'a>,
    prompts: &[Prompt<'a>],
    expected_destination: &Path,
) -> Result<(Prompt<'a>, PromptProjection<'a>), Diagnostic> {
    let Some(prompt) = prompts
        .iter()
        .copied()
        .find(|prompt| prompt.id() == command.prompt())
    else {
        return Err(diagnostic(
            command,
            State::Error,
            format!("referenced prompt {} is not declared", command.prompt()),
            "prompt missing",
        ));
    };
    let Some(projection) = prompt
        .projections()
        .find(|projection| projection.agent == command.agent && projection.scope == command.scope)
    else {
        return Err(diagnostic(
            command,
            State::Error,
            "referenced prompt has no projection for this binding",
            "projection missing",
        ));
    };
    if projection.destination != expected_destination {
        return Err(diagnostic(
            command,
            State::Error,
            format!(
                "prompt projection destination {} does not match {}",
                projection.destination.display(),
                expected_destination.display()
            ),
            "projection destination incompatible",
        ));
    }
    Ok((prompt, projection))
}

fn unsupported(
    agent: Option<Agent>,
    scope: Option<Scope>,
    name: Option<&str>,
) -> Diagnostic {
    let subject = match (agent, scope, name) {
        (Some(agent), Some(scope), Some(name)) => format!("{agent} {scope} command {name}"),
        (Some(agent), Some(scope), None) => format!("{agent} {scope} commands"),
        (Some(agent), None, None) => format!("{agent} commands"),
        (None, Some(scope), None) => format!("{scope} command scope"),
        _ => "command bindings".to_owned(),
    };
    let group = match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope} commands"),
        _ => "commands".to_owned(),
    };
    Diagnostic::new(
        "commands",
        State::Unsupported,
        format!("{subject} is not declared or has no stable command contract"),
    )
    .with_human(group, "capability · unsupported")
}

fn broken(command: CommandBinding<'_>, failure: Failure) -> Diagnostic {
    diagnostic(
        command,
        failure.state,
        failure.message,
        failure.summary,
    )
}

fn diagnostic(
    command: CommandBinding<'_>,
    state: State,
    message: impl Into<String>,
    summary: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        "commands",
        state,
        format!("{}: {}", subject(command), message.into()),
    )
    .with_human(
        format!("{} {} commands", command.agent, command.scope),
        format!("{} · {}", command.name(), summary.into()),
    )
}

fn subject(command: CommandBinding<'_>) -> String {
    let destination = capability::destination(command.agent, command.scope, command.name())
        .expect("supported bindings have a destination");
    format!(
        "managed {} {} command {} at {}",
        command.agent,
        command.scope,
        command.name(),
        destination.display()
    )
}
```

Ce code filtre avant toute I/O, ne résout jamais un prompt pour Cursor ou Codex, et transpose les
échecs de `prompts` sans changer leur état. Le tracker topologique reste volontairement absent afin
que les tests de collision de Tâche 4 établissent leur RED avant son branchement.

- [ ] **Étape 6: Router le module dans la bibliothèque et le binaire**

Ajouter `pub mod commands;` à `lib.rs`. Dans `main.rs`, importer `arnes::commands`, ajouter un bras
`Some(Resource::Commands)` symétrique à `Prompts`, puis :

```rust
fn diagnose_commands(
    roots: &Roots,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => commands::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("commands", State::Error, error.to_string())],
    }
}
```

- [ ] **Étape 7: Vérifier GREEN et les régressions directes**

```bash
cargo test --test commands
cargo test --test prompts --test prompt_failures --test prompt_topology --test prompt_adversarial
```

Attendu : PASS ; `doctor commands` produit un diagnostic par binding sélectionné et ne mute aucune
fixture ; tous les tests prompts restent PASS.

- [ ] **Étape 8: Committer la tranche verticale**

```bash
git add tooling/arnes/src/commands.rs tooling/arnes/src/commands tooling/arnes/src/lib.rs tooling/arnes/src/main.rs tooling/arnes/tests/commands.rs tooling/arnes/tests/support/commands.rs
git diff --cached --check
git commit -m "feat(arnes): diagnose Claude command bindings"
```

### Tâche 4: Fermer les cas limites, filtres et collisions

**Fichiers :**
- Modifier : `tooling/arnes/tests/commands.rs`
- Créer : `tooling/arnes/tests/command_failures.rs`
- Créer : `tooling/arnes/tests/command_topology.rs`
- Modifier : `tooling/arnes/src/commands.rs`
- Modifier : `tooling/arnes/src/commands/binding.rs`

- [ ] **Étape 1: Ajouter la matrice RED des capacités et filtres**

Dans `commands.rs`, tester Cursor user/project et Codex user/project : exit `0`, résumé
`capability · unsupported`, aucune mention d'une source de prompt volontairement absente. Tester une
sélection sans binding, puis un filtre Claude project qui exclut un binding Claude user dont la
source manque. Ajouter deux commandes dans l'ordre `zeta`, `alpha`, chacune avec deux bindings, et
affirmer l'ordre commande puis binding en humain et JSON. Créer aussi
`.claude/commands/unmanaged.md` et `.claude/commands/opsx/plugin.md`, puis affirmer leur absence de la
sortie et l'identité du snapshot.

Remplacer l'import `command_support` de Tâche 3 par celui-ci, ajouter `Fixture`, puis utiliser ces
tests complets :

```rust
use command_support::{
    CONTENTS, command, configured_fixture, manifest, output_tuple, prompt, run,
};
use support::Fixture;

#[test]
fn cursor_and_codex_are_unsupported_without_prompt_io() {
    for (agent, scope) in [
        ("cursor", "user"),
        ("cursor", "project"),
        ("codex", "user"),
        ("codex", "project"),
    ] {
        let fixture = Fixture::new();
        let prompts = prompt(
            "deploy",
            "claude",
            "project",
            "file",
            ".claude/commands/deploy.md",
        );
        let bindings = format!("      - {{ agent: {agent}, scope: {scope} }}\n");
        let commands = command("deploy", "deploy", &bindings);
        fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
        let (code, stdout, stderr) = run(
            &fixture,
            &["doctor", "commands", "--agent", agent, "--scope", scope],
        );
        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains("capability · unsupported"));
        assert!(!stdout.contains("source"));
        assert!(stderr.is_empty());
    }
}

#[test]
fn filters_exclude_bindings_before_io() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt("missing", "claude", "user", "file", ".claude/commands/missing.md"),
        prompt("selected", "claude", "project", "file", ".claude/commands/selected.md"),
    );
    let commands = format!(
        "{}{}",
        command("missing", "missing", "      - { agent: claude, scope: user }\n"),
        command("selected", "selected", "      - { agent: claude, scope: project }\n"),
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
    fixture.write_repository("harness/prompts/selected.md", CONTENTS);
    fixture.write_repository(".claude/commands/selected.md", CONTENTS);
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "commands", "--agent", "claude", "--scope", "project"],
    );
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("selected · current"));
    assert!(!stdout.contains("missing"));
    assert!(stderr.is_empty());
}

#[test]
fn an_empty_filtered_selection_is_unsupported() {
    let fixture = configured_fixture();
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "commands", "--agent", "cursor", "--scope", "project"],
    );
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("capability · unsupported"));
    assert!(stderr.is_empty());
}

#[test]
fn diagnostics_preserve_command_then_binding_order() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt("zeta", "claude", "user", "file", ".claude/commands/zeta.md"),
        prompt("alpha", "claude", "user", "file", ".claude/commands/alpha.md"),
    );
    let bindings = "      - { agent: claude, scope: user }\n      - { agent: cursor, scope: user }\n";
    let commands = format!(
        "{}{}",
        command("zeta", "zeta", bindings),
        command("alpha", "alpha", bindings),
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
    for name in ["zeta", "alpha"] {
        fixture.write_repository(format!("harness/prompts/{name}.md"), CONTENTS);
        fixture.write_home(format!(".claude/commands/{name}.md"), CONTENTS);
    }
    let (_, human, _) = run(&fixture, &["doctor", "commands", "--scope", "user"]);
    let positions = ["zeta · current", "capability · unsupported", "alpha · current"]
        .map(|needle| human.find(needle).unwrap());
    assert!(positions[0] < positions[1] && positions[1] < positions[2]);
    let (_, json, _) = run(
        &fixture,
        &["doctor", "commands", "--scope", "user", "--format", "json"],
    );
    let diagnostics: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    assert!(diagnostics[0]["message"].as_str().unwrap().contains("zeta"));
    assert_eq!(diagnostics[1]["state"], "unsupported");
    assert!(diagnostics[2]["message"].as_str().unwrap().contains("alpha"));
    assert_eq!(diagnostics[3]["state"], "unsupported");
}

#[test]
fn unmanaged_and_plugin_neighbors_are_ignored() {
    let fixture = configured_fixture();
    fixture.write_home(".claude/commands/unmanaged.md", "ignored\n");
    fixture.write_home(".claude/commands/opsx/plugin.md", "ignored\n");
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "commands", "--agent", "claude", "--scope", "user"],
    );
    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("unmanaged"));
    assert!(!stdout.contains("opsx"));
    assert!(stderr.is_empty());
}
```

- [ ] **Étape 2: Ajouter la matrice RED des erreurs et metadata**

Dans `command_failures.rs`, utiliser une fonction `assert_state(fixture, expected_code, expected)`
et couvrir : projection Claude absente ; destination déclarée différente de
`.claude/commands/<name>.md` ; destination absente, répertoire, symlink, périmée ou illisible ; source
absente ou illisible ; include absent ; variable non déclarée ; représentation `symlink` ;
frontmatter absent, non fermé ou YAML invalide ; description absente, non-string ou différente.
Pour les cas metadata, écrire le même contenu altéré dans la source et la projection afin que la
validation du prompt soit saine et que le contrôle du binding soit effectivement atteint.

Commencer `command_failures.rs` avec :

```rust
#[path = "support/commands.rs"]
pub mod command_support;
pub mod support;

use command_support::{
    CONTENTS, command, configured_fixture, manifest, output_tuple, prompt, run,
};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use support::Fixture;

const CLAUDE_PROJECT: &[&str] = &[
    "doctor", "commands", "--agent", "claude", "--scope", "project",
];

fn assert_state(fixture: &Fixture, code: i32, expected: &str) {
    let (actual, stdout, stderr) = run(fixture, CLAUDE_PROJECT);
    assert_eq!(actual, code, "{stdout}");
    assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    assert!(stderr.is_empty());
}

#[test]
fn missing_wrong_type_stale_and_symlink_destinations_are_drift() {
    let fixture = configured_fixture();
    fs::remove_file(fixture.repository().join(".claude/commands/deploy.md")).unwrap();
    assert_state(&fixture, 1, "is missing");

    let fixture = configured_fixture();
    let destination = fixture.repository().join(".claude/commands/deploy.md");
    fs::remove_file(&destination).unwrap();
    fs::create_dir(&destination).unwrap();
    assert_state(&fixture, 1, "is not a regular file");

    let fixture = configured_fixture();
    fixture.write_repository(".claude/commands/deploy.md", "stale\n");
    assert_state(&fixture, 1, "is stale");

    let fixture = configured_fixture();
    let destination = fixture.repository().join(".claude/commands/deploy.md");
    fs::remove_file(&destination).unwrap();
    symlink("../../harness/prompts/deploy.md", &destination).unwrap();
    assert_state(&fixture, 1, "is a symlink");
}

#[test]
fn unreadable_source_and_destination_are_errors() {
    for relative in ["harness/prompts/deploy.md", ".claude/commands/deploy.md"] {
        let fixture = configured_fixture();
        let path = fixture.repository().join(relative);
        let permissions = fs::metadata(&path).unwrap().permissions();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        let output = fixture.command(CLAUDE_PROJECT);
        fs::set_permissions(&path, permissions).unwrap();
        let (code, stdout, stderr) = output_tuple(output);
        assert_eq!(code, 2, "{stdout}");
        assert!(stdout.contains("could not be read"));
        assert!(stderr.is_empty());
    }
}

#[test]
fn projection_contract_failures_are_explicit() {
    let fixture = Fixture::new();
    let no_projection = prompt("deploy", "cursor", "project", "file", ".cursor/commands/deploy.md");
    let commands = command("deploy", "deploy", "      - { agent: claude, scope: project }\n");
    fixture.write_home(".arnes.yaml", &manifest(&no_projection, &commands));
    assert_state(&fixture, 2, "projection missing");

    let fixture = Fixture::new();
    let incompatible = prompt(
        "deploy",
        "claude",
        "project",
        "file",
        ".claude/commands/not-deploy.md",
    );
    fixture.write_home(".arnes.yaml", &manifest(&incompatible, &commands));
    assert_state(&fixture, 2, "projection destination incompatible");

    let fixture = Fixture::new();
    let symlinked = prompt(
        "deploy",
        "claude",
        "project",
        "symlink",
        ".claude/commands/deploy.md",
    );
    fixture.write_home(".arnes.yaml", &manifest(&symlinked, &commands));
    assert_state(&fixture, 0, "symlink projection unsupported");
}

#[test]
fn source_include_and_variable_failures_are_transposed() {
    let fixture = configured_fixture();
    fs::remove_file(fixture.repository().join("harness/prompts/deploy.md")).unwrap();
    assert_state(&fixture, 2, "source harness/prompts/deploy.md is missing");

    let fixture = Fixture::new();
    let prompts = "  - id: deploy
    source: { root: repository, path: harness/prompts/deploy.md }
    includes: [missing.md]
    variables: []
    projections:
      - agent: claude
        scope: project
        representation: file
        destination: { root: repository, path: .claude/commands/deploy.md }
";
    let commands = command("deploy", "deploy", "      - { agent: claude, scope: project }\n");
    fixture.write_home(".arnes.yaml", &manifest(prompts, &commands));
    fixture.write_repository("harness/prompts/deploy.md", "@missing.md\nDeploy\n");
    fixture.write_repository(".claude/commands/deploy.md", "@missing.md\nDeploy\n");
    assert_state(&fixture, 2, "include harness/prompts/missing.md is missing");

    let fixture = configured_fixture();
    let contents = format!("{CONTENTS}Deploy $undeclared\n");
    fixture.write_repository("harness/prompts/deploy.md", &contents);
    fixture.write_repository(".claude/commands/deploy.md", &contents);
    assert_state(&fixture, 2, "variables undeclared are referenced but not declared");
}

#[test]
fn frontmatter_and_description_mismatches_are_drift() {
    for (contents, expected) in [
        ("Deploy now\n", "frontmatter missing or malformed"),
        ("---\ndescription: Deploy safely\nDeploy now\n", "frontmatter missing or malformed"),
        ("---\ndescription: [\n---\nDeploy now\n", "frontmatter missing or malformed"),
        ("---\nother: value\n---\nDeploy now\n", "description is missing"),
        ("---\ndescription: 42\n---\nDeploy now\n", "frontmatter missing or malformed"),
        ("---\ndescription: Deploy unsafely\n---\nDeploy now\n", "description differs from manifest"),
    ] {
        let fixture = configured_fixture();
        fixture.write_repository("harness/prompts/deploy.md", contents);
        fixture.write_repository(".claude/commands/deploy.md", contents);
        assert_state(&fixture, 1, expected);
    }
}

#[test]
fn unrelated_frontmatter_keys_are_ignored() {
    let fixture = configured_fixture();
    let contents = "---\ndescription: Deploy safely\nallowed-tools: Bash\n---\nDeploy now\n";
    fixture.write_repository("harness/prompts/deploy.md", contents);
    fixture.write_repository(".claude/commands/deploy.md", contents);
    assert_state(&fixture, 0, "deploy · current");
}
```

Ajouter ce test de priorité :

```rust
#[test]
fn the_highest_state_controls_the_exit_code() {
    let fixture = Fixture::new();
    let prompts = "  - id: cursor-command
    source: { root: repository, path: harness/prompts/cursor-command.md }
    includes: []
    variables: []
    projections: []
  - id: drifting
    source: { root: repository, path: harness/prompts/drifting.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: file
        destination: { root: home, path: .claude/commands/drifting.md }
  - id: broken
    source: { root: repository, path: harness/prompts/broken.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: file
        destination: { root: home, path: .claude/commands/broken.md }
";
    let commands = "  - name: cursor-command
    description: Deploy safely
    prompt: cursor-command
    bindings: [{ agent: cursor, scope: user }]
  - name: drifting
    description: Deploy safely
    prompt: drifting
    bindings: [{ agent: claude, scope: user }]
  - name: broken
    description: Deploy safely
    prompt: broken
    bindings: [{ agent: claude, scope: user }]
";
    fixture.write_home(".arnes.yaml", &manifest(prompts, commands));
    fixture.write_repository("harness/prompts/drifting.md", CONTENTS);
    let (code, stdout, stderr) = run(&fixture, &["doctor", "commands"]);
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("unsupported"));
    assert!(stdout.contains("drift"));
    assert!(stdout.contains("error"));
    assert!(stderr.is_empty());
}
```

- [ ] **Étape 3: Ajouter les collisions topologiques RED**

Dans `command_topology.rs`, créer deux prompts et bindings Claude project dont les destinations
lexicales diffèrent mais dont les parents symlinkent le même répertoire réel ; attendre exit `2` et
`aliases managed destination`. Ajouter un cas où la destination du binding alias une destination de
resource préchargée dans le tracker. Les deux bindings en collision doivent être sélectionnés ; un
binding filtré ne doit jamais être lu ni enregistré par le tracker.

Créer le fichier avec cet en-tête et ces tests :

```rust
#[path = "support/commands.rs"]
pub mod command_support;
pub mod support;

use command_support::{CONTENTS, command, manifest, prompt, run};
use std::os::unix::fs::symlink;
use support::Fixture;

#[test]
fn selected_command_destinations_cannot_alias_each_other() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt("one", "claude", "project", "file", ".claude/commands/one.md"),
        prompt("two", "claude", "project", "file", ".claude/commands/two.md"),
    );
    let commands = format!(
        "{}{}",
        command("one", "one", "      - { agent: claude, scope: project }\n"),
        command("two", "two", "      - { agent: claude, scope: project }\n"),
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
    for name in ["one", "two"] {
        fixture.write_repository(format!("harness/prompts/{name}.md"), CONTENTS);
    }
    fixture.write_repository(".claude/commands/shared.md", CONTENTS);
    symlink("shared.md", fixture.repository().join(".claude/commands/one.md")).unwrap();
    symlink("shared.md", fixture.repository().join(".claude/commands/two.md")).unwrap();
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "commands", "--agent", "claude", "--scope", "project"],
    );
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("aliases managed destination"));
    assert!(stderr.is_empty());
}

#[test]
fn command_destinations_cannot_alias_managed_resources() {
    let fixture = Fixture::new();
    let prompts = prompt(
        "deploy",
        "claude",
        "project",
        "file",
        ".claude/commands/deploy.md",
    );
    let commands = command("deploy", "deploy", "      - { agent: claude, scope: project }\n");
    let manifest = manifest(&prompts, &commands).replace(
        "resources: []",
        "resources:\n  - id: managed\n    kind: instructions\n    agent: claude\n    scope: project\n    source: { root: repository, path: harness/AGENTS.md }\n    destination: { root: repository, path: .claude/commands/resource.md }",
    );
    fixture.write_home(".arnes.yaml", &manifest);
    fixture.write_repository("harness/prompts/deploy.md", CONTENTS);
    fixture.write_repository(".claude/commands/shared.md", CONTENTS);
    symlink("shared.md", fixture.repository().join(".claude/commands/deploy.md")).unwrap();
    symlink("shared.md", fixture.repository().join(".claude/commands/resource.md")).unwrap();
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "commands", "--agent", "claude", "--scope", "project"],
    );
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("aliases managed destination resource"));
    assert!(stderr.is_empty());
}
```

Le test `filters_exclude_bindings_before_io` de `commands.rs` couvre le troisième invariant : le
tracker est créé après filtrage et ne voit jamais le binding exclu.

- [ ] **Étape 4: Exécuter les nouveaux tests et observer les échecs précis**

```bash
cargo test --test commands --test command_failures --test command_topology
```

Attendu : les cas fonctionnels sont déjà GREEN grâce à la tranche verticale ; les deux collisions
topologiques sont RED parce que le tracker n'est pas encore branché. Aucune panique ni lecture du
vrai `HOME`.

- [ ] **Étape 5: Brancher le tracker uniquement sur les bindings sélectionnés**

Importer `ProjectionTracker`, le créer après le filtre, le passer à `diagnose_binding`, puis ajouter
ce bloc immédiatement avant `prompts::validate_projection` :

```rust
if let Err(failure) = topology.validate(roots, prompt, projection) {
    return broken(command, failure);
}
```

La signature devient :

```rust
fn diagnose_binding(
    roots: &Roots,
    command: CommandBinding<'_>,
    prompts: &[Prompt<'_>],
    topology: &mut ProjectionTracker,
) -> Diagnostic
```

Dans `diagnose`, utiliser exactement :

```rust
let mut topology = ProjectionTracker::new(roots, manifest);
selected
    .into_iter()
    .map(|binding| diagnose_binding(roots, binding, &prompts, &mut topology))
    .collect()
```

Les helpers `unsupported`, `broken`, `diagnostic` et `subject` de Tâche 3 restent inchangés : ils
préservent déjà les états prompt, classent les erreurs du parseur de binding en `Drift` et évitent
toute I/O pour une capacité absente. Ne pas ajouter scan, retry, fallback ou dépendance.

- [ ] **Étape 6: Faire passer chaque matrice puis toutes les régressions Arnes**

```bash
cargo test --test commands --test command_failures --test command_topology
cargo test --test manifest_commands --test manifest --test manifest_prompts
cargo test --test prompts --test prompt_failures --test prompt_topology --test prompt_adversarial
```

Attendu : PASS pour tous les binaires listés ; les snapshots avant/après restent identiques.

- [ ] **Étape 7: Committer les cas limites**

```bash
git add tooling/arnes/src/commands.rs tooling/arnes/src/commands/binding.rs tooling/arnes/tests/commands.rs tooling/arnes/tests/command_failures.rs tooling/arnes/tests/command_topology.rs
git diff --cached --check
git commit -m "test(arnes): cover command binding failures"
```

### Tâche 5: Exécuter la barrière finale et auditer la cohésion

**Fichiers :**
- Modifier uniquement les fichiers nommés par un échec reproductible du formateur, de Clippy ou des
  tests causé par les tâches 1 à 4.

- [ ] **Étape 1: Formater puis vérifier le format**

```bash
cd tooling/arnes
cargo fmt
cargo fmt --check
```

Attendu : PASS sur macOS ; inspecter le diff du formateur avant de poursuivre.

- [ ] **Étape 2: Exécuter lint et tests complets dans l'environnement local**

```bash
cargo clippy --all-targets -- -D warnings
cargo test
```

Attendu : PASS sur macOS Darwin arm64. Cette preuve ne couvre pas Ubuntu.

- [ ] **Étape 3: Vérifier les extensions et la taille des unités réellement touchées**

```bash
git diff --check origin/main...HEAD
find src tests -type f -name '*.rs' -print0 | xargs -0 wc -l | sort -n
git diff --name-only origin/main...HEAD
```

Attendu : aucun whitespace error ; chaque nouveau fichier manuscrit reste sous 250 lignes. Inspecter
chaque fonction modifiée ; toute fonction dépassant 50 lignes doit être découpée par responsabilité
ou justifiée explicitement dans la livraison.

- [ ] **Étape 4: Vérifier la CI Ubuntu seulement après publication autorisée**

Le workflow `.github/workflows/test-arnes.yml` exécute la barrière Rust sur Ubuntu. Après push ou PR
explicitement autorisé, attendre son résultat avec `gh pr checks --watch` et ne déclarer Ubuntu vert
que si ce run précis réussit. Sans publication, noter Ubuntu « non exercé ».

- [ ] **Étape 5: Committer uniquement les corrections mécaniques finales si nécessaire**

```bash
git status --short
git diff --check
git add tooling/arnes
git commit -m "chore(arnes): satisfy command validation gates"
```

Attendu : créer ce commit seulement si `cargo fmt` ou Clippy a produit une correction ; sinon garder
les quatre commits précédents sans commit vide.

## Critères de livraison

- `doctor commands` ne tombe plus dans le fallback vide et respecte les filtres avant I/O.
- Les commandes logiques évitent la duplication multi-agent et restent uniques par
  `(agent, scope, name)`.
- Claude réutilise exactement le snapshot de projection validé par `prompts`; aucune seconde lecture
  ni duplication de validation du corps.
- Cursor et Codex valident le manifeste puis retournent `unsupported` sans inspecter le disque.
- Aucun répertoire unmanaged ou plugin n'est parcouru, aucune mutation ni dépendance n'est ajoutée.
- Les preuves locales nomment macOS ; Ubuntu n'est revendiqué qu'après le workflow réel.
- Aucun commentaire de code n'est prévu.
