# Plan d’implémentation de la mémoire durable locale

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Livrer une mémoire locale durable, partagée et observable pour Codex, Claude Code et Cursor sans confier son domaine ni son état à Arnes.

**Architecture:** `tooling/agent-memory/` est un package Cargo autonome qui possède tout le domaine mémoire et ses adapters runtime. Arnes configure, valide et mesure uniquement les hooks et leurs exécutables; le `Makefile` déploie séparément `agent-memory` et le `agent-handoff` Rust livré par son propre plan, sans workspace ni crate partagé.

**Tech Stack:** Rust 2024, `clap`, `serde`/`serde_yaml_ng`, `sha2`, `rustix`, `jiff`, `unicode-normalization`, `url`, Git, curl, Bun 1.4, TypeScript 7, Make, YAML et JSON.

**Spec:** `docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md`

## Global Constraints

- Lire la spec, `AGENTS.md`, `harness/AGENTS.md`, `harness/USER.md`, l’index ADR, ADR-038, ADR-041, ADR-042 et la skill mémoire avant toute modification.
- Exécuter la tâche 1 de ce plan, puis le plan distinct `docs/superpowers/plans/2026-08-29-agent-handoff-rust.md` et sa revue avant de reprendre ici à la tâche 2. Il produit `tooling/agent-handoff/`, package Cargo autonome conservant exactement stdin, environnement, stdout, stderr et codes de sortie du runtime Bun.
- `agent-memory` et `agent-handoff` ont chacun manifest, lockfile, source, tests et binaire; aucun workspace, crate interne partagé ou dépendance mutuelle.
- Arnes ne dépend d’aucun de ces crates, ne contient aucun domaine mémoire/handoff et ne lit ni n’écrit leur état. Il configure, valide et mesure seulement hooks et exécutables.
- Les YAML sous `~/.local/share/agent-memory/` sont l’autorité; index et cache sont dérivés. Répertoires `0700`, fichiers `0600`; aucune donnée mémoire ou résultat brut n’entre dans Git.
- Scope `project` par défaut; scope `user` après autorisation explicite. Types fermés : `goal`, `decision`, `evidence`, `invariant`, `unknown`, `assumption`. Sources fermées : `git-file`, `local-file`, `official-url`, `user-decision`.
- Clé projet : `project_<sha256(realpath(git-common-dir))>`; hors Git, résultat ambigu/non absolu/non canonique : rejet. ID : `mem_<24 premiers hex de sha256(schema_version, kind, scope.key, statement normalisé)>`; document canonique identique : `duplicate`; même identité et autre contenu : `conflict`.
- Aucune commande shell dans un YAML; aucun commentaire de code. Diagnostics redacted sans statement, source brute, credential, prompt ou transcript.
- Une `official-url` est HTTPS sans credentials, IP littérale ni fragment, avec cinq redirections HTTPS maximum, corps 1 Mio, connexion 5 s et durée totale 15 s; son domaine exige une source `user-decision` co-présente.
- Publication : verrou global → préparation YAML/index → rename YAML → fsync répertoire → rename index → fsync. L’index ne pointe jamais vers un YAML absent; un index périmé est reconstruit au retrieval.
- Recherche locale lexicale déterministe, cinq injections maximum. Verdict `valid` consommable strictement moins de 48 h; changement local immédiatement invalidant.
- `invalid` produit seulement `invalidated`. Les terminaux métier `achieved`, `abandoned`, `superseded`, `resolved`, `confirmed` exigent une conclusion humaine `valid` compatible; une entrée ne transite qu’une fois.
- Demande explicite ou proposition acceptée écrit; détection implicite propose sans écrire. Aucun état généré propre à un agent n’est édité.
- Cœur portable prouvé sur macOS/Linux; agents réels seulement sur l’environnement nommé. Baseline : Codex `0.150.1`, Claude `2.1.250`, Cursor `3.15.6`; Cursor observé `2026.08.25-3e8eec8`, `Darwin arm64 26.6.2`.
- Utiliser `enforcement-code` pour les refus, `skill-manager fix memory-governance user`, puis `skill-manager sync-index user`.
- Aucune capacité n’est annoncée avant son oracle E2E vert.

## Structure cible

```text
tooling/
├── agent-handoff/                 # produit du plan handoff exécuté avant celui-ci
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   └── tests/
├── agent-memory/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── src/
│   │   ├── lib.rs                 # réexporte la façade memory
│   │   ├── main.rs
│   │   ├── cli.rs
│   │   ├── admission.rs
│   │   ├── hook.rs
│   │   ├── memory.rs              # façade du domaine déplacée depuis Arnes
│   │   └── memory/                # implémentation du domaine déplacée depuis Arnes
│   └── tests/
├── agent-memory-eval.ts
├── agent-memory-eval.test.ts
└── agent-memory-eval-scenarios.json
```

