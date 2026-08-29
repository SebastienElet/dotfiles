# Migration Rust d’Agent Handoff — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remplacer l’exécutable Bun `agent-handoff` par un package Cargo autonome qui conserve exactement son contrat applicatif observable sur macOS et Linux après réception des octets stdin et avec stdout accessible en écriture.

**Architecture:** `tooling/agent-handoff/` devient un crate binaire indépendant, sans workspace ni dépendance vers Arnes ou `agent-memory`. Le crate sépare parsing du hook, lecture du transcript, décision de seuil et publication atomique du sentinel ; Arnes reste limité à configurer et valider le chemin absolu du binaire. Une implémentation Bun relocalisée temporairement sert d’oracle différentiel, puis disparaît seulement après égalité byte-for-byte des erreurs applicatives, de l’environnement, de stdout, de stderr et du code de sortie dans la frontière de parité.

**Tech Stack:** Rust 2024, `serde`, `serde_json`, bibliothèque standard Rust, `tempfile` pour les tests, Bun uniquement comme oracle temporaire de migration, Cargo, Make et GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md`

## Contraintes globales

- L’ADR-042 en vigueur doit attribuer le runtime handoff exclusivement à `agent-handoff` avant toute implémentation de ce plan.
- Le package réside dans `tooling/agent-handoff/`, possède son propre `Cargo.toml` et son propre `Cargo.lock`, et n’appartient à aucun workspace Cargo.
- `agent-handoff` ne dépend ni d’Arnes, ni d’`agent-memory`, ni d’un crate interne partagé.
- Arnes configure et valide uniquement la commande absolue `~/.local/bin/agent-handoff`; il n’exécute, ne parse et ne réimplémente aucun comportement handoff.
- La migration conserve les octets écrits sur stdout et stderr ainsi que le code de sortie pour chaque ensemble d'octets stdin déjà reçu, environnement et état de fichiers caractérisés, sous l'hypothèse d'un stdout accessible en écriture.
- Les échecs de lecture stdin et d'écriture stdout au niveau du système d'exploitation sont hors de la frontière différentielle : Rust les normalise en `unexpected failure`, code 3; aucune divergence applicative n'est admise derrière cette exception.
- L’implémentation Bun n’est supprimée qu’après exécution verte de la matrice différentielle Bun/Rust.
- Le `Makefile` est l’unique installateur; dans un worktree, sa cible d’installation est vérifiée exclusivement avec `make -n` et les tests de déploiement.
- Le binaire est formaté, linté et testé depuis son propre manifeste sur `macos-latest` et `ubuntu-latest`.
- `RUSTFLAGS=-Funsafe-code`; aucune dépendance native, commande externe ou comportement spécifique à GNU n’est introduit.
- Aucun commentaire de code n’est ajouté. Les fonctions de production dépassant 50 lignes logiques et les fichiers manuscrits dépassant 250 lignes déclenchent une séparation ou une justification dans la livraison.
- Tous les échecs d’entrée, de transcript, d’environnement et de sentinel échouent explicitement; aucun échec de contrôle ne devient un succès silencieux.

---

### Tâche 1 : Figer le contrat Bun et ouvrir la frontière Cargo

**Fichiers :**

- Créer : `tooling/agent-handoff-legacy/agent-handoff`
- Créer : `tooling/agent-handoff-legacy/error.ts`
- Créer : `tooling/agent-handoff-legacy/run.ts`
- Créer : `tooling/agent-handoff-legacy/transcript.ts`
- Créer : `tooling/agent-handoff-parity-support.ts`
- Créer : `tooling/agent-handoff-parity.test.ts`
- Déplacer : `tooling/agent-handoff`, `tooling/agent-handoff-error.ts`, `tooling/agent-handoff.ts`, `tooling/agent-handoff-transcript.ts`
- Modifier : `tooling/agent-handoff-test-support.ts`
- Modifier : `Makefile`

**Interfaces :**

- Consomme : le point d’entrée Bun actuel et son contrat déjà couvert par `tooling/agent-handoff.test.ts` et `tooling/agent-handoff-failures.test.ts`.
- Produit : `runLegacy(input: Uint8Array, environment: Readonly<Record<string, string>>, fixture: Fixture): Promise<HookResult>`; `HookResult = Readonly<{ exitCode: number; stdout: Uint8Array; stderr: Uint8Array }>`; une cible Make temporaire qui continue à déployer le runtime Bun relocalisé.

- [ ] **Étape 1 : Écrire le RED de la matrice différentielle**

Créer une table `parityCases` qui construit dans un répertoire temporaire les cas suivants et compare `exitCode`, `stdout` et `stderr` comme octets, sans parser les sorties :

```ts
type ParityCase = Readonly<{
  name: string;
  input: (fixture: Fixture) => Uint8Array;
  environment: (fixture: Fixture) => Readonly<Record<string, string>>;
  prepare?: (fixture: Fixture) => void;
}>;

