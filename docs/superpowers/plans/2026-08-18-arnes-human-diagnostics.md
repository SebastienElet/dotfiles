# Arnes Human Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rendre toutes les sorties humaines de `arnes doctor` concises par défaut, exhaustives avec `-v`, et regrouper `doctor skills` par agent sans modifier les diagnostics canoniques.

**Architecture:** La collecte continue de produire un `Report` ordonné servant au JSON et aux exit codes. Un renderer humain séparé reçoit un contexte et une verbosité, travaille sur des références aux diagnostics, puis utilise des métadonnées humaines structurées pour les sections et détails skills sans parser `message`.

**Tech Stack:** Rust 2024, Clap 4.5, Serde, tests d’intégration Cargo, fixtures texte.

---

## Structure des fichiers

**Créer :**

- `tooling/arnes/src/diagnostic/human.rs` — projection humaine, compteurs, filtres et tri stable ;
- `tooling/arnes/tests/human_cli.rs` — contrat `-v`, aide et refus du couple verbose/JSON ;
- `tooling/arnes/tests/human_skills.rs` — snapshots multi-agent normal et verbose ;
- `tooling/arnes/tests/fixtures/diagnostic/report-verbose.txt` — vue générique exhaustive ;
- `tooling/arnes/tests/fixtures/skills/report.txt` — vue skills normale exacte ;
- `tooling/arnes/tests/fixtures/skills/report-verbose.txt` — vue skills verbose exacte.

**Modifier :**

- `tooling/arnes/src/main.rs` — parse, validation et contexte de rendu ;
- `tooling/arnes/src/diagnostic.rs` — modèle canonique inchangé, métadonnées humaines et délégation ;
- `tooling/arnes/src/skills.rs` — attachement centralisé des sections agent/scope ;
- `tooling/arnes/src/skills/projection.rs` — détails structurés des écarts de projection ;
- `tooling/arnes/tests/diagnostic.rs` et `tests/fixtures/diagnostic/report.txt` — contrat générique ;
- les tests d’intégration qui inspectent des lignes saines — demander `-v` lorsque leur objet exige l’inventaire ;
- `docs/arnes-capacites-externes.md` — documenter vue normale, verbose et JSON.

`diagnostic.rs`, `main.rs` et chaque fonction de production restent sous les seuils de 250 et 50
lignes. Les nouveaux tests ne sont pas ajoutés à `tests/cli.rs` ou `tests/commands.rs`, déjà proches
de 250 lignes. Le plan lui-même dépasse ce seuil parce qu'il est une checklist séquentielle
exécutable ; le fractionner rendrait les signatures et checkpoints interdépendants ambigus.

### Task 1: Établir le contrat CLI de verbosité

**Files:**

- Create: `tooling/arnes/tests/human_cli.rs`
- Modify: `tooling/arnes/src/main.rs:20-77`

- [ ] **Step 1: Écrire les tests CLI rouges**

Créer `tooling/arnes/tests/human_cli.rs` avec le contrat de parsing et la validation avant `HOME` :

```rust
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(args)
        .env_clear()
        .output()
        .unwrap()
}

#[test]
fn doctor_help_lists_verbose_options() {
    let output = run(&["doctor", "--help"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("-v, --verbose"), "{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn verbose_json_is_rejected_before_home_is_read() {
    for args in [
        vec!["doctor", "skills", "-v", "--format", "json"],
        vec!["doctor", "--verbose", "skills", "--format=json"],
        vec!["doctor", "--format", "json", "skills", "-v"],
    ] {
        let output = run(&args);

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "--verbose cannot be used with --format json\n",
            "{args:?}"
        );
    }
}

#[test]
fn duplicate_format_is_rejected_by_clap() {
    let output = run(&[
        "doctor", "skills", "--format", "human", "--format", "json",
    ]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("cannot be used multiple times"), "{stderr}");
}
```

- [ ] **Step 2: Exécuter le test ciblé et constater le rouge**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test human_cli
```

Expected: FAIL ; l'aide ne contient pas `-v, --verbose` et les invocations sont refusées par Clap
comme arguments inconnus.

- [ ] **Step 3: Ajouter l'option typée et le guard format/verbosité**

Dans `tooling/arnes/src/main.rs`, compléter les dérivations et `Command::Doctor`, puis valider les
valeurs typées avant `diagnose` :

```rust
#[derive(Subcommand)]
enum Command {
    Doctor {
        #[arg(value_enum)]
        resource: Option<Resource>,
        #[arg(long, value_enum)]
        agent: Option<Agent>,
        #[arg(long, value_enum, default_value = "user")]
        scope: Option<Scope>,
        #[arg(long, value_enum, default_value_t)]
        format: Format,
        #[arg(short, long)]
        verbose: bool,
    },
}

#[derive(Clone, Copy, Default, Eq, PartialEq, ValueEnum)]
enum Format {
    #[default]
    Human,
    Json,
}

fn validate_render_options(format: Format, verbose: bool) -> Result<(), &'static str> {
    if verbose && format == Format::Json {
        return Err("--verbose cannot be used with --format json");
    }
    Ok(())
}
```

Conserver `#[default]` sur `Format::Human`. Après la destructuration de `Command::Doctor`, insérer
avant la première ligne qui appelle `diagnose` :

```rust
if let Err(error) = validate_render_options(format, verbose) {
    eprintln!("{error}");
    return ExitCode::from(2);
}
```

Ne pas passer `verbose` à `diagnose` et ne modifier ni `Report::json()` ni `Report::exit_code()`.