Arnes connaît les chemins absolus des deux exécutables, jamais leurs types, formats persistants ou invariants métier.

## Matrice des échecs à préserver

| Frontière       | Entrées et effet                                                                                                                                                        |
| --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Parsing         | YAML invalide, clé dupliquée, champ inconnu, version future, taille >1 Mio, Unicode/date invalide → `rejected`, aucune écriture                                         |
| Domaine         | kind/status/transition incohérent, terme vide, preuve/oracle absent, commande shell → `rejected`                                                                        |
| Confidentialité | token préfixé, clé privée, URL credential, champ secret, marqueur prompt/transcript → `rejected`, diagnostic redacted                                                   |
| Scope           | Git absent, common dir ambigu/non canonique, scope key appelant, user non autorisé, mismatch projet → `rejected`                                                        |
| Source locale   | absent, non régulier, symlink, illisible, hors dépôt, non suivi, trop grand → `invalid` si disparition prouvée, sinon `unavailable`                                     |
| URL             | schéma/host interdit ou redirect non HTTPS → refus; DNS/TLS/timeout/429/5xx/curl absent → `unavailable`; 404/410 ou fingerprint différent → `invalid`                   |
| Admission       | document identique → `duplicate`; identité occupée, source changée sous lock, lock expiré, concurrence divergente → `conflict`                                          |
| Persistance     | temp/flush/fsync/rename YAML échoué → rien de visible; index échoué après YAML → YAML conservé, index périmé et diagnostic                                              |
| Index           | absent/périmé/corrompu → reconstruction atomique; YAML invalide/futur → omission; reconstruction impossible → aucune injection                                          |
| Cache           | timestamp futur/malformé, verdict non-valid, source locale changée → miss; write cache échoué → verdict courant utilisable sans cache                                   |
| Retrieval       | requête vide, entrée disparue, oracle expiré indisponible/ambigu, transition impossible → omission, aucun ancien contexte                                               |
| Adapter         | stdin vide/malformé/surdimensionné, query/cwd absent, binaire inexécutable, sortie host impossible → aucune injection; indisponibilité annoncée quand le host le permet |

---

### Tâche 1 : corriger ADR-042 avant le code

**Files:**

- Modify: `docs/adr/042-memoire-durable-locale-partagee.md`
- Verify: `docs/adr/038-frontieres-home-harness-tooling.md`
- Verify: `docs/adr/041-frontiere-automatisation-typescript-rust.md`
- Verify: `docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md`

**Interfaces:**

- Consumes: spec approuvée au commit `7f58a03`.
- Produces: autorité attribuant domaine mémoire à `agent-memory`, runtime handoff à `agent-handoff`, et seulement configuration/validation/mesure à Arnes.

- [ ] **Step 1: écrire le RED documentaire**

```bash
rg -n 'Arnes pour l.automatisation|Arnes est l.unique frontière Rust|frontière Arnes concentre|second binaire ou crate' docs/adr/042-memoire-durable-locale-partagee.md
```

Expected: les trois affirmations obsolètes sont trouvées.

- [ ] **Step 2: corriger décision, conséquences et alternatives**

Corriger d'abord le contexte : ADR-041 impose Rust, pas Arnes, pour cette automatisation à état durable. Écrire explicitement : `agent-memory` possède schéma, admission, identité projet, sources, store, index, oracles, cache, retrieval, transitions, CLI et adapters runtime; `agent-handoff` possède son runtime; Arnes configure, valide et mesure leurs hooks/exécutables. Les packages sont indépendants sans workspace/crate partagé. Conserver inchangées les décisions fonctionnelles mémoire.

- [ ] **Step 3: vérifier puis committer**

```bash
prettier --check docs/adr/042-memoire-durable-locale-partagee.md docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md
! rg -n 'Arnes pour l.automatisation|Arnes est l.unique frontière Rust|frontière Arnes concentre|second binaire ou crate' docs/adr/042-memoire-durable-locale-partagee.md
git diff --check
git add docs/adr/042-memoire-durable-locale-partagee.md
git commit -m "docs(memory): correct runtime ownership"
```

### Tâche 2 : extraire le domaine dans `agent-memory`

**Files:**