test.each(parityCases)(
  "matches Bun for $name",
  async ({ input, environment, prepare }) => {
    const legacyFixture = createParityFixture();
    const rustFixture = createParityFixture();
    prepare?.(legacyFixture);
    prepare?.(rustFixture);
    const legacy = await runLegacy(
      input(legacyFixture),
      environment(legacyFixture),
      legacyFixture,
    );
    const rust = await runRust(
      input(rustFixture),
      environment(rustFixture),
      rustFixture,
    );
    expect(rust).toEqual(legacy);
  },
);
```

La table nomme explicitement : JSON invalide; JSON `null`; objet sans événement; événements Claude et Codex autres que `Stop`; deux noms d’événement dont un contradictoire; `session_id` absent, vide, `.`, `..`, avec `/`, et valide avec lettres, chiffres, `.`, `_`, `-`; transcript absent; `stop_hook_active` non booléen; hook déjà actif avec transcript et environnement absents; transcript Claude sous seuil, à 85 %, au-dessus, sidechain et dernier record main-chain; transcript avec octet UTF-8 invalide isolé et octet invalide dans une chaîne JSON ignorée; transcript Codex avec fenêtre propre et invocation `$handoff`; les 499, 500 et 501 dernières lignes physiques; ligne JSON malformée retenue et ligne malformée hors fenêtre; nombres négatif, fractionnaire, supérieur à `Number.MAX_SAFE_INTEGER`, total Claude supérieur à `Number.MAX_SAFE_INTEGER`, fenêtre Codex nulle; seuil explicite valide, vide, nul, négatif, fractionnaire, `85k` et supérieur à `Number.MAX_SAFE_INTEGER`; fenêtre Claude absente, vide et valide; priorité `XDG_STATE_HOME`, repli sur `HOME`, absence des deux; racine XDG contenant `file/..` ou le symlink `alias/..`; sentinel fichier déjà présent; sentinel répertoire déjà présent; parent du sentinel fichier non répertoire; trois processus concurrents du même `session_id`; stdin non UTF-8; argument CLI supplémentaire ignoré.

Chaque runtime reçoit un fixture distinct mais structurellement identique afin qu’un sentinel créé par le premier ne modifie jamais l’observation du second. Chaque cas part d’un environnement vidé puis reçoit seulement `PATH`, `HOME`, `XDG_STATE_HOME`, `CLAUDE_CODE_AUTO_COMPACT_WINDOW` et les overrides déclarés. Les cas concurrents comparent le multiensemble trié des trois résultats et exigent exactement une sortie de blocage.

- [ ] **Étape 2 : Exécuter le test pour vérifier son échec**

Commande :

```bash
bun test tooling/agent-handoff-parity.test.ts
```

Résultat attendu : FAIL parce que `tooling/agent-handoff/target/debug/agent-handoff` n’existe pas encore; le runner Bun doit déjà produire un résultat complet.

- [ ] **Étape 3 : Relocaliser l’oracle Bun sans changer son comportement**

Déplacer les quatre fichiers dans `tooling/agent-handoff-legacy/`, réduire leurs noms internes à `error.ts`, `run.ts` et `transcript.ts`, puis adapter exclusivement leurs imports relatifs. Modifier `agent-handoff-test-support.ts` pour cibler `agent-handoff-legacy/agent-handoff`.

Le support de parité résout le binaire Bun une fois avec `Bun.which("bun")`, lance `bun tooling/agent-handoff-legacy/agent-handoff`, transmet les octets sur stdin, utilise `env` sans héritage implicite et retourne les trois observations brutes. `runRust` cible `tooling/agent-handoff/target/debug/agent-handoff` et retourne la même forme.

Conserver provisoirement l’installation existante avec :

```make
${LOCAL_BIN}/agent-handoff: ${DOTFILES_PATH}/tooling/agent-handoff-legacy/agent-handoff | ${LOCAL_BIN}
	${CREATE_SYMLINK}