- [ ] **Step 4: Exécuter le test CLI ciblé et constater le vert**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test human_cli
```

Expected: PASS, 3 tests.

- [ ] **Step 5: Vérifier les bypasses du guard**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test human_cli verbose_json_is_rejected_before_home_is_read
```

Expected: PASS pour forme courte, forme longue, `--format=json` et ordre inversé. Le sink protégé est
le couple typé `(Format, bool)`, jamais la chaîne argv.

- [ ] **Step 6: Committer le contrat CLI**

```bash
git add tooling/arnes/src/main.rs tooling/arnes/tests/human_cli.rs
git commit -m "feat(arnes): add doctor verbosity contract" -m "Validate the parsed format and verbosity pair before diagnostics. Covered bypass forms: short and long flags, format equals syntax, and reversed argument order."
```

### Task 2: Extraire le renderer humain générique

**Files:**

- Create: `tooling/arnes/src/diagnostic/human.rs`
- Create: `tooling/arnes/tests/fixtures/diagnostic/report-verbose.txt`
- Modify: `tooling/arnes/src/diagnostic.rs:1-127`
- Modify: `tooling/arnes/src/main.rs:35-77`
- Modify: `tooling/arnes/tests/diagnostic.rs:1-91`
- Modify: `tooling/arnes/tests/fixtures/diagnostic/report.txt`

- [ ] **Step 1: Remplacer les tests de rendu générique par les contrats normal et verbose**

Dans `tooling/arnes/tests/diagnostic.rs`, importer les options et le contexte puis remplacer le test
humain exact par ces tests :

```rust
use arnes::diagnostic::{Diagnostic, HumanContext, HumanOptions, Report, State};

fn context() -> HumanContext {
    HumanContext::new("Diagnostics")
}

#[test]
fn normal_human_output_hides_healthy_details() {
    assert_eq!(
        format!("{}\n", report().human(&context(), HumanOptions::normal())),
        include_str!("fixtures/diagnostic/report.txt")
    );
}

#[test]
fn verbose_human_output_includes_healthy_details() {
    assert_eq!(
        format!("{}\n", report().human(&context(), HumanOptions::verbose())),
        include_str!("fixtures/diagnostic/report-verbose.txt")
    );
}

#[test]
fn empty_human_report_does_not_claim_health() {
    let report = Report::new(Vec::new());

    assert_eq!(
        report.human(&context(), HumanOptions::normal()),
        "No diagnostics"
    );
}

#[test]
fn report_without_healthy_diagnostics_displays_zero() {
    let report = Report::new(vec![Diagnostic::new(
        "skills",
        State::Unsupported,
        "inventory unavailable",
    )]);

    assert!(
        report
            .human(&context(), HumanOptions::normal())
            .starts_with("Diagnostics\n✓ 0 healthy\n")
    );
}
```

Conserver sans modification les tests JSON, ordre canonique, échappement et exit codes. Adapter le
test des groupes pour appeler `HumanOptions::verbose()` afin qu'il continue de vérifier les deux
lignes.

- [ ] **Step 2: Écrire les deux fixtures exactes**

Remplacer `tooling/arnes/tests/fixtures/diagnostic/report.txt` par :

```text
Diagnostics
✓ 1 healthy
! 1 unsupported (non-blocking)

unsupported rules: cursor does not expose native user rules
drift skills: destination is missing
error config: settings.json could not be read
```

Créer `tooling/arnes/tests/fixtures/diagnostic/report-verbose.txt` :

```text
Diagnostics
✓ 1 healthy
! 1 unsupported (non-blocking)

healthy manifest: manifest is valid
unsupported rules: cursor does not expose native user rules
drift skills: destination is missing
error config: settings.json could not be read
```

- [ ] **Step 3: Exécuter les tests de diagnostic et constater le rouge de compilation**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test diagnostic
```

Expected: FAIL ; `HumanContext`, `HumanOptions` et la nouvelle signature de `Report::human` n'existent
pas.

- [ ] **Step 4: Définir l'API humaine sans modifier le modèle sérialisé**

Dans `tooling/arnes/src/diagnostic.rs`, ajouter `mod human;`, réexporter les options, puis déléguer :

```rust
mod human;

pub use human::{HumanContext, HumanOptions};

impl Report {
    pub fn human(&self, context: &HumanContext, options: HumanOptions) -> String {
        human::render(&self.diagnostics, context, options)
    }
}
```

Supprimer uniquement l'ancien corps de `Report::human`. Ne toucher ni aux trois champs publics de
`Diagnostic`, ni à `#[serde(skip)]`, ni à `Report::json`, ni à `Report::exit_code`.

- [ ] **Step 5: Implémenter le renderer plat minimal**

Créer `tooling/arnes/src/diagnostic/human.rs` avec des fonctions courtes et sans dépendance :