- Create: `tooling/agent-memory/Cargo.toml`
- Create: `tooling/agent-memory/Cargo.lock`
- Create: `tooling/agent-memory/src/lib.rs`
- Move: `tooling/arnes/src/memory.rs` → `tooling/agent-memory/src/memory.rs`
- Move: `tooling/arnes/src/memory/` → `tooling/agent-memory/src/memory/`
- Move: `tooling/arnes/tests/memory_model.rs` → `tooling/agent-memory/tests/memory_model.rs`
- Move: `tooling/arnes/tests/memory_validation.rs` → `tooling/agent-memory/tests/memory_validation.rs`
- Move: `tooling/arnes/tests/memory_identity.rs` → `tooling/agent-memory/tests/memory_identity.rs`
- Move: `tooling/arnes/tests/memory_sources.rs` → `tooling/agent-memory/tests/memory_sources.rs`
- Move: `tooling/arnes/tests/memory_store.rs` and `tooling/arnes/tests/memory_store/` → `tooling/agent-memory/tests/`
- Move: `tooling/arnes/tests/memory_concurrency.rs` → `tooling/agent-memory/tests/memory_concurrency.rs`
- Move: `tooling/arnes/tests/memory_index.rs` and `tooling/arnes/tests/memory_index/` → `tooling/agent-memory/tests/`
- Move: `tooling/arnes/tests/memory_search.rs` and `tooling/arnes/tests/memory_search/` → `tooling/agent-memory/tests/`
- Move: `tooling/arnes/tests/memory_oracle.rs` → `tooling/agent-memory/tests/memory_oracle.rs`
- Move: `tooling/arnes/tests/memory_cache.rs` → `tooling/agent-memory/tests/memory_cache.rs`
- Move: `tooling/arnes/tests/memory_retrieval.rs` → `tooling/agent-memory/tests/memory_retrieval.rs`
- Move: `tooling/arnes/tests/memory_task6/` → `tooling/agent-memory/tests/memory_task6/`
- Move: `tooling/arnes/tests/support/memory.rs` → `tooling/agent-memory/tests/support/memory.rs`
- Modify: `tooling/arnes/src/lib.rs`
- Modify: `tooling/arnes/Cargo.toml`
- Modify: `tooling/arnes/Cargo.lock`

**Interfaces:**

- Consumes: code commité des tâches 2–5 jusqu’à `4f0c843` et WIP Task 6 dans les chemins Arnes.
- Produces: crate importable `agent_memory` avec mêmes exports publics; WIP déplacé et réutilisé; Arnes sans module/dépendance mémoire.

- [ ] **Step 1: inventorier le WIP sans le restaurer**

```bash
git status --short
git diff -- tooling/arnes/Cargo.toml tooling/arnes/Cargo.lock tooling/arnes/src/memory.rs tooling/arnes/src/memory tooling/arnes/tests > /tmp/pr-249-task6.patch
{
  git diff --name-only -z -- tooling/arnes/Cargo.toml tooling/arnes/Cargo.lock tooling/arnes/src/memory.rs tooling/arnes/src/memory tooling/arnes/tests
  git ls-files --others --exclude-standard -z -- tooling/arnes/src/memory tooling/arnes/tests
} | sort -zu | xargs -0 shasum -a 256 > /tmp/pr-249-task6-files.sha256
```

Expected: le WIP Task 6 est inventorié; aucun reset, checkout ou suppression.

- [ ] **Step 2: écrire le test RED de crate**

Créer `tooling/agent-memory/tests/crate_boundary.rs` :

```rust
use agent_memory::{MemoryRoot, parse_draft};

#[test]
fn domain_is_exported_by_agent_memory() {
    let _parse = parse_draft;
    let _root = MemoryRoot::new;
}
```

Run: `cargo test --manifest-path tooling/agent-memory/Cargo.toml --test crate_boundary`.
Expected: FAIL, crate absent.

- [ ] **Step 3: déplacer code et tests avec leur état de travail**

Créer le package édition 2024 avec `getrandom`, `jiff`, `rustix`, `serde`, `serde_path_to_error`, `serde_json`, `serde_yaml_ng`, `sha2`, `tempfile`, `url`, `unicode-normalization`. `src/lib.rs` contient exactement `mod memory; pub use memory::*;`; la façade déplacée reste `src/memory.rs`, afin que ses sous-modules continuent de résoudre sous `src/memory/`. Remplacer `arnes::memory` par `agent_memory` dans les tests, retirer `pub mod memory` d’Arnes et seulement ses dépendances devenues inutilisées. Régénérer les deux lockfiles par leur manifest; ne créer ni `Cargo.toml` racine ni `tooling/Cargo.toml`.

- [ ] **Step 4: prouver séparation et conservation**