```

- [ ] **Étape 4 : Vérifier que la relocalisation ne modifie pas l’oracle**

Commandes :

```bash
bun test tooling/agent-handoff.test.ts tooling/agent-handoff-failures.test.ts
bun run typecheck
bun tooling/format-typescript.ts --check
make -n claude-code-hooks
make -n codex-hooks
```

Résultat attendu : tous les tests Bun passent; les deux dry-runs montrent encore `~/.local/bin/agent-handoff` lié au launcher Bun relocalisé puis configuré par Arnes.

- [ ] **Étape 5 : Commit**

```bash
git add Makefile tooling/agent-handoff tooling/agent-handoff-legacy tooling/agent-handoff-test-support.ts tooling/agent-handoff-parity-support.ts tooling/agent-handoff-parity.test.ts
git add -u -- tooling/agent-handoff-error.ts tooling/agent-handoff.ts tooling/agent-handoff-transcript.ts
git commit -m "test(handoff): freeze the Bun runtime contract"
```

### Tâche 2 : Parser strictement les événements et transcripts en Rust

**Fichiers :**

- Créer : `tooling/agent-handoff/Cargo.toml`
- Créer : `tooling/agent-handoff/Cargo.lock`
- Créer : `tooling/agent-handoff/src/lib.rs`
- Créer : `tooling/agent-handoff/src/error.rs`
- Créer : `tooling/agent-handoff/src/event.rs`
- Créer : `tooling/agent-handoff/src/transcript.rs`
- Créer : `tooling/agent-handoff/src/main.rs`
- Créer : `tooling/agent-handoff/tests/event.rs`
- Créer : `tooling/agent-handoff/tests/transcript.rs`

**Interfaces :**

- Consomme : stdin comme octets; documents JSON des hooks `Stop` Claude/Codex; transcripts JSONL Claude/Codex.
- Produit : `pub struct HandoffError { pub message: String, pub exit_code: u8 }`; `pub fn parse_hook_event(input: &[u8]) -> Result<HookEvent, HandoffError>`; `pub fn find_latest_usage(transcript: &str) -> Result<Usage, HandoffError>`; `pub enum Agent { ClaudeCode, Codex }`; `pub struct Usage { pub agent: Agent, pub used: u64, pub window: Option<u64> }`.

- [ ] **Étape 1 : Écrire les RED du parsing d’événement**

Tester chaque branche de la matrice de tâche 1 et les valeurs exactes :

```rust
assert_eq!(parse_hook_event(b"not-json").unwrap_err(), HandoffError::usage("invalid hook event: expected JSON"));
assert_eq!(parse_hook_event(br#"{"hook_event_name":"Stop","session_id":"../x","transcript_path":"/tmp/t"}"#).unwrap_err(), HandoffError::usage("invalid session_id"));
assert_eq!(parse_hook_event(br#"{"event":"Stop","session_id":"s-1","transcript_path":"/tmp/t"}"#).unwrap(), HookEvent {
    session_id: "s-1".into(),
    stop_hook_active: false,
    transcript_path: PathBuf::from("/tmp/t"),
});
```

`HandoffError::usage` fixe `exit_code = 1`; `HandoffError::unexpected` fixe `exit_code = 3`. La validation de session accepte exclusivement `[A-Za-z0-9._-]+`, sauf `.` et `..`, sans crate regex.

- [ ] **Étape 2 : Exécuter les tests d’événement pour vérifier leur échec**

```bash
cargo test --manifest-path tooling/agent-handoff/Cargo.toml --test event
```

Résultat attendu : FAIL sur imports ou fonctions absentes.

- [ ] **Étape 3 : Implémenter le modèle d’erreur et le parser d’événement minimaux**

Créer le manifeste indépendant puis son lockfile :

```toml
[package]
name = "agent-handoff"
version = "0.1.0"
edition = "2024"

[lints.rust]
unsafe_code = "forbid"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
tempfile = "3.21"
```

```bash
cargo generate-lockfile --manifest-path tooling/agent-handoff/Cargo.toml
```

Déclarer les types exacts :

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookEvent {
    pub session_id: String,
    pub stop_hook_active: bool,
    pub transcript_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffError {
    pub message: String,
    pub exit_code: u8,
}
```

Parser d’abord `serde_json::Value`, exiger un objet, valider séparément `hook_event_name` et `event` lorsqu’ils existent, puis les trois champs utiles. Convertir stdin en texte avec `String::from_utf8_lossy` afin de conserver le comportement observé de `Bun.stdin.text()` sur les octets invalides.

`lib.rs` déclare `mod error`, `mod event`, `mod transcript` et réexporte uniquement les interfaces publiques nommées dans cette tâche. `main.rs` contient provisoirement `fn main() {}` afin que le binaire existe sans prétendre implémenter le runtime avant la tâche 4.

- [ ] **Étape 4 : Écrire les RED du transcript**

Tester Claude, Codex, priorité du dernier record supporté, sidechains, fenêtre physique de 500 lignes, ligne vide, JSON retenu invalide et toutes les bornes numériques. Utiliser `const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991` et vérifier que chaque nombre ou somme au-delà échoue avec le diagnostic Bun exact.

```rust
assert_eq!(find_latest_usage(&claude_usage(84_999, false)).unwrap().used, 84_999);
assert_eq!(find_latest_usage(&codex_usage(90_000, 100_000)).unwrap().window, Some(100_000));
assert_eq!(find_latest_usage("{broken}\n").unwrap_err().message, "malformed transcript JSON at retained line 1");
```

- [ ] **Étape 5 : Exécuter les tests de transcript pour vérifier leur échec**

```bash
cargo test --manifest-path tooling/agent-handoff/Cargo.toml --test transcript
```

Résultat attendu : FAIL sur `find_latest_usage` absent.

- [ ] **Étape 6 : Implémenter le parser de transcript minimal**

`find_latest_usage` retire seulement l’ultime segment vide créé par un `\n` final, conserve les 500 dernières lignes physiques, ignore les lignes dont `trim()` est vide et parcourt dans l’ordre. `parse_claude_usage` additionne `input_tokens`, `cache_read_input_tokens` et `cache_creation_input_tokens`; les deux caches absents valent zéro. `parse_codex_usage` lit seulement `last_token_usage.input_tokens` et `model_context_window`, refuse la fenêtre nulle et ignore `total_token_usage`.

- [ ] **Étape 7 : Exécuter les tests Rust ciblés**

```bash
cargo test --manifest-path tooling/agent-handoff/Cargo.toml --test event --test transcript
cargo fmt --manifest-path tooling/agent-handoff/Cargo.toml --check
cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings
```

Résultat attendu : PASS. Le test différentiel complet reste rouge car `main` ne porte pas encore le comportement runtime.

- [ ] **Étape 8 : Commit**

```bash
git add tooling/agent-handoff
git commit -m "feat(handoff): parse hook events and transcripts in Rust"
```

### Tâche 3 : Reproduire la décision de seuil et la sortie du hook

**Fichiers :**

- Créer : `tooling/agent-handoff/src/environment.rs`
- Créer : `tooling/agent-handoff/src/decision.rs`
- Créer : `tooling/agent-handoff/tests/decision.rs`
- Modifier : `tooling/agent-handoff/src/lib.rs`

**Interfaces :**

- Consomme : `Usage` de tâche 2 et valeurs optionnelles `HANDOFF_TOKEN_THRESHOLD`, `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, `XDG_STATE_HOME`, `HOME`.
- Produit : `pub struct Environment`; `pub fn Environment::from_iter(values: impl IntoIterator<Item = (OsString, OsString)>) -> Self`; `pub fn Environment::current() -> Self`; `pub fn select_threshold(usage: &Usage, environment: &Environment) -> Result<u64, HandoffError>`; `pub fn handoff_output(usage: &Usage, threshold: u64) -> Vec<u8>`.

- [ ] **Étape 1 : Écrire les RED de l’environnement et du seuil**

Éprouver le seuil explicite prioritaire, le repli fenêtre Claude, la fenêtre Codex prioritaire, les chaînes vides et toutes les formes numériques refusées. Exiger le plancher entier exact :

```rust
assert_eq!(select_threshold(&claude(0), &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "100000")])).unwrap(), 85_000);
assert_eq!(select_threshold(&codex(0, 100_001), &Environment::default()).unwrap(), 85_000);
assert_eq!(select_threshold(&claude(0), &environment(&[("HANDOFF_TOKEN_THRESHOLD", "85k")])).unwrap_err().message, "invalid HANDOFF_TOKEN_THRESHOLD");
```

Le calcul `(window * 85) / 100` utilise `u128` intermédiaire, puis revient en `u64`; les valeurs acceptées restent bornées par `MAX_SAFE_INTEGER`.

- [ ] **Étape 2 : Exécuter le test pour vérifier son échec**

```bash
cargo test --manifest-path tooling/agent-handoff/Cargo.toml --test decision
```

Résultat attendu : FAIL sur `Environment`, `select_threshold` et `handoff_output` absents.

- [ ] **Étape 3 : Implémenter l’environnement fermé et la décision**

`Environment` ne conserve que les quatre variables du contrat. `Environment::current` lit avec `var_os` et convertit avec `to_string_lossy`, ce qui reproduit la frontière texte de `NodeJS.ProcessEnv` sans panic sur un environnement Unix non UTF-8; cette conversion reste confinée à la frontière du processus.

La sortie est produite par `serde_json::to_vec_pretty` depuis :

```rust
#[derive(Serialize)]
struct BlockDecision<'a> {
    decision: &'static str,
    reason: &'a str,
}
```

Ajouter exactement un `\n`. La raison utilise la division entière par 1000 et `/handoff` pour Claude Code, `$handoff` pour Codex.

- [ ] **Étape 4 : Vérifier les octets de sortie**

```rust
assert_eq!(
    handoff_output(&claude(85_000), 85_000),
    b"{\n  \"decision\": \"block\",\n  \"reason\": \"Context is at 85k tokens, past the 85k handoff threshold. Start no new work. Use /handoff to emit the resume prompt for a fresh session, then stop.\"\n}\n",
);
```

Puis exécuter :

```bash
cargo test --manifest-path tooling/agent-handoff/Cargo.toml --test decision
cargo fmt --manifest-path tooling/agent-handoff/Cargo.toml --check
cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings
```

Résultat attendu : PASS.

- [ ] **Étape 5 : Commit**

```bash
git add tooling/agent-handoff
git commit -m "feat(handoff): preserve threshold decisions in Rust"
```

### Tâche 4 : Porter le sentinel et fermer le contrat runtime

**Fichiers :**

- Créer : `tooling/agent-handoff/src/state.rs`
- Créer : `tooling/agent-handoff/src/run.rs`
- Créer : `tooling/agent-handoff/tests/cli.rs`
- Créer : `tooling/agent-handoff/tests/cli/runtime_parity.rs`
- Créer : `tooling/agent-handoff/tests/concurrency.rs`
- Créer : `tooling/agent-handoff-parity-runtime-cases.ts`
- Modifier : `tooling/agent-handoff/src/lib.rs`
- Modifier : `tooling/agent-handoff/src/main.rs`
- Modifier : `tooling/agent-handoff-parity.test.ts`

**Interfaces :**

- Consomme : `HookEvent`, `Usage`, `Environment`, seuil et sérialisation des tâches 2–3; vrai système de fichiers local.
- Produit : `pub enum SentinelState { Created, Existing }`; `pub fn state_root(environment: &Environment) -> Result<PathBuf, HandoffError>`; `pub fn inspect_sentinel(path: &Path) -> Result<bool, HandoffError>`; `pub fn create_sentinel(path: &Path) -> Result<SentinelState, HandoffError>`; `pub fn run_agent_handoff(input: &[u8], environment: &Environment, stdout: &mut impl Write) -> Result<(), HandoffError>`; un vrai binaire avec les codes `0`, `1` et `3`.

- [ ] **Étape 1 : Écrire les RED du système de fichiers**

Avec `tempfile::TempDir`, couvrir : priorité XDG; repli HOME; absence des deux; normalisation POSIX lexicale des séparateurs, `.` et `..`; racines contenant un composant non-répertoire `file/..` ou un symlink `alias/..`; sentinel absent; sentinel fichier existant; sentinel répertoire existant; erreur d’inspection; création concurrente; parent existant comme fichier. Exiger :

```rust
assert_eq!(create_sentinel(&path).unwrap(), SentinelState::Created);
assert_eq!(create_sentinel(&path).unwrap(), SentinelState::Existing);
assert_eq!(inspect_sentinel(&directory).unwrap_err().exit_code, 3);
assert_eq!(create_sentinel(&blocked_child).unwrap_err().message, "cannot create handoff sentinel");
```

La création utilise `create_dir_all` puis `OpenOptions::new().write(true).create_new(true).open(path)`. Seul `AlreadyExists` devient `Existing`; tout autre échec devient `cannot create handoff sentinel`, code 3. Une erreur `NotFound` lors de l’inspection signifie absent; toute autre erreur devient `cannot inspect handoff sentinel`, code 3.

- [ ] **Étape 2 : Exécuter les tests pour vérifier leur échec**

```bash
cargo test --manifest-path tooling/agent-handoff/Cargo.toml --test concurrency
```

Résultat attendu : FAIL sur l’API sentinel absente.

- [ ] **Étape 3 : Implémenter la publication atomique du sentinel**

Écrire `state.rs` avec les seules opérations ci-dessus. Le test concurrent lance plusieurs threads sur le même chemin et exige un seul `Created`; aucun pré-check n’est utilisé comme verrou.

- [ ] **Étape 4 : Écrire les RED du vrai point d’entrée**

Les tests lancent `env!("CARGO_BIN_EXE_agent-handoff")`, utilisent `env_clear`, transmettent stdin comme octets déjà reçus par le processus, capturent stdout/stderr/code et drainent un stdout accessible en écriture. Ils exigent les quatre formes exactes :

```text
succès sans blocage: exit 0, stdout vide, stderr vide
succès avec blocage: exit 0, JSON pretty exact sur stdout, stderr vide
erreur d’usage: exit 1, stdout vide, "agent-handoff: <message>\n" sur stderr
erreur inattendue: exit 3, stdout vide, "agent-handoff: <message>\n" sur stderr
```

Vérifier aussi que `stop_hook_active: true` retourne avant la résolution d’environnement et toute lecture de transcript, et qu’un sentinel n’est créé qu’après validation du transcript et constat du seuil atteint.

- [ ] **Étape 5 : Exécuter les tests CLI pour vérifier leur échec**

```bash
cargo test --manifest-path tooling/agent-handoff/Cargo.toml --test cli
```

Résultat attendu : FAIL car `run_agent_handoff` ou `main` ne conduit pas encore le flux.

- [ ] **Étape 6 : Implémenter le flux runtime minimal**

`run_agent_handoff` suit strictement cet ordre : parser l’événement; sortir immédiatement si récursif; résoudre et normaliser lexicalement le state root selon `path.posix.join`; inspecter le sentinel; lire intégralement le transcript comme octets ou retourner `cannot read transcript`, code 1; décoder avec remplacement UTF-8 lossy; trouver l’usage; sélectionner le seuil; sortir si sous seuil; créer le sentinel avec `create_new`; sortir si un concurrent l’a créé; écrire le JSON exact. `main` lit stdin en octets, construit `Environment::current`, écrit le diagnostic préfixé sur stderr en cas d’erreur et retourne `ExitCode::from(error.exit_code)`; tout échec non classé, notamment une lecture stdin ou une écriture stdout impossible au niveau du système d'exploitation, devient `unexpected failure`, code 3.

- [ ] **Étape 7 : Exécuter la matrice différentielle complète**

```bash
cargo build --manifest-path tooling/agent-handoff/Cargo.toml
bun test tooling/agent-handoff-parity.test.ts
cargo test --manifest-path tooling/agent-handoff/Cargo.toml
```

Résultat attendu : PASS; après réception de stdin et avec stdout accessible en écriture, chaque cas compare les octets stdout/stderr et le code de sortie des deux processus. Si un cas applicatif diverge, conserver Bun comme oracle déployé, ajouter le cas minimal qui révèle la divergence, puis corriger Rust avant de poursuivre. Les pannes de lecture stdin et d'écriture stdout au niveau du système d'exploitation restent couvertes séparément par le contrat Rust fail-closed `unexpected failure`, code 3.

- [ ] **Étape 8 : Commit**

```bash
git add tooling/agent-handoff tooling/agent-handoff-parity.test.ts tooling/agent-handoff-parity-runtime-cases.ts
git commit -m "feat(handoff): complete the Rust runtime contract"
```

### Tâche 5 : Déployer Rust, limiter Arnes au câblage et retirer Bun

**Fichiers :**

- Modifier : `Makefile`
- Modifier : `tooling/deployment-codex-wiring.test.ts`
- Modifier : `tooling/arnes/src/hooks.rs`
- Modifier : `tooling/arnes/tests/hooks_setup.rs`
- Modifier : `tooling/arnes/tests/hooks_validation.rs`
- Supprimer : `tooling/agent-handoff-legacy/`
- Supprimer : `tooling/agent-handoff.test.ts`
- Supprimer : `tooling/agent-handoff-failures.test.ts`
- Supprimer : `tooling/agent-handoff-test-support.ts`
- Supprimer : `tooling/agent-handoff-parity-support.ts`
- Supprimer : `tooling/agent-handoff-parity.test.ts`
- Supprimer : `tooling/agent-handoff-parity-runtime-cases.ts`

**Interfaces :**

- Consomme : `tooling/agent-handoff/target/release/agent-handoff` vert sur toute la matrice différentielle.
- Produit : cible fichier `tooling/agent-handoff/target/release/agent-handoff` dépendant du manifeste, du lockfile, des sources et des tests du crate; cible fichier `~/.local/bin/agent-handoff` vers ce binaire release; hooks Claude/Codex qui ne contiennent que ce chemin absolu; tests Rust permanents qui conservent chaque cas de la matrice sans dépendre de Bun.

- [ ] **Étape 1 : Écrire les RED de déploiement et de réconciliation**

Dans les tests de déploiement, exiger pour `claude-code-hooks` et `codex-hooks` le lien vers `tooling/agent-handoff/target/release/agent-handoff` sans invocation de Bun pour handoff. Chaque modification du manifeste, du lockfile, des sources ou des tests exacts du crate rend le binaire release obsolète et déclenche un unique build; un second appel ne reconstruit pas le binaire. Une destination absente est créée; le lien historique exact vers `${DOTFILES_PATH}/tooling/agent-handoff` est migré seulement lorsque le binaire release est plus récent; une destination inattendue à jour n’est ni inspectée ni modifiée, tandis qu’une destination inattendue obsolète est refusée sans écrasement.

Dans les tests Arnes, construire un exécutable fixture `~/.local/bin/agent-handoff`, puis exiger que le hook configuré soit exactement ce chemin absolu, sans arguments Codex et avec `args: []` Claude. Ajouter une fixture contenant les anciens chemins `${repository}/tooling/agent-handoff` et `${repository}/scripts/agent_handoff`; `setup hooks` les retire, conserve les handlers tiers et installe uniquement `~/.local/bin/agent-handoff`. Aucun test Arnes ne lance le binaire handoff ni ne lit son sentinel.

- [ ] **Étape 2 : Exécuter les tests pour vérifier leur échec**

```bash
bun test tooling/deployment-codex-wiring.test.ts
cargo test --manifest-path tooling/arnes/Cargo.toml --test hooks_setup --test hooks_validation
```

Résultat attendu : FAIL sur la recette Bun encore active et sur la migration incomplète des anciennes commandes.

- [ ] **Étape 3 : Basculer le Makefile vers le crate indépendant**

Déployer le crate et son lien par deux cibles fichier ordinaires conformes aux ADR-001 et ADR-003 :

```make
${DOTFILES_PATH}/tooling/agent-handoff/target/release/agent-handoff: \
	${DOTFILES_PATH}/tooling/agent-handoff/Cargo.lock \
	${DOTFILES_PATH}/tooling/agent-handoff/Cargo.toml \
	${DOTFILES_PATH}/tooling/agent-handoff/src/decision.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/src/environment.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/src/error.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/src/event.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/src/lib.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/src/main.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/src/run.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/src/state.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/src/transcript.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/tests/cli.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/tests/cli/runtime_parity.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/tests/concurrency.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/tests/decision.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/tests/event.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/tests/transcript.rs \
	${DOTFILES_PATH}/tooling/agent-handoff/tests/transcript/numeric.rs \
	| ${BREW_BIN}/cargo
	cd ${DOTFILES_PATH}/tooling/agent-handoff && ${BREW_BIN}/cargo build --release
	test -x "$@"
	touch "$@"