```rust
use super::{Diagnostic, State};

#[derive(Clone, Debug, Eq, PartialEq)]
struct SectionCount {
    singular: &'static str,
    plural: &'static str,
    empty: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanContext {
    parts: Vec<String>,
    section_count: Option<SectionCount>,
}

impl HumanContext {
    pub fn new(heading: impl Into<String>) -> Self {
        Self {
            parts: vec![heading.into()],
            section_count: None,
        }
    }

    pub fn with_qualifier(mut self, qualifier: impl Into<String>) -> Self {
        self.parts.push(qualifier.into());
        self
    }

    pub fn with_section_count(
        mut self,
        singular: &'static str,
        plural: &'static str,
        empty: &'static str,
    ) -> Self {
        self.section_count = Some(SectionCount {
            singular,
            plural,
            empty,
        });
        self
    }

    pub(super) fn heading(&self, count: Option<usize>) -> String {
        let mut parts = self.parts.clone();
        if let Some(labels) = &self.section_count {
            parts.push(match count {
                Some(1) => format!("1 {}", labels.singular),
                Some(count) => format!("{count} {}", labels.plural),
                None => labels.empty.to_owned(),
            });
        }
        parts.join(" · ")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HumanOptions {
    verbose: bool,
}

impl HumanOptions {
    pub fn normal() -> Self {
        Self { verbose: false }
    }

    pub fn verbose() -> Self {
        Self { verbose: true }
    }

    pub fn includes_healthy(self) -> bool {
        self.verbose
    }
}

pub(super) fn render(
    diagnostics: &[Diagnostic],
    context: &HumanContext,
    options: HumanOptions,
) -> String {
    if diagnostics.is_empty() {
        return "No diagnostics".to_owned();
    }
    let mut lines = vec![
        context.heading(None),
        format!("✓ {} healthy", state_count(diagnostics, State::Healthy)),
    ];
    let unsupported = state_count(diagnostics, State::Unsupported);
    if unsupported > 0 {
        lines.push(format!("! {unsupported} unsupported (non-blocking)"));
    }
    lines.push(String::new());
    lines.extend(render_flat(diagnostics, options));
    trim_trailing_empty(&mut lines);
    lines.join("\n")
}

fn state_count(diagnostics: &[Diagnostic], state: State) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.state == state)
        .count()
}

fn render_flat(diagnostics: &[Diagnostic], options: HumanOptions) -> Vec<String> {
    let mut lines = Vec::new();
    let mut group = None;
    for diagnostic in diagnostics
        .iter()
        .filter(|diagnostic| options.includes_healthy() || diagnostic.state != State::Healthy)
    {
        if let Some(human) = diagnostic.human() {
            if group != Some(human.group()) {
                if !lines.is_empty() {
                    lines.push(String::new());
                }
                lines.push(human.group().to_owned());
                group = Some(human.group());
            }
            lines.push(format!("  {:11} {}", diagnostic.state, human.summary()));
        } else {
            group = None;
            lines.push(diagnostic.to_string());
        }
    }
    lines
}

fn trim_trailing_empty(lines: &mut Vec<String>) {
    while lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
}
```

Ajouter sur `HumanDiagnostic` des accesseurs privés de crate `group()` et `summary()`, et sur
`Diagnostic` l'accesseur `pub(super) fn human(&self) -> Option<&HumanDiagnostic>`. Garder
`human_field` pour l'échappement des sorties plates.

- [ ] **Step 6: Relier `main` au contexte et à la verbosité**

Dériver `Copy, Eq, PartialEq` sur `Resource`, ajouter son libellé, puis construire un contexte sans
parser de diagnostic :

```rust
impl Resource {
    fn heading(self) -> &'static str {
        match self {
            Self::Manifest => "Manifest",
            Self::Config => "Config",
            Self::Instructions => "Instructions",
            Self::Skills => "Skills",
            Self::Prompts => "Prompts",
            Self::Commands => "Commands",
            Self::Rules => "Rules",
            Self::Hooks => "Hooks",
            Self::Mcp => "MCP",
            Self::Statusline => "Statusline",
        }
    }
}

fn human_context(
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> HumanContext {
    let resource = resource.unwrap_or(Resource::Manifest);
    let mut context = HumanContext::new(resource.heading());
    if resource != Resource::Manifest {
        if let Some(scope) = scope {
            context = context.with_qualifier(format!("{scope} scope"));
        }
        if let Some(agent) = agent {
            context = context.with_qualifier(format!("{agent} agent"));
        } else if resource == Resource::Skills {
            context = context.with_section_count("agent", "agents", "all agents");
        }
    }
    context
}
```

Remplacer le choix de format par :

```rust
let output = match format {
    Format::Human => report.human(
        &human_context(resource, agent, scope),
        if verbose {
            HumanOptions::verbose()
        } else {
            HumanOptions::normal()
        },
    ),
    Format::Json => report.json().expect("diagnostics are JSON serializable"),
};
```

Passer `resource` copiable à `diagnose` sans changer sa signature ni le moment de collecte.

- [ ] **Step 7: Exécuter les tests génériques ciblés**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test diagnostic --test human_cli
```

Expected: PASS ; le JSON exact de `report.json` reste identique à `report.json` existant.

- [ ] **Step 8: Committer le renderer générique**

```bash
git add tooling/arnes/src/main.rs tooling/arnes/src/diagnostic.rs tooling/arnes/src/diagnostic/human.rs tooling/arnes/tests/diagnostic.rs tooling/arnes/tests/fixtures/diagnostic/report.txt tooling/arnes/tests/fixtures/diagnostic/report-verbose.txt
git commit -m "feat(arnes): summarize healthy diagnostics"
```

### Task 3: Structurer la vue humaine skills par agent

**Files:**

- Create: `tooling/arnes/tests/human_skills.rs`
- Create: `tooling/arnes/tests/fixtures/skills/report.txt`
- Create: `tooling/arnes/tests/fixtures/skills/report-verbose.txt`
- Modify: `tooling/arnes/src/diagnostic.rs`
- Modify: `tooling/arnes/src/diagnostic/human.rs`
- Modify: `tooling/arnes/src/skills.rs:12-55`
- Modify: `tooling/arnes/src/skills/projection.rs:10-166`
- Modify: `tooling/arnes/src/main.rs`

- [ ] **Step 1: Écrire le test exact multi-agent normal et verbose**

Créer `tooling/arnes/tests/human_skills.rs` avec un rapport entièrement déterministe :

```rust
use arnes::diagnostic::{
    Diagnostic, HumanContext, HumanDetail, HumanOptions, HumanSection, Report, State,
};