```bash
cargo fmt --manifest-path tooling/agent-memory/Cargo.toml --check
cargo clippy --manifest-path tooling/agent-memory/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tooling/agent-memory/Cargo.toml --test memory_model --test memory_validation --test memory_identity --test memory_sources --test memory_store --test memory_concurrency --test memory_index --test memory_search
cargo test --manifest-path tooling/arnes/Cargo.toml
! rg -n 'pub mod memory|arnes::memory|agent_memory|agent-memory|ARNES_MEMORY_ROOT' tooling/arnes/src tooling/arnes/tests tooling/arnes/Cargo.toml
test ! -e Cargo.toml && test ! -e tooling/Cargo.toml
test -f tooling/agent-memory/src/memory/cache.rs
test -f tooling/agent-memory/tests/memory_cache.rs
git diff --find-renames=50% --summary 4f0c843.. -- tooling/arnes tooling/agent-memory
```

Expected: tâches 2–5 vertes depuis le nouveau manifest; WIP Task 6 présent; aucun double domaine.

- [ ] **Step 5: commit**

```bash
git add tooling/agent-memory tooling/arnes/src/lib.rs tooling/arnes/Cargo.toml tooling/arnes/Cargo.lock tooling/arnes/tests
git commit -m "refactor(memory): isolate the memory runtime"
```

### Tâche 3 : terminer oracles, cache et retrieval depuis le WIP

**Files:**

- Modify: `tooling/agent-memory/src/lib.rs`
- Modify: `tooling/agent-memory/src/memory/clock.rs`
- Modify: `tooling/agent-memory/src/memory/oracle.rs`
- Modify: `tooling/agent-memory/src/memory/cache.rs`
- Modify: `tooling/agent-memory/src/memory/retrieval.rs`
- Modify: `tooling/agent-memory/src/memory/retrieval/operation.rs`
- Modify: `tooling/agent-memory/src/memory/retrieval/transition.rs`
- Modify: `tooling/agent-memory/src/memory/store.rs`
- Modify: `tooling/agent-memory/src/memory/store/retrieval.rs`
- Test: `tooling/agent-memory/tests/memory_oracle.rs`
- Test: `tooling/agent-memory/tests/memory_cache.rs`
- Test: `tooling/agent-memory/tests/memory_retrieval.rs`
- Test: `tooling/agent-memory/tests/memory_task6/`

**Interfaces:**

- Consumes: `Store`, `Index`, `search`, `Clock`, `SourceResolver` et WIP déplacé.
- Produces: `evaluate_oracle(entry: &MemoryEntry, context: OracleContext<'_>) -> OracleEvaluation`; `retrieve(request: RetrievalRequest<'_>, context: RetrievalContext<'_>) -> RetrievalReport`; `confirm(id: &str, conclusion: HumanConclusion, context: TransitionContext<'_>) -> Result<TransitionResult, MemoryError>`.

- [ ] **Step 1: obtenir le RED réel**

```bash
cargo test --manifest-path tooling/agent-memory/Cargo.toml --test memory_oracle --test memory_cache --test memory_retrieval
```

Expected: premier contrat WIP incomplet en échec. Si tout passe déjà, ajouter d’abord un cas manquant de Step 2 et observer son échec.

- [ ] **Step 2: fermer les matrices par tests**

Tester hit `47:59:59.999`, miss `48:00:00.000`, timestamp futur, changement local, URL non refetchée avant 48 h, URL expirée indisponible, fallback humain et aucun cache non-valid. Tester les cinq constructeurs `goal_achieved`, `goal_abandoned`, `decision_superseded`, `unknown_resolved`, `assumption_confirmed`, plus terminal incompatible, seconde transition, raison vide, fingerprint différent vers `invalidated`, YAML byte-identique sur `unavailable`/`needs_confirmation`.

- [ ] **Step 3: achever cache/oracle/retrieval**

Cache v1 : records triés par `entry_id`; digest canonique de l’oracle déclaratif; `proof_digest = sha256(JSON canonique compact des sources ordonnées {kind,locator,fingerprint})`; fingerprints `{kind,fingerprint}` dans l’ordre YAML; `validated_at`; environnement `{os,arch}`; seulement `verdict: valid`; aucun locator/réponse humaine en clair. Un hit exige le même `proof_digest`; refingerprint local avant hit; URL hit avant 48 h; échec write cache sans invalider le verdict courant.

`HumanConclusion` reste une union fermée `GoalAchieved | GoalAbandoned | DecisionSuperseded | UnknownResolved | AssumptionConfirmed`, chaque variante portant sa raison. La validation du fallback humain utilise une réponse `ProofValid` distincte, qui peut produire un verdict cacheable mais jamais une transition YAML; `confirm` n'accepte que les cinq conclusions métier.

Retrieval : charger/reparser seulement cinq YAML sélectionnés, évaluer l’oracle, persister atomiquement une invalidation certaine, puis rendre :