${LOCAL_BIN}/agent-handoff: ${DOTFILES_PATH}/tooling/agent-handoff/target/release/agent-handoff | ${LOCAL_BIN}
	test ! -L "$@" || test "$$(readlink "$@")" != "${DOTFILES_PATH}/tooling/agent-handoff" || ln -sfn "$<" "$@"
	test -e "$@" || test -L "$@" || ln -s "$<" "$@"
	test "$$(readlink "$@")" = "$<"
```

Faire dépendre `claude-code-hooks` et `codex-hooks` de `${LOCAL_BIN}/agent-handoff`. La recette du lien ne s’exécute que si la cible est absente ou obsolète; elle refuse alors un fichier réel, un lien vers une autre source et un lien pendant inattendu, sans les supprimer.

- [ ] **Étape 4 : Borner Arnes à la migration de configuration**

Changer l’interface privée en `fn handoff_aliases(command: &Path, repository: &Path) -> Result<Vec<String>, HooksError>`. La liste contient le chemin absolu courant, l’ancienne source `${repository}/tooling/agent-handoff` et `${repository}/scripts/agent_handoff`; elle n’inspecte ni n’exécute le runtime. Utiliser cette liste seulement pour retirer les anciennes représentations dans la configuration avant d’insérer le chemin absolu courant.

- [ ] **Étape 5 : Conserver la matrice comme tests Rust permanents**

Avant suppression, vérifier que `tests/event.rs`, `tests/transcript.rs`, `tests/decision.rs`, `tests/cli.rs`, `tests/cli/runtime_parity.rs` et `tests/concurrency.rs` nomment chacun des cas de `parityCases` et `runtimeParityCases`. Ajouter tout cas absent au test Rust correspondant et relancer :

```bash
cargo test --manifest-path tooling/agent-handoff/Cargo.toml
cargo build --manifest-path tooling/agent-handoff/Cargo.toml
bun test tooling/agent-handoff-parity.test.ts
```

Résultat attendu : PASS des tests permanents et de la dernière comparaison différentielle Bun/Rust.

- [ ] **Étape 6 : Supprimer l’implémentation Bun seulement après la preuve**

Supprimer le répertoire legacy, ses anciens tests et le runner différentiel. Vérifier qu’aucun code ou test ne référence les fichiers retirés :

```bash
rg -n "agent-handoff-legacy|agent-handoff-error|agent-handoff-transcript|agent-handoff-test-support|agent-handoff-parity" . --glob '!docs/superpowers/plans/**'
```

Résultat attendu : aucune correspondance. Les mentions de `agent-handoff` restantes désignent le binaire, son crate, son hook, sa skill ou sa documentation.

- [ ] **Étape 7 : Vérifier le déploiement et la frontière Arnes**

```bash
make -n "$(pwd)/tooling/agent-handoff/target/release/agent-handoff"
make -n "$HOME/.local/bin/agent-handoff"
make -n claude-code-hooks
make -n codex-hooks
bun test tooling/deployment-agent-handoff.test.ts tooling/deployment-codex-wiring.test.ts
cargo test --manifest-path tooling/arnes/Cargo.toml --test hooks_setup --test hooks_validation
cargo test --manifest-path tooling/agent-handoff/Cargo.toml
git diff --check
```

Résultat attendu : PASS; les dry-runs ne mutent pas le poste, construisent le crate autonome et configurent les hooks via Arnes. Aucun module Arnes ne dépend du crate ou de son état.

- [ ] **Étape 8 : Commit**

```bash
git add Makefile tooling/deployment-codex-wiring.test.ts tooling/arnes/src/hooks.rs tooling/arnes/tests/hooks_setup.rs tooling/arnes/tests/hooks_validation.rs tooling/agent-handoff
git add -u -- tooling/agent-handoff-legacy tooling/agent-handoff.test.ts tooling/agent-handoff-failures.test.ts tooling/agent-handoff-test-support.ts tooling/agent-handoff-parity-support.ts tooling/agent-handoff-parity.test.ts tooling/agent-handoff-parity-runtime-cases.ts
git commit -m "refactor(handoff): deploy the independent Rust binary"
```

### Tâche 6 : Fermer les oracles macOS et Linux

**Fichiers :**

- Créer : `.github/workflows/test-agent-handoff.yml`
- Créer : `tooling/agent-handoff-ci.test.ts`
- Modifier : `.github/workflows/test-typescript.yml`

**Interfaces :**

- Consomme : manifeste autonome `tooling/agent-handoff/Cargo.toml`, tests Rust permanents et tests de déploiement.
- Produit : gate CI dédiée sur `macos-latest` et `ubuntu-latest` exécutant format, Clippy et tous les tests du crate; contrat versionné qui refuse la disparition d’un OS, d’une commande ou d’un path filter.

- [ ] **Étape 1 : Écrire le RED du workflow**

Dans `tooling/agent-handoff-ci.test.ts`, parser le YAML avec `Bun.YAML.parse` et un schéma Zod fermé. Exiger :

```ts
const agentHandoffJob = z.object({
  strategy: z.object({
    matrix: z.object({
      os: z.tuple([z.literal("macos-latest"), z.literal("ubuntu-latest")]),
    }),
  }),
  "runs-on": z.literal("${{ matrix.os }}"),
  env: z.object({ RUSTFLAGS: z.literal("-Funsafe-code") }),
  steps: z.tuple([
    z.object({ uses: z.literal("actions/checkout@v5") }),
    z.object({
      run: z.literal(
        "cargo fmt --manifest-path tooling/agent-handoff/Cargo.toml --check",
      ),
    }),
    z.object({
      run: z.literal(
        "cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings",
      ),
    }),
    z.object({
      run: z.literal(
        "cargo test --manifest-path tooling/agent-handoff/Cargo.toml",
      ),
    }),
  ]),
});
```

Exiger les path filters `.cargo/**`, `tooling/agent-handoff/**`, `Makefile`, `tooling/deployment-codex-wiring.test.ts`, `tooling/arnes/**` et le workflow lui-même pour `push` et `pull_request`. Ajouter des mutations adversariales qui retirent successivement Linux, macOS, Clippy, les tests ou un filtre, et exiger leur rejet.

- [ ] **Étape 2 : Exécuter le contrat pour vérifier son échec**

```bash
bun test tooling/agent-handoff-ci.test.ts
```

Résultat attendu : FAIL car le workflow dédié est absent.

- [ ] **Étape 3 : Créer le workflow minimal et retirer les anciens tests Bun**

Créer `.github/workflows/test-agent-handoff.yml` avec le job exact validé ci-dessus, `name: Agent Handoff tests`, et les filtres symétriques. Aucun `continue-on-error`, `if` permissif, suppression stderr ou fallback n’est admis.

Retirer de `test-typescript.yml` uniquement les références devenues impossibles aux anciens fichiers handoff; ne réduire aucune autre gate TypeScript.

- [ ] **Étape 4 : Exécuter tous les oracles locaux disponibles**

Sur le worktree macOS courant :

```bash
cargo fmt --manifest-path tooling/agent-handoff/Cargo.toml --check
cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tooling/agent-handoff/Cargo.toml
cargo test --manifest-path tooling/arnes/Cargo.toml --test hooks_setup --test hooks_validation
bun test tooling/agent-handoff-ci.test.ts tooling/deployment-codex-wiring.test.ts
bun run typecheck
git diff --check
```

Résultat attendu : PASS sur macOS, avec `uname -s` et `uname -m` enregistrés dans la livraison. La preuve Linux reste conditionnée au job GitHub Actions `Agent Handoff tests (ubuntu-latest)` vert; aucune phrase de livraison ne transforme le résultat macOS local en garantie Linux.

- [ ] **Étape 5 : Inspecter les frontières et tailles**

```bash
rg -n "arnes|agent-memory" tooling/agent-handoff --glob '!Cargo.lock'
find tooling/agent-handoff/src -type f -print0 | xargs -0 wc -l
rg -n '//|/\*' tooling/agent-handoff/src tooling/agent-handoff/tests
```

Résultat attendu : aucune dépendance vers Arnes ou `agent-memory`; aucun commentaire ajouté; aucune fonction de production au-dessus de 50 lignes logiques ni fichier manuscrit au-dessus de 250 lignes sans justification écrite dans la livraison.

- [ ] **Étape 6 : Commit**

```bash
git add .github/workflows/test-agent-handoff.yml .github/workflows/test-typescript.yml tooling/agent-handoff-ci.test.ts
git commit -m "ci(handoff): verify Rust parity on macOS and Linux"
```

### Tâche 7 : Revue finale de migration et preuve de non-régression

**Fichiers :**

- Vérifier : tous les fichiers modifiés depuis la base de ce plan
- Modifier : uniquement les fichiers nécessaires à corriger un défaut objectif découvert par la revue

**Interfaces :**

- Consomme : les six livrables précédents et la spec approuvée.
- Produit : une migration sans runtime Bun, sans domaine handoff dans Arnes, installable par Make, et prouvée sur chaque cible nommée par un oracle vert.

- [ ] **Étape 1 : Demander une revue fraîche du diff complet**

Le reviewer vérifie explicitement : couverture de chaque ligne de la matrice; égalité différentielle obtenue avant suppression; ordre des effets; atomicité `create_new`; préservation du sentinel en concurrence; diagnostics et codes exacts; absence d’écrasement d’une destination inattendue; retrait des chemins historiques dans les configs; absence de dépendance croisée; couverture réelle des extensions Rust, YAML, Make et TypeScript touchées.

- [ ] **Étape 2 : Corriger chaque défaut avec un RED ciblé**

Pour chaque défaut confirmé, ajouter d’abord le plus petit test qui échoue dans le crate, le test de déploiement ou le contrat CI propriétaire de l’invariant; exécuter ce test rouge, corriger, puis le repasser vert. Ne pas réintroduire Bun comme dépendance de production ou oracle permanent.

- [ ] **Étape 3 : Rejouer la barrière complète dans les environnements disponibles**

```bash
cargo fmt --manifest-path tooling/agent-handoff/Cargo.toml --check
cargo clippy --manifest-path tooling/agent-handoff/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tooling/agent-handoff/Cargo.toml
cargo fmt --manifest-path tooling/arnes/Cargo.toml --check
cargo clippy --manifest-path tooling/arnes/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tooling/arnes/Cargo.toml
bun test
bun run typecheck
make -n agent-handoff
make -n claude-code-hooks
make -n codex-hooks
git diff --check
```

Résultat attendu : toutes les commandes passent sur l’environnement nommé. Confirmer séparément les jobs GitHub Actions macOS et Linux avant d’affirmer la portabilité; un job absent, ignoré ou annulé n’est pas vert.

- [ ] **Étape 4 : Commit des corrections de revue, s’il y en a**

```bash
git add Makefile .github/workflows/test-agent-handoff.yml .github/workflows/test-typescript.yml tooling/agent-handoff tooling/agent-handoff-ci.test.ts tooling/deployment-codex-wiring.test.ts tooling/arnes/src/hooks.rs tooling/arnes/tests/hooks_setup.rs tooling/arnes/tests/hooks_validation.rs
git commit -m "fix(handoff): address migration review findings"
```

Ne créer ce commit que si la revue a produit un diff. La livraison liste les commentaires ajoutés et le fait externe qu’ils consignent; la liste attendue est vide.