fn section(key: &str, label: &str) -> HumanSection {
    HumanSection::new(key, label)
}

fn diagnostic(
    section: HumanSection,
    state: State,
    message: &str,
    summary: &str,
) -> Diagnostic {
    Diagnostic::new("skills", state, message)
        .with_human_summary(summary)
        .with_human_section(section)
}

fn report() -> Report {
    let claude = section("claude:user", "CLAUDE");
    let cursor = section("cursor:user", "CURSOR");
    let codex = section("codex:user", "CODEX");
    Report::new(vec![
        diagnostic(claude.clone(), State::Healthy, "handoff current", "handoff"),
        diagnostic(
            claude,
            State::Unsupported,
            "system skills inventory is unsupported",
            "system skills inventory",
        ),
        diagnostic(
            cursor.clone(),
            State::Healthy,
            "pr-verdict current",
            "pr-verdict",
        ),
        diagnostic(
            cursor.clone(),
            State::Unsupported,
            "extension skill exposure unavailable",
            "extension skill exposure",
        ),
        diagnostic(
            cursor,
            State::Drift,
            "destination is missing",
            "enforcement-code",
        )
        .with_human_details([
            HumanDetail::new("expected", "managed skill present"),
            HumanDetail::new("actual", "destination missing"),
            HumanDetail::new("path", "~/.cursor/skills/enforcement-code"),
        ]),
        diagnostic(
            codex,
            State::Unsupported,
            "browser plugin version unavailable",
            "browser plugin version/cache",
        ),
    ])
}

fn context() -> HumanContext {
    HumanContext::new("Skills")
        .with_qualifier("user scope")
        .with_section_count("agent", "agents", "all agents")
}

#[test]
fn normal_skills_output_matches_the_exact_fixture() {
    assert_eq!(
        format!("{}\n", report().human(&context(), HumanOptions::normal())),
        include_str!("fixtures/skills/report.txt")
    );
}

#[test]
fn verbose_skills_output_matches_the_exact_fixture() {
    assert_eq!(
        format!("{}\n", report().human(&context(), HumanOptions::verbose())),
        include_str!("fixtures/skills/report-verbose.txt")
    );
}
```

Créer `tooling/arnes/tests/fixtures/skills/report.txt` avec ce contenu exact :

```text
Skills · user scope · 3 agents
✓ 2 healthy
! 3 unsupported (non-blocking)

CURSOR
  1 issue · 1 unsupported · 1 healthy

  DRIFT enforcement-code
    expected  managed skill present
    actual    destination missing
    path      ~/.cursor/skills/enforcement-code
  UNSUPPORTED extension skill exposure

CLAUDE
  1 unsupported · 1 healthy

  UNSUPPORTED system skills inventory

CODEX
  1 unsupported · 0 healthy

  UNSUPPORTED browser plugin version/cache
```

Créer `tooling/arnes/tests/fixtures/skills/report-verbose.txt` avec ce contenu exact :

```text
Skills · user scope · 3 agents
✓ 2 healthy
! 3 unsupported (non-blocking)

CURSOR
  1 issue · 1 unsupported · 1 healthy

  DRIFT enforcement-code
    expected  managed skill present
    actual    destination missing
    path      ~/.cursor/skills/enforcement-code
  UNSUPPORTED extension skill exposure
  HEALTHY pr-verdict

CLAUDE
  1 unsupported · 1 healthy

  UNSUPPORTED system skills inventory
  HEALTHY handoff

CODEX
  1 unsupported · 0 healthy

  UNSUPPORTED browser plugin version/cache
```

- [ ] **Step 2: Écrire les assertions unitaires de tri stable et de section saine masquée**

Dans `tooling/arnes/tests/diagnostic.rs`, ajouter un helper et deux tests :

```rust
fn section(key: &str, label: &str) -> HumanSection {
    HumanSection::new(key, label)
}

#[test]
fn structured_sections_sort_by_severity_without_mutating_report_order() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Unsupported, "claude limitation")
            .with_human_section(section("claude:user", "CLAUDE")),
        Diagnostic::new("skills", State::Healthy, "cursor current")
            .with_human_section(section("cursor:user", "CURSOR")),
        Diagnostic::new("skills", State::Drift, "cursor missing")
            .with_human_section(section("cursor:user", "CURSOR")),
    ]);

    let output = report.human(&context(), HumanOptions::normal());

    assert!(output.find("CURSOR").unwrap() < output.find("CLAUDE").unwrap());
    assert!(!output.contains("cursor current"));
    assert_eq!(
        report
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        ["claude limitation", "cursor current", "cursor missing"]
    );
}

#[test]
fn verbose_places_healthy_diagnostics_after_other_states() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Healthy, "current")
            .with_human_section(section("cursor:user", "CURSOR")),
        Diagnostic::new("skills", State::Unsupported, "inventory unavailable")
            .with_human_section(section("cursor:user", "CURSOR")),
    ]);

    let output = report.human(&context(), HumanOptions::verbose());

    assert!(output.find("inventory unavailable").unwrap() < output.find("current").unwrap());
}

#[test]
fn healthy_only_section_is_hidden_until_verbose() {
    let report = Report::new(vec![
        Diagnostic::new("skills", State::Healthy, "claude current")
            .with_human_section(section("claude:user", "CLAUDE")),
    ]);

    assert!(!report
        .human(&context(), HumanOptions::normal())
        .contains("CLAUDE"));
    assert!(report
        .human(&context(), HumanOptions::verbose())
        .contains("CLAUDE"));
}
```

Ajouter `HumanSection` à l'import du test.

- [ ] **Step 3: Exécuter les tests ciblés et constater le rouge**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test diagnostic --test human_skills
```