```rust
pub struct RetrievalReport {
    pub injected: Vec<InjectedMemory>,
    pub omitted: Vec<OmittedMemory>,
    pub omitted_by_limit: usize,
}
```

Résumé source : `git-file` relatif; `official-url` sans query/fragment; `local-file` et `user-decision` sans locator. Omission : id, code, question éventuelle, `NotApplied`.

- [ ] **Step 4: GREEN et commit**

```bash
cargo fmt --manifest-path tooling/agent-memory/Cargo.toml --check
cargo clippy --manifest-path tooling/agent-memory/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tooling/agent-memory/Cargo.toml
git add tooling/agent-memory
git commit -m "feat(memory): verify freshness before retrieval"
```

### Tâche 4 : admission et CLI `agent-memory`

**Files:**

- Create: `tooling/agent-memory/src/main.rs`
- Create: `tooling/agent-memory/src/cli.rs`
- Create: `tooling/agent-memory/src/admission.rs`
- Create: `tooling/agent-memory/tests/memory_admission.rs`
- Create: `tooling/agent-memory/tests/memory_cli.rs`
- Modify: `tooling/agent-memory/src/lib.rs`
- Modify: `tooling/agent-memory/Cargo.toml`
- Modify: `tooling/agent-memory/Cargo.lock`

**Interfaces:**

- Consumes: YAML/query stdin, cwd, ID, terminal, raison; services du domaine.
- Produces: `agent-memory admit|retrieve|confirm|audit|hook`; JSON stdout; diagnostic redacted stderr; exit `0` succès/duplicate, `2` usage/rejet, `3` conflit, `4` indisponibilité.

- [ ] **Step 1: écrire les RED admission et CLI**

```rust
pub struct AdmissionContext<'a> {
    pub store: &'a Store,
    pub cwd: &'a Path,
    pub clock: &'a dyn Clock,
    pub processes: &'a dyn ProcessRunner,
    pub authorization: AdmissionAuthorization,
}

pub fn admit(bytes: &[u8], context: AdmissionContext<'_>)
    -> Result<AdmissionResult, MemoryError>;
```

Prouver parsing → validation → scope → sources → oracle → duplicate → commit; projet défaut, autorisation user, source changée entre résolution/lock, retry post-YAML, concurrence. Tester exactement :

```bash
printf '%s' "$draft" | agent-memory admit --format json
printf '%s' "$query" | agent-memory retrieve --query-stdin --format json
printf '%s' "$reason" | agent-memory confirm --id "$id" --status confirmed --reason-stdin
agent-memory audit --include-terminal --format json
```

Tester stdin vide/>1 Mio, option inconnue, refus user et redaction.

- [ ] **Step 2: implémenter la frontière**

`admit` remplit ID/scope/fingerprints/timestamps. `retrieve` résout cwd et scopes projet+user. `confirm` accepte uniquement terminal humain compatible. `audit` ne répare ni réactive. Root : `AGENT_MEMORY_ROOT` ou `~/.local/share/agent-memory`; jamais `ARNES_MEMORY_ROOT`.

- [ ] **Step 3: GREEN et commit**

```bash
cargo test --manifest-path tooling/agent-memory/Cargo.toml --test memory_admission --test memory_cli
cargo test --manifest-path tooling/agent-memory/Cargo.toml
! rg -n 'arnes|ARNES_MEMORY_ROOT' tooling/agent-memory/src tooling/agent-memory/tests
git add tooling/agent-memory
git commit -m "feat(memory): expose admission and retrieval commands"
```

### Tâche 5 : adapters runtime Codex et Claude

**Files:**

- Create: `tooling/agent-memory/src/hook.rs`
- Create: `tooling/agent-memory/src/hook/codex.rs`
- Create: `tooling/agent-memory/src/hook/claude.rs`
- Create: `tooling/agent-memory/tests/memory_hooks.rs`
- Create: `tooling/agent-memory/tests/fixtures/hooks/codex-minimal.json`
- Create: `tooling/agent-memory/tests/fixtures/hooks/codex-complete.json`
- Create: `tooling/agent-memory/tests/fixtures/hooks/claude-minimal.json`
- Create: `tooling/agent-memory/tests/fixtures/hooks/claude-complete.json`
- Modify: `tooling/agent-memory/src/cli.rs`
- Modify: `tooling/agent-memory/src/main.rs`

**Interfaces:**

- Consumes: payload `UserPromptSubmit` stdin et `HookAgent::Codex | HookAgent::Claude`.
- Produces: `agent-memory hook --agent codex|claude`, réponse host `additionalContext`; aucune variante Cursor.

- [ ] **Step 1: écrire les RED protocoles et snapshots**

```rust
pub enum HookAgent { Codex, Claude }
pub struct HookRequest { pub query: String, pub cwd: PathBuf }
pub fn parse_hook_request(agent: HookAgent, bytes: &[u8]) -> Result<HookRequest, HookError>;
pub fn render_hook_response(agent: HookAgent, report: &RetrievalReport) -> Result<Vec<u8>, HookError>;
```

Tester query/cwd, mauvais événement, stdin vide/>1 Mio, champ absent, aucun résultat, oracle indisponible, limite cinq. Contexte `AGENT_MEMORY_CONTEXT_V1`, injections avec kind/statement/source/âge, omissions distinctes; jamais index/cache/path store/source brute.

- [ ] **Step 2: implémenter sans repli Arnes**

Lire stdin une fois, borner avant parsing, appeler le domaine, produire réponse host. Payload invalide sort `2`; store/oracle indisponible sort `4`; aucune erreur ne restitue d’ancien contexte.

- [ ] **Step 3: GREEN et commit**

```bash
cargo test --manifest-path tooling/agent-memory/Cargo.toml --test memory_hooks
cargo test --manifest-path tooling/agent-memory/Cargo.toml
git add tooling/agent-memory
git commit -m "feat(memory): adapt supported prompt hooks"
```

### Tâche 6 : configurer les hooks depuis Arnes

**Files:**

- Modify: `tooling/arnes/src/manifest/hooks.rs`
- Modify: `tooling/arnes/src/hooks.rs`
- Modify: `tooling/arnes/src/hooks/adapters.rs`
- Modify: `tooling/arnes/src/hooks/reconcile.rs`
- Modify: `tooling/arnes/tests/hooks_setup.rs`
- Modify: `tooling/arnes/tests/hooks_reconciliation/installation.rs`
- Modify: `tooling/arnes/tests/hooks_reconciliation/ownership.rs`
- Create: `tooling/arnes/tests/runtime_boundaries.rs`

**Interfaces:**

- Consumes: `HookKind::Memory`, `~/.local/bin/agent-memory`, `~/.local/bin/agent-handoff`.
- Produces: hooks Codex/Claude vers le binaire mémoire; Stop vers handoff; validation fichier régulier/exécutable et ownership de configuration.

- [ ] **Step 1: écrire les RED de réconciliation et frontière**

Tester événements par kind, ordre handlers tiers, suppression owned, chemin absolu, absent/non-régulier/non-exécutable, config invalide, concurrence, idempotence; aucun handler mémoire Cursor. Commandes après résolution home :

```text
'/absolute/home/.local/bin/agent-memory' hook --agent codex
'/absolute/home/.local/bin/agent-memory' hook --agent claude
```

`runtime_boundaries.rs` échoue si Arnes dépend des crates, possède `src/memory.rs`, lit `AGENT_MEMORY_ROOT`/`.local/share/agent-memory`, ou nomme `MemoryEntry`/`RetrievalReport`.

- [ ] **Step 2: implémenter uniquement config/validation/mesure**

Arnes ne parse pas le prompt, ne rend pas `additionalContext`, ne mappe pas les erreurs mémoire. Sa mesure observe exécution/durée/exit sans lire stdin privé ni état mémoire.