Expected: FAIL ; `HumanSection` et les snapshots skills n'existent pas, et le renderer ne regroupe
pas les agents.

- [ ] **Step 4: Ajouter les métadonnées humaines structurées**

Dans `tooling/arnes/src/diagnostic.rs`, conserver `resource`, `state`, `message` et Serde tels quels,
puis étendre uniquement les champs ignorés par Serde :

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanSection {
    key: String,
    label: String,
}

impl HumanSection {
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }

    pub(super) fn key(&self) -> &str {
        &self.key
    }

    pub(super) fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HumanDetail {
    label: String,
    value: String,
}

impl HumanDetail {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}
```

Ajouter `#[serde(skip)] section: Option<HumanSection>` à `Diagnostic` et
`details: Vec<HumanDetail>` à `HumanDiagnostic`. Conserver `group: String` et la signature publique
existante de `with_human`, puis ajouter :

```rust
pub fn with_human_section(mut self, section: HumanSection) -> Self {
    self.section = Some(section);
    self
}

pub fn with_human_summary(mut self, summary: impl Into<String>) -> Self {
    let details = self
        .human
        .take()
        .map(|human| human.details)
        .unwrap_or_default();
    self.human = Some(HumanDiagnostic {
        group: String::new(),
        summary: summary.into(),
        details,
    });
    self
}

pub fn with_human_details(
    mut self,
    details: impl IntoIterator<Item = HumanDetail>,
) -> Self {
    self.human
        .as_mut()
        .expect("human details require a human summary")
        .details = details.into_iter().collect();
    self
}

pub(super) fn section(&self) -> Option<&HumanSection> {
    self.section.as_ref()
}
```

Initialiser `section: None` et les détails vides dans les constructeurs. Ajouter sur `HumanDetail`
les accesseurs `pub(super) fn label(&self) -> &str` et `value(&self) -> &str`; les autres accesseurs
utilisés par `human.rs` restent non publics hors crate. Dans le renderer structuré, un `group` vide
n'affiche aucun sous-en-tête.

- [ ] **Step 5: Implémenter le regroupement et le tri sur des références**

Dans `tooling/arnes/src/diagnostic/human.rs`, faire choisir `render` entre fallback plat et sections
uniquement lorsque tous les diagnostics portent une section. Le chemin structuré appelle
`context.heading(Some(sections.len()))`; le fallback appelle `context.heading(None)`. Les deux
partagent le total sain et l'unique ligne globale `unsupported (non-blocking)`. Étendre l'import à
`use super::{Diagnostic, HumanDiagnostic, State};`, puis ajouter ces unités :

```rust
struct Section<'a> {
    key: &'a str,
    label: &'a str,
    first_index: usize,
    diagnostics: Vec<(usize, &'a Diagnostic)>,
}

fn state_rank(state: State) -> u8 {
    match state {
        State::Error => 3,
        State::Drift => 2,
        State::Unsupported => 1,
        State::Healthy => 0,
    }
}

fn collect_sections(diagnostics: &[Diagnostic]) -> Vec<Section<'_>> {
    let mut sections = Vec::<Section<'_>>::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        let section = diagnostic.section().expect("structured report has sections");
        if let Some(existing) = sections.iter_mut().find(|item| item.key == section.key()) {
            existing.diagnostics.push((index, diagnostic));
        } else {
            sections.push(Section {
                key: section.key(),
                label: section.label(),
                first_index: index,
                diagnostics: vec![(index, diagnostic)],
            });
        }
    }
    sections.sort_by_key(|section| {
        (
            std::cmp::Reverse(
                section
                    .diagnostics
                    .iter()
                    .map(|(_, diagnostic)| state_rank(diagnostic.state))
                    .max()
                    .unwrap_or(0),
            ),
            section.first_index,
        )
    });
    sections
}

fn ordered_diagnostics<'a>(section: &Section<'a>) -> Vec<&'a Diagnostic> {
    let mut diagnostics = section.diagnostics.clone();
    diagnostics.sort_by_key(|(index, diagnostic)| {
        (std::cmp::Reverse(state_rank(diagnostic.state)), *index)
    });
    diagnostics
        .into_iter()
        .map(|(_, diagnostic)| diagnostic)
        .collect()
}
```

Remplacer l'orchestration de `render` par ces fonctions ; `issues` vaut `error + drift` :