- [ ] **Step 3: GREEN et commit**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test hooks_setup --test hooks_reconciliation --test runtime_boundaries
cargo test --manifest-path tooling/arnes/Cargo.toml
git add tooling/arnes/src/manifest/hooks.rs tooling/arnes/src/hooks.rs tooling/arnes/src/hooks tooling/arnes/tests
git commit -m "feat(arnes): configure the memory runtime hook"
```

### Tâche 7 : skill, règle Cursor et déploiement

**Files:**

- Modify: `harness/skills/memory-governance/SKILL.md`
- Create: `harness/skills/memory-governance/references/entry-contract.md`
- Modify: `harness/skills/memory-governance/evals/trigger-queries.json`
- Modify generated: `harness/skills/README.md`
- Create: `harness/rules/memory-governance-cursor.mdc`
- Modify: `home/.arnes.yaml`
- Modify: `Makefile`
- Modify: `tooling/deployment-links.test.ts`
- Modify: `tooling/deployment-codex-wiring.test.ts`
- Modify: `tooling/arnes/src/rules.rs`
- Modify: `tooling/arnes/tests/rules.rs`
- Modify: `tooling/arnes/tests/manifest_rules.rs`

**Interfaces:**

- Consumes: CLI/adapters mémoire, config Arnes, cible handoff préexistante.
- Produces: binaire mémoire déployé séparément; workflow partagé; skill trois agents; hooks Codex/Claude; règle Cursor.

- [ ] **Step 1: écrire le RED déploiement**

Exiger liens skill trois agents, règle Cursor, hook mémoire Codex/Claude seulement et binaires séparés sous `~/.local/bin`.

Run: `bun test tooling/deployment-links.test.ts tooling/deployment-codex-wiring.test.ts`.
Expected: FAIL sur cibles/déclarations absentes.

- [ ] **Step 2: migrer skill et règle**

Exécuter `skill-manager fix memory-governance user`. Triggers : demande, acceptation, détection durable, début tâche Cursor. Body : retrieval avant travail Cursor, annonce, proposition sans écriture, admission autorisée, confirmation, refus redacted. Référence : schéma, draft complet, commandes `agent-memory`. Supprimer toute commande `arnes memory`/`ARNES_MEMORY_ROOT`.

Règle Cursor `alwaysApply: true` : charger la skill, fournir prompt, attendre `agent-memory retrieve`, suivre le résultat; en échec annoncer indisponibilité et ne rien appliquer. Aucun schéma/ranking/politique dupliqué.

- [ ] **Step 3: déclarer et déployer**

Manifeste : skill trois agents, hook mémoire Codex/Claude, règle user Cursor. Make : cible `agent-memory` buildant son manifest et liant son binaire; conserver la cible handoff déjà livrée. Les cibles agents dépendent des exécutables avant setup.

- [ ] **Step 4: vérifier puis commit**

```bash
skill-manager doctor memory-governance user
skill-manager sync-index user
shasum -a 256 harness/skills/README.md > /tmp/memory-index.sha256
skill-manager sync-index user
shasum -a 256 -c /tmp/memory-index.sha256
make -n agent-memory agent-handoff codex claude-code cursor
bun test tooling/deployment-links.test.ts tooling/deployment-codex-wiring.test.ts
cargo test --manifest-path tooling/arnes/Cargo.toml --test rules --test manifest_rules
git add harness/skills/memory-governance harness/skills/README.md harness/rules/memory-governance-cursor.mdc home/.arnes.yaml Makefile tooling/deployment-links.test.ts tooling/deployment-codex-wiring.test.ts tooling/arnes/src/rules.rs tooling/arnes/tests/rules.rs tooling/arnes/tests/manifest_rules.rs
git commit -m "feat(memory): deploy the runtime to every agent"
```

### Tâche 8 : oracle end-to-end multi-agent

**Files:**

- Create: `tooling/agent-memory-eval.ts`
- Create: `tooling/agent-memory-eval.test.ts`
- Create: `tooling/agent-memory-eval-scenarios.json`
- Replace: `docs/memory-governance-validation.md`

**Interfaces:**

- Consumes: agents résolus, adapters déployés dans homes temporaires, `agent-memory`, fixture Git et `AGENT_MEMORY_ROOT` temporaires.
- Produces: raw hors Git et rapport normalisé; preuve `agent → adapter configuré → agent-memory → adapter → agent`.

- [ ] **Step 1: écrire les RED runner**

Faux agents/binaire : process frais, timeout 120 s, exit non nul, JSONL malformé, nonce absent, appel binaire absent, ordre incorrect, mutation contrôle, store personnel, version absente, cleanup interrompu. Refuser un store hors racine temporaire.

- [ ] **Step 2: implémenter quatre scénarios, contrôle/déployé, trois réplicats**

1. détection durable → proposition → aucune écriture;
2. acceptation → `stored` → session pertinente avec preuve/fraîcheur → session sans rapport sans injection;
3. secret/transcript → `rejected` → store inchangé;
4. source indisponible omise sans mutation → contradiction vers `invalidated`.

Chaque réplicat a store, home, dépôt et processus distincts.

- [ ] **Step 3: implémenter les oracles agents**

- Codex : `UserPromptSubmit` → fin `agent-memory` → `additionalContext` → premier événement modèle avec nonce.
- Claude : même ordre via `--include-hook-events`.
- Cursor : lecture règle/skill → fin `agent-memory retrieve` → première analyse/action avec nonce; preuve comportementale versionnée.
- Tous : permissions, YAML, index, cache, absence mutation implicite et absence lecture store personnel.

- [ ] **Step 4: tests, runs réels et rapport**

```bash
bun test tooling/agent-memory-eval.test.ts
bun run typecheck && bun run lint && bun run format:typescript:check
bun tooling/agent-memory-eval.ts --agent codex --replicates 3
bun tooling/agent-memory-eval.ts --agent claude --replicates 3
bun tooling/agent-memory-eval.ts --agent cursor --replicates 3
```

Rapport : date, OS/arch, versions, SHA binaire, commandes, résultats par capacité/agent, limites Cursor/Linux, nettoyage. Une capacité sous `3/3` bloque la PR sans dégradation/report.

- [ ] **Step 5: commit**

```bash
git add tooling/agent-memory-eval.ts tooling/agent-memory-eval.test.ts tooling/agent-memory-eval-scenarios.json docs/memory-governance-validation.md
git commit -m "test(memory): prove the adapter runtime path"
```

### Tâche 9 : CI portable et vérification finale

**Files:**

- Create: `.github/workflows/test-agent-memory.yml`
- Modify: `.github/workflows/test-arnes.yml`
- Modify: `.github/workflows/lint.yml`
- Create: `tooling/agent-memory-workflow.test.ts`

**Interfaces:**

- Consumes: implémentation/rapport; workflow handoff livré par son plan.
- Produces: gates indépendantes mémoire/handoff/Arnes macOS+Linux; head exact revu.

- [ ] **Step 1: écrire le RED CI puis séparer les workflows**

Le contract test exige `ubuntu-latest`/`macos-latest` et :

```bash
cargo fmt --manifest-path tooling/agent-memory/Cargo.toml --check
cargo clippy --manifest-path tooling/agent-memory/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tooling/agent-memory/Cargo.toml
```

`test-arnes.yml` reste indépendant; workflow handoff inchangé; aucun `--workspace`. Le lint couvre nouveaux Markdown/YAML/JSON/TS.

- [ ] **Step 2: exécuter toutes les barrières**

```bash
for manifest in tooling/agent-memory/Cargo.toml tooling/agent-handoff/Cargo.toml tooling/arnes/Cargo.toml; do
  cargo fmt --manifest-path "$manifest" --check
  cargo clippy --manifest-path "$manifest" --all-targets -- -D warnings
  cargo test --manifest-path "$manifest"
done
bun test
bun run lint && bun run typecheck && bun run format:typescript:check
prettier --check docs/adr/042-memoire-durable-locale-partagee.md docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md docs/memory-governance-validation.md home/.arnes.yaml harness/skills/memory-governance/SKILL.md harness/skills/memory-governance/references/entry-contract.md harness/skills/memory-governance/evals/trigger-queries.json harness/rules/memory-governance-cursor.mdc tooling/agent-memory-eval-scenarios.json
git diff --check 7f58a03..HEAD
```

Expected: PASS local macOS; CI répète les trois crates sur macOS/Linux.

- [ ] **Step 3: vérifier frontières, tailles et commentaires**

```bash
! rg -n 'agent_memory|AGENT_MEMORY_ROOT|MemoryEntry|RetrievalReport|\.local/share/agent-memory' tooling/arnes/src tooling/arnes/Cargo.toml
test ! -e Cargo.toml && test ! -e tooling/Cargo.toml
rg -n '^\s*(//|/\*|\*)' tooling/agent-memory/src tooling/agent-handoff/src tooling/agent-memory-eval.ts || true
git status --short
```

Inspecter fonctions production >50 lignes logiques et fichiers manuscrits >250 lignes; scinder ou justifier. Expected: aucun commentaire ajouté, donnée mémoire ou raw suivi.

- [ ] **Step 4: commit, oracles head exact et revue**

```bash
git add .github/workflows/test-agent-memory.yml .github/workflows/test-arnes.yml .github/workflows/lint.yml
git commit -m "ci(memory): verify independent agent runtimes"
```

Rejouer Step 2 et les trois runs Task 8; actualiser le SHA du rapport si binaire/adapters changent. Utiliser `superpowers:requesting-code-review`, puis `enforcement-code` sur traversal, symlink, TOCTOU, collision, lock, URL/redirect, secrets, scope user, hook absent et couplage Arnes. Toute correction rejoue test ciblé, trois suites Cargo et oracles affectés.

## Self-review du plan

- Tâche 1 corrige l’autorité avant tout code; le plan handoff s’exécute ensuite, avant la tâche 2.
- Tâche 2 extrait les tâches 2–5 et déplace le WIP Task 6; tâches 3–5 terminent seulement `agent-memory`; tâche 6 borne Arnes.
- Modèle/refus, sources/scope, store/atomicité, index/recherche, cache/oracles/transitions, admission/CLI et adapters sont couverts.
- Tâche 7 déploie les surfaces; tâche 8 prouve le chemin complet en `3/3`; tâche 9 couvre macOS/Linux.
- Le package handoff et sa parité stricte sont un prérequis d’un plan distinct, non dupliqué.
- Aucun placeholder, capacité différée, workspace, état agent généré, commentaire de code ou dépendance crate vers Arnes.