```rust
#[derive(Default)]
struct Counts {
    issues: usize,
    unsupported: usize,
    healthy: usize,
}

pub(super) fn render(
    diagnostics: &[Diagnostic],
    context: &HumanContext,
    options: HumanOptions,
) -> String {
    if diagnostics.is_empty() {
        return "No diagnostics".to_owned();
    }
    let structured = diagnostics.iter().all(|diagnostic| diagnostic.section().is_some());
    let sections = structured.then(|| collect_sections(diagnostics));
    let mut lines = summary_lines(context, diagnostics, sections.as_deref());
    let body = match &sections {
        Some(sections) => render_sections(sections, options),
        None => render_flat(diagnostics, options),
    };
    if !body.is_empty() {
        lines.push(String::new());
        lines.extend(body);
    }
    lines.join("\n")
}

fn summary_lines(
    context: &HumanContext,
    diagnostics: &[Diagnostic],
    sections: Option<&[Section<'_>]>,
) -> Vec<String> {
    let mut lines = vec![
        context.heading(sections.map(|sections| sections.len())),
        format!("✓ {} healthy", state_count(diagnostics, State::Healthy)),
    ];
    let unsupported = state_count(diagnostics, State::Unsupported);
    if unsupported > 0 {
        lines.push(format!("! {unsupported} unsupported (non-blocking)"));
    }
    lines
}

fn render_sections(sections: &[Section<'_>], options: HumanOptions) -> Vec<String> {
    let mut lines = Vec::new();
    for section in sections {
        let visible = ordered_diagnostics(section)
            .into_iter()
            .filter(|diagnostic| {
                options.includes_healthy() || diagnostic.state != State::Healthy
            })
            .collect::<Vec<_>>();
        if visible.is_empty() {
            continue;
        }
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(section.label.to_owned());
        lines.push(render_section_counts(&section.diagnostics));
        lines.push(String::new());
        lines.extend(render_section_diagnostics(&visible));
    }
    lines
}

fn render_section_counts(diagnostics: &[(usize, &Diagnostic)]) -> String {
    let counts = diagnostics.iter().fold(Counts::default(), |mut counts, (_, diagnostic)| {
        match diagnostic.state {
            State::Error | State::Drift => counts.issues += 1,
            State::Unsupported => counts.unsupported += 1,
            State::Healthy => counts.healthy += 1,
        }
        counts
    });
    let mut parts = Vec::new();
    if counts.issues > 0 {
        parts.push(plural(counts.issues, "issue", "issues"));
    }
    if counts.unsupported > 0 {
        parts.push(format!("{} unsupported", counts.unsupported));
    }
    parts.push(format!("{} healthy", counts.healthy));
    format!("  {}", parts.join(" · "))
}

fn plural(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

fn render_section_diagnostics(diagnostics: &[&Diagnostic]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut group = None;
    for diagnostic in diagnostics {
        let human = diagnostic.human();
        let next_group = human.map(HumanDiagnostic::group).filter(|group| !group.is_empty());
        if next_group != group {
            if let Some(next_group) = next_group {
                lines.push(format!("  {next_group}"));
            }
            group = next_group;
        }
        let indent = if group.is_some() { 4 } else { 2 };
        lines.extend(render_structured_diagnostic(diagnostic, indent));
    }
    lines
}

fn render_structured_diagnostic(diagnostic: &Diagnostic, indent: usize) -> Vec<String> {
    let human = diagnostic.human();
    let summary = human
        .map(HumanDiagnostic::summary)
        .unwrap_or(&diagnostic.message);
    let mut lines = vec![format!(
        "{}{} {}",
        " ".repeat(indent),
        diagnostic.state.to_string().to_uppercase(),
        super::human_field(summary),
    )];
    if let Some(human) = human {
        lines.extend(human.details().iter().map(|detail| {
            format!(
                "{}{:9} {}",
                " ".repeat(indent + 2),
                detail.label(),
                super::human_field(detail.value()),
            )
        }));
    }
    lines
}
```

Rendre `human_field` `pub(super)` dans `diagnostic.rs`. Ajouter sur `HumanDiagnostic` les accesseurs
`group()`, `summary()` et `details()`. Supprimer `trim_trailing_empty`, devenu inutilisé après cette
orchestration. Le renderer ne doit muter ni réordonner `Report::diagnostics`.

- [ ] **Step 6: Attacher chaque diagnostic skills à sa section au point d'orchestration**

Dans `tooling/arnes/src/skills.rs`, ajouter ces helpers :

```rust
fn human_section(agent: Agent, scope: Scope) -> HumanSection {
    HumanSection::new(
        format!("{agent}:{scope}"),
        agent.to_string().to_uppercase(),
    )
}

fn section_diagnostics(
    diagnostics: Vec<Diagnostic>,
    agent: Agent,
    scope: Scope,
) -> Vec<Diagnostic> {
    let section = human_section(agent, scope);
    diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.with_human_section(section.clone()))
        .collect()
}
```

Dans la boucle de `diagnose`, envelopper le résultat de chaque `diagnose_one` avec
`section_diagnostics`. Pour la branche `combinations.is_empty()`, attacher une section uniquement
lorsque `agent` et `scope` sont tous deux présents ; sinon conserver le fallback non sectionné. Ne
modifier aucun producteur externe ni l'ordre du vecteur.

- [ ] **Step 7: Produire les détails `expected / actual / path` sans parser le message**

Dans `tooling/arnes/src/skills/projection.rs`, introduire une vue humaine passée à
`expected_link` :

```rust
struct ProjectionHuman {
    summary: String,
    path: String,
}

impl ProjectionHuman {
    fn missing_destination(&self, diagnostic: Diagnostic) -> Diagnostic {
        diagnostic
            .with_human_summary(&self.summary)
            .with_human_details([
                HumanDetail::new("expected", "managed skill present"),
                HumanDetail::new("actual", "destination missing"),
                HumanDetail::new("path", &self.path),
            ])
    }
}
```

Dans `leaf`, construire puis passer cette vue à `expected_link` :

```rust
let human = ProjectionHuman {
    summary: name.to_owned(),
    path: label(resource.scope, &installed),
};
```

Dans la branche `ErrorKind::NotFound`, construire d'abord le `Diagnostic` canonique avec `broken`,
puis retourner `human.missing_destination(diagnostic)`. Dans `root`, passer cette vue :

```rust
let human = ProjectionHuman {
    summary: "managed skills projection".to_owned(),
    path: label(resource.scope, resource.destination),
};
```

Les autres erreurs conservent leur message humain de fallback tant qu'aucun détail structuré fidèle
n'est disponible ; aucun `split`, regex ou extraction de texte n'est autorisé.

- [ ] **Step 8: Ajouter la preuve E2E que `skills::diagnose` attache les sections**

Compléter `tooling/arnes/tests/human_skills.rs` avec la fixture isolée :

```rust
#[path = "support/skills.rs"]
mod skill_support;
mod support;

use skill_support::{configured_fixture, run};

#[test]
fn skills_doctor_attaches_agent_sections_without_reading_real_home() {
    let fixture = configured_fixture();
    std::fs::remove_file(fixture.home().join(".cursor/skills/alpha")).unwrap();

    let (code, normal, stderr) = run(&fixture, &["doctor", "skills"]);
    let (_, verbose, verbose_stderr) = run(&fixture, &["doctor", "skills", "-v"]);

    assert_eq!(code, 1, "{normal}");
    assert!(normal.find("CURSOR").unwrap() < normal.find("CLAUDE").unwrap());
    assert!(!normal.contains("HEALTHY"));
    assert!(verbose.contains("HEALTHY"));
    assert!(stderr.is_empty());
    assert!(verbose_stderr.is_empty());
}
```

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test human_skills --test diagnostic
```

Expected: PASS, avec snapshots exacts déterministes et fixture E2E confinée au `HOME` temporaire.

- [ ] **Step 9: Prouver que JSON et ordre canonique n'ont pas changé**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test diagnostic json_output_matches_the_exact_fixture
cargo test --manifest-path tooling/arnes/Cargo.toml --test diagnostic report_preserves_diagnostic_order
```

Expected: PASS pour les deux oracles historiques, sans modification de `report.json`.

- [ ] **Step 10: Committer la vue skills structurée**

```bash
git add tooling/arnes/src/diagnostic.rs tooling/arnes/src/diagnostic/human.rs tooling/arnes/src/skills.rs tooling/arnes/src/skills/projection.rs tooling/arnes/src/main.rs tooling/arnes/tests/diagnostic.rs tooling/arnes/tests/human_skills.rs tooling/arnes/tests/fixtures/skills/report.txt tooling/arnes/tests/fixtures/skills/report-verbose.txt
git commit -m "feat(arnes): group skill diagnostics by agent"
```

### Task 4: Migrer les contrats humains existants et la documentation

**Files:**

- Modify: `tooling/arnes/tests/skills.rs`
- Modify: `tooling/arnes/tests/skill_adversarial.rs`
- Modify: `tooling/arnes/tests/skill_failures.rs`
- Modify: `tooling/arnes/tests/external_managed_skills.rs`
- Modify: `tooling/arnes/tests/external_system_skills.rs`
- Modify: `tooling/arnes/tests/external_cursor_plugins.rs`
- Modify: `tooling/arnes/tests/external_claude_plugins.rs`
- Modify: `tooling/arnes/tests/external_codex_plugins.rs`
- Modify: `tooling/arnes/tests/external_boundary_failures.rs`
- Modify: `tooling/arnes/tests/config.rs`
- Modify: `tooling/arnes/tests/instructions.rs`
- Modify: `tooling/arnes/tests/instruction_adversarial.rs`
- Modify: `tooling/arnes/tests/instruction_failures.rs`
- Modify: `tooling/arnes/tests/prompts.rs`
- Modify: `tooling/arnes/tests/commands.rs`
- Modify: `tooling/arnes/tests/cli.rs`
- Modify: `docs/arnes-capacites-externes.md:18-23`

- [ ] **Step 1: Inventorier les assertions humaines affectées**

Run:

```bash
rg -n 'healthy|\.human\(|doctor.*(skills|config|instructions|prompts|commands)' tooling/arnes/tests -g '*.rs'
```

Expected: chaque assertion de détail sain est classée soit « inventaire » et doit appeler `-v`, soit
« résumé normal » et doit vérifier compteurs/absence de lignes saines. Ne modifier aucun test JSON.

- [ ] **Step 2: Passer en verbose les tests dont l'objet est l'inventaire sain**

Appliquer la transformation explicite suivante aux tableaux argv concernés :

```rust
let (code, stdout, stderr) = run(&fixture, &["doctor", "skills", "-v"]);
```

Pour une commande avec filtres, ajouter `"-v"` sans déplacer les valeurs existantes :

```rust
let (code, stdout, stderr) = run(
    &fixture,
    &[
        "doctor", "commands", "--agent", "claude", "--scope", scope, "-v",
    ],
);
```

Conserver la vue normale pour les tests qui valident `drift`, `error` ou `unsupported`. Mettre à jour
leurs égalités exactes avec l'en-tête et `✓ N healthy`, sans relâcher une égalité en simple
`contains` lorsque l'ordre fait partie du contrat.

- [ ] **Step 3: Ajouter le comportement générique hors skills au test CLI**

Dans `tooling/arnes/tests/human_cli.rs`, ajouter un manifeste sain et vérifier les deux vues :

```rust
mod support;

use support::Fixture;

#[test]
fn verbose_is_generic_across_doctor_resources() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents: []\nresources: []\n",
    );

    let normal = fixture.command(["doctor", "manifest"]);
    let verbose_after = fixture.command(["doctor", "manifest", "--verbose"]);
    let verbose_before = fixture.command(["doctor", "-v", "manifest"]);
    let verbose_after_stdout = String::from_utf8(verbose_after.stdout).unwrap();
    let verbose_before_stdout = String::from_utf8(verbose_before.stdout).unwrap();

    assert_eq!(String::from_utf8(normal.stdout).unwrap(), "Manifest\n✓ 1 healthy\n");
    assert_eq!(
        verbose_after_stdout,
        "Manifest\n✓ 1 healthy\n\nhealthy manifest: manifest is valid\n"
    );
    assert_eq!(verbose_before_stdout, verbose_after_stdout);
}
```

Fusionner l'unique déclaration `mod support;` en tête de fichier ; ne pas la dupliquer.

Dans `tooling/arnes/tests/commands.rs`, ajouter la preuve hors skills avec la fixture déjà importée :

```rust
#[test]
fn command_healthy_details_require_verbose() {
    let fixture = configured_fixture();
    let (_, normal, _) = run(
        &fixture,
        &["doctor", "commands", "--agent", "claude", "--scope", "user"],
    );
    let (_, verbose, _) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "claude", "--scope", "user", "-v",
        ],
    );

    assert!(normal.contains("✓"));
    assert!(!normal.contains("healthy     deploy"));
    assert!(verbose.contains("healthy     deploy"));
}
```

- [ ] **Step 4: Documenter le contrat utilisateur**

Dans `docs/arnes-capacites-externes.md`, remplacer le paragraphe sur les formats par :

```markdown
Les diagnostics conservent le schéma partagé `resource/state/message`. La sortie humaine affiche le
nombre de diagnostics `healthy`, masque leur détail par défaut et inventorie tous les états
`error`, `drift` et `unsupported`. `-v, --verbose` réinsère les détails sains ; `--format json` reste
exhaustif et ne se combine pas avec `--verbose`.
```

- [ ] **Step 5: Exécuter les suites historiquement affectées**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test cli --test config --test instructions --test prompts --test commands --test skills --test skill_adversarial --test skill_failures
```

Expected: PASS.

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test external_managed_skills --test external_system_skills --test external_claude_plugins --test external_cursor_plugins --test external_codex_plugins --test external_boundary_failures
```

Expected: PASS.

- [ ] **Step 6: Vérifier les tailles après migration**

Run:

```bash
wc -l tooling/arnes/src/main.rs tooling/arnes/src/diagnostic.rs tooling/arnes/src/diagnostic/human.rs tooling/arnes/tests/human_cli.rs tooling/arnes/tests/human_skills.rs tooling/arnes/tests/cli.rs tooling/arnes/tests/commands.rs tooling/arnes/tests/config.rs
```

Expected: aucun nouveau fichier de production au-dessus de 250 lignes et aucune fonction de
production au-dessus de 50 lignes. `tests/config.rs` peut rester au-dessus de 250 uniquement parce
que les changements y sont mécaniques et que cette dette préexistante n'est pas copiée ailleurs ;
la signaler dans la livraison.

- [ ] **Step 7: Committer la migration des contrats**

```bash
git add tooling/arnes/tests docs/arnes-capacites-externes.md
git commit -m "test(arnes): adopt concise doctor output"
```

### Task 5: Vérifier l'implémentation complète

**Files:**

- Verify: `tooling/arnes/src/**/*.rs`
- Verify: `tooling/arnes/tests/**/*.rs`
- Verify: `docs/arnes-capacites-externes.md`

- [ ] **Step 1: Formater puis vérifier le format Rust**

Run:

```bash
cargo fmt --manifest-path tooling/arnes/Cargo.toml
cargo fmt --manifest-path tooling/arnes/Cargo.toml --check
```

Expected: les deux commandes terminent avec l'exit code `0`.

- [ ] **Step 2: Exécuter Clippy sur toutes les cibles**

Run:

```bash
cargo clippy --manifest-path tooling/arnes/Cargo.toml --all-targets -- -D warnings
```

Expected: exit code `0`, aucun warning.

- [ ] **Step 3: Exécuter toute la suite Arnes**

Run:

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml
```

Expected: exit code `0`, zéro test échoué.

- [ ] **Step 4: Vérifier le scénario réel sur macOS sans mutation**

Run depuis le checkout canonique :

```bash
cargo run --manifest-path tooling/arnes/Cargo.toml -- doctor skills
cargo run --manifest-path tooling/arnes/Cargo.toml -- doctor skills -v
```

Expected: la première sortie montre le total sain, le drift Cursor puis tous les `unsupported` sans
détails sains ; la seconde ajoute chaque ligne saine dans sa section. Les deux conservent l'exit
code `1` tant que `~/.cursor/skills/enforcement-code` manque. Comparer un snapshot du dépôt et de
`HOME` avant/après avec le helper d'intégration plutôt que revendiquer la lecture seule sur simple
observation manuelle.

- [ ] **Step 5: Vérifier le diff, les commentaires et les extensions couvertes**

Run:

```bash
git diff --check
git diff --stat
git diff -- tooling/arnes docs/arnes-capacites-externes.md
```

Expected: aucun whitespace invalide, aucun changement hors périmètre. Lister dans la livraison tous
les commentaires ajoutés et le fait externe qu'ils enregistrent ; la liste attendue est vide. Les
barrières Rust couvrent `.rs`, mais aucune barrière automatisée ne couvre `.md` : ne pas qualifier la
documentation de « green » sur la seule base de Cargo.

- [ ] **Step 6: Vérifier Ubuntu en CI avant toute affirmation de portabilité**

Après push de la branche, attendre le workflow `.github/workflows/test-arnes.yml` et relever son nom,
son run et son résultat. Expected: job Ubuntu exit `0`. Sans ce run, livrer uniquement la preuve
macOS locale et déclarer Ubuntu non exercé.

- [ ] **Step 7: Committer uniquement les éventuels changements de formatage**

Si `cargo fmt` a modifié des fichiers après Task 4 :

```bash
git add tooling/arnes
git commit -m "style(arnes): format human diagnostics"
```

Sinon, ne créer aucun commit vide.
