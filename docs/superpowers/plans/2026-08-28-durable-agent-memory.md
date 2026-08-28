# Plan d’implémentation de la mémoire durable locale

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Livrer dans la PR 249 une mémoire locale durable, partagée et observable pour Codex, Claude Code et Cursor, avec admission, propositions automatiques, YAML hors Git, index dérivé, oracles et cache de fraîcheur de 48 h.

**Architecture:** Étendre Arnes, déjà responsable du harnais et de ses hooks, avec un domaine Rust fermé qui possède validation, identité projet, sources, stockage, index, oracles et retrieval. Codex et Claude reçoivent le contexte par leur hook synchrone `UserPromptSubmit`; Cursor, dont le hook pré-prompt ne sait pas injecter, reçoit une règle user minimale qui impose l’appel de la même CLI avant tout travail, puis une évaluation comportementale bloque toute promesse non prouvée.

**Tech Stack:** Rust 2024, `serde`/`serde_yaml_ng`, `sha2`, `rustix`, `jiff`, `unicode-normalization`, `url`, Git, curl, Bun 1.4, TypeScript 7, Make, YAML et JSON.

**Spec:** `docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md`

## Global Constraints

- Les YAML sous `~/.local/share/agent-memory/` sont l’unique autorité; index et cache restent dérivés, supprimables et reconstruisibles.
- Le store et ses répertoires sont `0700`; chaque YAML, index, cache, verrou et fichier temporaire est `0600`.
- Aucun YAML, index, cache, résultat brut d’agent, prompt privé ou transcript n’entre dans Git.
- Les scopes persistés sont `project` par défaut et `user` seulement après autorisation explicite; deux worktrees d’un même Git partagent la même clé projet.
- Les six types restent fermés à `goal`, `decision`, `evidence`, `invariant`, `unknown` et `assumption`; les statuts autorisés restent exactement ceux de la spec.
- Les sources initiales restent fermées à `git-file`, `local-file`, `official-url` et `user-decision`; aucune commande shell n’est persistée ou exécutée depuis un YAML.
- La recherche reste locale, lexicale et déterministe, sans embedding, service ni similarité implicite; elle injecte au plus cinq entrées et annonce le nombre de résultats écartés.
- Un verdict `valid` est réutilisable strictement moins de 48 heures; à exactement 48 heures il est expiré, et tout changement local observable invalide immédiatement le cache.
- Une demande explicite ou l’acceptation d’une proposition peut écrire; une détection implicite propose sans jamais écrire.
- `~/.codex/memories/` et tout état généré propre à un agent restent intouchables.
- Les diagnostics contiennent identifiant, code du contrôle et effet, jamais statement complet, contenu source, credential, prompt ou transcript.
- Le cœur portable est prouvé sur macOS et Linux; les agents réels sont évalués sur l’environnement nommé et ne valent pas pour une autre version ou plateforme.
- Baseline de la spec : Codex CLI `0.150.1`, Claude Code `2.1.250`, Cursor `3.15.6`; version Cursor réellement observée pendant la discovery : `2026.08.25-3e8eec8` sur `Darwin arm64 26.6.2`.
- Utiliser `enforcement-code` pour chaque refus, `skill-manager fix memory-governance user` pour la skill, puis `skill-manager sync-index user`; ne pas ajouter de commentaire de code.
- Ne publier aucune promesse de persistance, injection, proposition automatique, isolation ou fraîcheur avant le GREEN de son oracle end-to-end nommé.

## Décisions fermées par la discovery

- Ajouter l’ADR-042 avant le code : la spec approuvée est une exigence, mais `docs/adr/` reste l’autorité des décisions structurelles en vigueur.
- Conserver un seul binaire `arnes`. Un second crate ou binaire dupliquerait racines, installation, hooks et diagnostics sans frontière métier indépendante.
- Calculer `scope.key` comme `project_<sha256(realpath(git-common-dir))>`. Les worktrees convergent; deux clones et un dépôt déplacé restent volontairement distincts. Hors dépôt Git, `realpath` impossible ou résultat non absolu : admission projet refusée.
- Générer `id` comme `mem_<24 premiers hex de sha256(schema_version, kind, scope.key, statement normalisé)>`. Même identité et document canonique identique : `duplicate`; même identité et autre contenu : `conflict`.
- Un oracle automatisé `source-fingerprint` compare toutes les empreintes de preuve. Il produit `valid`, `invalid` ou `unavailable`; `invalid` entraîne uniquement `invalidated`. Les statuts `achieved`, `abandoned`, `superseded`, `resolved` et `confirmed` exigent une conclusion humaine `valid` explicitement typée pour le `kind`.
- Une entrée ne transite qu’une fois de `active` vers un statut terminal et n’est jamais réactivée; l’objet `transition` unique est donc l’historique complet autorisé dans cette version.
- L’ordre de commit est verrou global → préparation YAML et index → rename YAML → fsync répertoire → rename index → fsync. Il empêche un index vers un YAML absent; un crash après le YAML laisse seulement un index périmé, détecté et reconstruit au retrieval.
- Codex et Claude utilisent `UserPromptSubmit` avec `additionalContext`. Cursor n’a aucune injection query-specific native avant modèle : sa règle user `alwaysApply` ne garantit que la présence de l’instruction; la capacité n’est annoncée que si la skill exécute effectivement le retrieval avant influence dans `3/3` processus frais.
- Le contrôle de contenu sensible est une barrière déterministe pour les formes nommées et une heuristique advisory pour un prompt privé ou transcript non marqué. La skill et l’E2E complètent cette limite; aucune détection universelle n’est revendiquée.
- `official-url` exige HTTPS sans credentials, IP littérale ni fragment, cinq redirections HTTPS maximum, 1 Mio maximum, connexion 5 s et durée totale 15 s. L’officialité du domaine est une décision utilisateur persistée comme source `user-decision`; le fetch ne prétend pas l’inférer.

## Matrice des échecs

| Frontière       | Échecs fermés et effet                                                                                                                                                            |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Parsing         | YAML invalide, clé dupliquée, champ inconnu, version future, taille > 1 Mio, Unicode ou date invalide → `rejected`, aucune écriture                                               |
| Domaine         | type/statut/transition incohérent, termes vides, preuve ou oracle absent, commande arbitraire → `rejected`                                                                        |
| Confidentialité | token préfixé connu, clé privée, URL avec credentials, champ secret, marqueur de prompt/transcript → `rejected`; diagnostic redacted                                              |
| Scope           | Git absent, common dir ambigu/non canonique, scope key fourni par l’appelant, user scope non autorisé, mismatch projet → `rejected`                                               |
| Source locale   | absent, non régulier, symlink, illisible, hors dépôt pour `git-file`, non suivi, taille excessive → `invalid` si disparition prouvée, sinon `unavailable`                         |
| URL             | schéma/host interdit, redirection non HTTPS, DNS/TLS/timeout/429/5xx, corps excessif, curl absent → `unavailable`; 404/410 ou empreinte différente → `invalid`                    |
| Admission       | doublon exact → `duplicate`; identité occupée, source changée pendant validation, lock après délai ou concurrence divergente → `conflict`                                         |
| Persistance     | permissions, temp, flush, fsync ou rename YAML échoué → aucune entrée visible; index échoué après commit YAML → entrée autoritaire conservée, index périmé et diagnostic          |
| Index           | absent/périmé/corrompu → reconstruction atomique; YAML invalide/futur → omission diagnostiquée; reconstruction impossible → aucun résultat injecté                                |
| Cache           | timestamp futur/malformé, verdict non-valid, source locale changée → miss; écriture cache impossible → verdict courant utilisable sans cache                                      |
| Retrieval       | requête vide, entrée disparue après sélection, oracle expiré indisponible/ambigu, transition impossible → omission et diagnostic, jamais d’ancien contexte                        |
| Adapter         | stdin vide/malformé/surdimensionné, query/cwd absent, binaire non exécutable, sortie host impossible → aucune injection; l’agent reçoit l’indisponibilité quand le host le permet |

---

### Tâche 1 : autorité architecturale et contrat exécutable

**Fichiers :**

- Créer : `docs/adr/042-memoire-durable-locale-partagee.md`
- Modifier : `docs/adr/README.md`
- Modifier : `docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md`

**Interfaces :**

- Consomme : spec approuvée au commit `25860b4`, ADR-025, ADR-036, ADR-038, ADR-040, ADR-041 et documentation officielle des hooks vérifiée le 2026-08-28.
- Produit : autorité en vigueur pour les décisions listées dans « Décisions fermées par la discovery » et contrats non ambigus pour les tâches suivantes.

- [ ] **Step 1: Écrire le test documentaire RED**

Exécuter :

```bash
test -f docs/adr/042-memoire-durable-locale-partagee.md
```

Résultat attendu : FAIL, fichier absent.

- [ ] **Step 2: Écrire ADR-042**

L’ADR contient exactement ces décisions : YAML autoritaire hors Git; Arnes comme frontière Rust; clé projet par `git-common-dir` canonique; ID déterministe; commit YAML avant index; verdict `invalid` vers `invalidated`; conclusions humaines typées; hooks Codex/Claude; règle + skill mesurée pour Cursor; limites de la détection sensible et de l’officialité URL; cache `< 48 h`; aucun état agent généré édité.

Inclure dans « Alternatives écartées » : second binaire, SQLite, MCP/embeddings, règle globale partagée non mesurée, wrapper Cursor limité au CLI, écriture dans `~/.codex/memories/`.

- [ ] **Step 3: Fermer le schéma dans la spec**

Ajouter les unions suivantes sans changer les capacités demandées :

```yaml
proof:
  sources:
    - kind: git-file | local-file | official-url | user-decision
      locator: <forme propre au kind>
      fingerprint: sha256:<64 hex>
oracle:
  automated:
    kind: source-fingerprint
    expected: all-proof-sources-unchanged
  human_fallback:
    question: <question>
    valid_when: <réponse observable>
transition:
  from: active
  to: <statut terminal autorisé par kind>
  at: <RFC 3339 UTC>
  verdict: valid | invalid
  reason: <texte concis>
```

Préciser que `automated` peut être absent seulement pour `user-decision`, que `human_fallback` reste obligatoire, que `invalid` produit `invalidated` et qu’un terminal métier exige `valid`.

- [ ] **Step 4: Vérifier format et autorité**

Exécuter :

```bash
prettier --check docs/adr/042-memoire-durable-locale-partagee.md \
  docs/adr/README.md \
  docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md
rg -n '042-memoire-durable-locale-partagee' docs/adr/README.md
git diff --check
```

Résultat attendu : PASS; l’ADR apparaît une fois dans l’index.

- [ ] **Step 5: Commit**

```bash
git add docs/adr/042-memoire-durable-locale-partagee.md docs/adr/README.md \
  docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md
git commit -m "docs(memory): record durable memory architecture"
```

### Tâche 2 : modèle fermé, parsing et refus sensibles

**Fichiers :**

- Créer : `tooling/arnes/src/memory.rs`
- Créer : `tooling/arnes/src/memory/model.rs`
- Créer : `tooling/arnes/src/memory/error.rs`
- Créer : `tooling/arnes/src/memory/validation.rs`
- Créer : `tooling/arnes/src/memory/sensitive.rs`
- Créer : `tooling/arnes/tests/memory_model.rs`
- Créer : `tooling/arnes/tests/memory_validation.rs`
- Modifier : `tooling/arnes/src/lib.rs`
- Modifier : `tooling/arnes/Cargo.toml`
- Modifier : `tooling/arnes/Cargo.lock`

**Interfaces :**

- Consomme : YAML UTF-8 de 1 Mio maximum.
- Produit : `parse_draft(bytes: &[u8]) -> Result<AdmissionDraft, MemoryError>`, `parse_entry(bytes: &[u8]) -> Result<MemoryEntry, MemoryError>`, `validate_draft(draft: AdmissionDraft, authorization: AdmissionAuthorization) -> Result<ValidatedDraft, MemoryError>`.

- [ ] **Step 1: Écrire les tests RED du modèle discriminé**

Écrire des tables couvrant les six kinds et chaque statut autorisé/interdit. Les assertions structurantes sont :

```rust
assert_eq!(parse_entry(active_invariant()).unwrap().status(), Status::Active);
assert_eq!(parse_entry(goal_with_status("superseded")).unwrap_err().code(), "invalid_kind_status");
assert_eq!(parse_entry(active_with_transition()).unwrap_err().code(), "unexpected_transition");
assert_eq!(parse_entry(terminal_without_transition()).unwrap_err().code(), "missing_transition");
assert_eq!(parse_entry(future_schema()).unwrap_err().code(), "unsupported_schema");
assert_eq!(parse_entry(duplicate_yaml_key()).unwrap_err().code(), "duplicate_field");
```

- [ ] **Step 2: Exécuter le RED ciblé**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test memory_model --test memory_validation
```

Résultat attendu : FAIL, module `memory` absent.

- [ ] **Step 3: Implémenter les types fermés**

Utiliser des enums `serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)` et ces résultats publics :

```rust
pub enum AdmissionResult {
    Stored {
        id: MemoryId,
        index_rebuild_required: bool,
    },
    Duplicate { id: MemoryId },
    Rejected { error: MemoryError },
    Conflict { id: MemoryId, error: MemoryError },
}

pub enum OracleVerdict {
    Valid,
    Invalid,
    Unavailable,
    NeedsConfirmation,
}

pub enum ScopeDraft {
    Project,
    User,
}
```

`MemoryId`, `ProjectKey`, `Statement`, `RetrievalTerm`, `Fingerprint` et `UtcTimestamp` restent des newtypes validés; aucun appelant ne construit directement un `MemoryEntry` invalide.

- [ ] **Step 4: Implémenter les refus de contenu**

La barrière refuse PEM privées, URL avec userinfo, `Authorization:`, assignments `password|secret|token|api_key`, préfixes `sk-`, `ghp_`, `github_pat_`, `xox[baprs]-`, marqueurs de system prompt et blocs transcript à rôles répétés. Les longueurs sont bornées : statement 1–500 caractères, summary 1–1000, terme 1–100, question et raison 1–500, 20 termes et 20 sources maximum.

Tester chaque forme, casse et séparateur, plus les faux positifs autorisés `token budget` et `secret management policy`. Le test nomme la liste des contournements conformément à `enforcement-code`.

- [ ] **Step 5: Exécuter le GREEN et les checks Rust**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test memory_model --test memory_validation
cargo fmt --manifest-path tooling/arnes/Cargo.toml --check
cargo clippy --manifest-path tooling/arnes/Cargo.toml --all-targets -- -D warnings
```

Résultat attendu : PASS sur l’environnement courant.

- [ ] **Step 6: Commit**

```bash
git add tooling/arnes/src/lib.rs tooling/arnes/src/memory.rs tooling/arnes/src/memory \
  tooling/arnes/tests/memory_model.rs tooling/arnes/tests/memory_validation.rs \
  tooling/arnes/Cargo.toml tooling/arnes/Cargo.lock
git commit -m "feat(memory): define the durable entry contract"
```

### Tâche 3 : identité projet et résolution des sources

**Fichiers :**

- Créer : `tooling/arnes/src/memory/identity.rs`
- Créer : `tooling/arnes/src/memory/source.rs`
- Créer : `tooling/arnes/src/memory/process.rs`
- Créer : `tooling/arnes/tests/memory_identity.rs`
- Créer : `tooling/arnes/tests/memory_sources.rs`
- Créer : `tooling/arnes/tests/support/memory.rs`

**Interfaces :**

- Consomme : `ValidatedDraft`, cwd, Git et curl injectables dans les tests.
- Produit : `resolve_project(cwd: &Path, git: &dyn ProcessRunner) -> Result<ProjectScope, MemoryError>` et `resolve_sources(draft: ValidatedDraft, context: &SourceContext) -> Result<ResolvedDraft, MemoryError>`.

- [ ] **Step 1: Écrire les tests RED d’identité**

Créer un dépôt principal et deux worktrees réels; exiger la même clé. Exiger une autre clé pour un clone séparé, puis `scope_unavailable` hors Git, sur sortie Git vide, chemin relatif, chemin non canonique et process Git non nul.

```rust
assert_eq!(resolve_project(main_worktree)?, resolve_project(linked_worktree)?);
assert_ne!(resolve_project(main_worktree)?, resolve_project(separate_clone)?);
assert_eq!(resolve_project(non_git).unwrap_err().code(), "scope_unavailable");
```

- [ ] **Step 2: Écrire les tests RED de sources**

Tester `git-file` suivi/modifié, fichier non suivi, chemin `..`, symlink, fichier local absolu, fichier disparu, permission refusée, URL HTTP, credentials, IP littérale, redirect HTTPS, redirect HTTP, 404, 429, 5xx, timeout, TLS, corps > 1 Mio, curl absent et décision utilisateur sans transcript.

Chaque test vérifie `valid`, `invalid` ou `unavailable` et l’absence du locator sensible dans le diagnostic.

- [ ] **Step 3: Implémenter l’identité et les empreintes locales**

Appeler Git avec argv séparé :

```text
git rev-parse --path-format=absolute --git-common-dir
git -C <cwd> ls-files --error-unmatch -- <relative-path>
```

Canonicaliser sans suivre un symlink final, lire seulement un fichier régulier borné, puis produire `sha256:<hex>`. Recalculer l’empreinte après validation et avant commit d’admission afin de détecter le TOCTOU.

- [ ] **Step 4: Implémenter le fetch URL borné**

Parser avec `url`; exécuter curl par argv avec HTTPS seulement, redirections et délais fermés :

```text
curl --silent --show-error --fail-with-body --location --max-redirs 5
     --proto =https --proto-redir =https --connect-timeout 5 --max-time 15
     --max-filesize 1048576 --output <0600-temp> --write-out <status/final-url/remote-ip>
```

Valider URL finale, statut et IP rapportée avant de hasher le corps. Mapper 404/410 vers `invalid`; 429, 5xx, DNS, TLS et timeout vers `unavailable`; toute autre sortie non reconnue échoue `source_unavailable`.

- [ ] **Step 5: Exécuter les tests ciblés**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test memory_identity --test memory_sources
```

Résultat attendu : PASS sans accès réseau réel; le faux curl du fixture couvre toutes les sorties.

- [ ] **Step 6: Commit**

```bash
git add tooling/arnes/src/memory tooling/arnes/tests/memory_identity.rs \
  tooling/arnes/tests/memory_sources.rs tooling/arnes/tests/support/memory.rs \
  tooling/arnes/Cargo.toml tooling/arnes/Cargo.lock
git commit -m "feat(memory): resolve project and proof sources"
```

### Tâche 4 : store YAML atomique et concurrence

**Fichiers :**

- Créer : `tooling/arnes/src/memory/path.rs`
- Créer : `tooling/arnes/src/memory/store.rs`
- Créer : `tooling/arnes/src/memory/lock.rs`
- Créer : `tooling/arnes/tests/memory_store.rs`
- Créer : `tooling/arnes/tests/memory_concurrency.rs`

**Interfaces :**

- Consomme : `ResolvedDraft`, `MemoryRoot` issu de `ARNES_MEMORY_ROOT` ou de `~/.local/share/agent-memory`.
- Produit : `Store::open`, `Store::admit`, `Store::replace_active`, `Store::load`, `Store::list`; aucune autre fonction n’écrit un YAML.

- [ ] **Step 1: Écrire les tests RED de layout et permissions**

Exiger ce snapshot :

```text
agent-memory/                         0700
  .lock                              0600
  entries/user/<id>.yaml             0600
  entries/project/<key>/<id>.yaml    0600
  index.json                         0600
  oracle-cache.json                  0600
```

Tester racine symlink, parent non répertoire, permissions trop ouvertes réparables, chmod refusé, fichier final existant et traversal par ID.

- [ ] **Step 2: Écrire les tests RED d’atomicité et concurrence**

Injecter des failpoints avant flush, avant fsync, avant rename YAML, après rename YAML et avant rename index. Après chaque interruption, rouvrir le store et exiger soit aucun YAML, soit un YAML complet avec index déclaré périmé, jamais un YAML partiel ni un index pendant.

Lancer deux processus : même draft donne un `stored` et un `duplicate`; drafts divergents au même ID donnent un `stored` et un `conflict`.

- [ ] **Step 3: Implémenter le verrou et les remplacements atomiques**

Utiliser le lock exclusif `rustix`, une attente bornée à 2 s, un nom temporaire aléatoire créé `create_new`, `write_all`, `sync_all`, rename dans le même répertoire et fsync du répertoire. Refuser tout symlink à chaque composant contrôlé.

- [ ] **Step 4: Implémenter l’admission idempotente**

Canonicaliser le document sans `created_at`, calculer l’ID, acquérir le lock, revalider les sources locales, comparer le YAML existant, préparer l’index en mémoire, puis appliquer l’ordre de commit global. Un échec d’index après rename YAML retourne `Stored { index_rebuild_required: true }`, jamais `rejected` ni rollback destructif.

- [ ] **Step 5: Exécuter les tests ciblés**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test memory_store --test memory_concurrency
```

Résultat attendu : PASS; aucun fixture ne touche le store personnel.

- [ ] **Step 6: Commit**

```bash
git add tooling/arnes/src/memory tooling/arnes/tests/memory_store.rs \
  tooling/arnes/tests/memory_concurrency.rs
git commit -m "feat(memory): persist entries atomically"
```

### Tâche 5 : index dérivé et classement lexical

**Fichiers :**

- Créer : `tooling/arnes/src/memory/index.rs`
- Créer : `tooling/arnes/src/memory/search.rs`
- Créer : `tooling/arnes/tests/memory_index.rs`
- Créer : `tooling/arnes/tests/memory_search.rs`

**Interfaces :**

- Consomme : YAML du store et `SearchRequest { query, project_key, include_user: true, limit: 5 }`.
- Produit : `Index::load_or_rebuild(store) -> Result<IndexLoad, MemoryError>` et `search(index, request) -> SearchSelection` avec `selected`, `omitted_by_limit` et diagnostics.

- [ ] **Step 1: Écrire les tests RED de reconstruction**

Tester index absent, JSON corrompu, inventaire périmé après ajout/modification/suppression, YAML illisible, schema futur, statut invalide, entrée terminale, index non inscriptible et deuxième reconstruction byte-identique.

Une entrée invalide est omise avec `entry_id`, `check` et `effect`; son statement ne paraît jamais dans le diagnostic.

- [ ] **Step 2: Écrire les tests RED du ranking**

Normaliser NFKD, retirer les diacritiques, passer en minuscules et remplacer tout séparateur par un espace. Le score est le tuple décroissant : nombre de phrases `retrieval_terms` présentes, nombre de tokens de termes présents, nombre de tokens statement présents. Une phrase de terme suffit; sans phrase, au moins deux tokens distincts sont requis. Les ties sont `id` bytewise.

```rust
assert_eq!(ids(search("deploiement mémoire", fixture_index())), ["mem_alias", "mem_statement"]);
assert_eq!(search("sans rapport", fixture_index()).selected.len(), 0);
assert_eq!(search("agent", six_matches()).selected.len(), 5);
assert_eq!(search("agent", six_matches()).omitted_by_limit, 1);
```

- [ ] **Step 3: Implémenter l’index minimal**

Chaque ligne contient seulement `id`, `kind`, `status`, `scope`, `retrieval_terms`, résumé tronqué à 160 caractères Unicode, chemin relatif du YAML, longueur et mtime nanoseconde. L’en-tête contient `schema_version: 1` et le digest SHA-256 de l’inventaire trié.

Le check de fraîcheur liste et stat les YAML sans les parser; tout écart reconstruit sous lock. L’index n’est jamais injecté en bloc.

- [ ] **Step 4: Exécuter le GREEN**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test memory_index --test memory_search
```

Résultat attendu : PASS sur le filesystem temporaire.

- [ ] **Step 5: Commit**

```bash
git add tooling/arnes/src/memory tooling/arnes/tests/memory_index.rs \
  tooling/arnes/tests/memory_search.rs
git commit -m "feat(memory): derive the lexical index"
```

### Tâche 6 : oracles, cache 48 h et transitions

**Fichiers :**

- Créer : `tooling/arnes/src/memory/clock.rs`
- Créer : `tooling/arnes/src/memory/oracle.rs`
- Créer : `tooling/arnes/src/memory/cache.rs`
- Créer : `tooling/arnes/src/memory/retrieval.rs`
- Créer : `tooling/arnes/tests/memory_oracle.rs`
- Créer : `tooling/arnes/tests/memory_cache.rs`
- Créer : `tooling/arnes/tests/memory_retrieval.rs`

**Interfaces :**

- Consomme : `SearchSelection`, `Clock`, `SourceResolver`, `Store`.
- Produit : `evaluate_oracle(entry, context) -> OracleEvaluation`, `retrieve(request, context) -> RetrievalReport`, `confirm(id, HumanConclusion, context) -> TransitionResult`.

- [ ] **Step 1: Écrire les tests RED de fraîcheur**

Avec une horloge injectée, couvrir `47:59:59.999` cache hit, `48:00:00` miss, timestamp futur miss, modification locale pendant la fenêtre, distant non refetché avant 48 h, distant expiré indisponible, fallback humain, et absence de cache pour trois verdicts non-valid.

```rust
assert!(cache.usable_at(validated_at + hours(48) - millis(1)));
assert!(!cache.usable_at(validated_at + hours(48)));
assert!(!cache.usable_at(validated_at - millis(1)));
```

- [ ] **Step 2: Écrire les tests RED de transitions**

Pour chaque kind, tester chaque terminal autorisé avec verdict humain `valid`; tester terminal incompatible, seconde transition et raison vide. Un fingerprint différent produit seulement `invalidated`; `unavailable` et `needs_confirmation` ne modifient aucun octet YAML.

- [ ] **Step 3: Implémenter cache et oracle**

Le cache JSON contient `entry_id`, digest de l’oracle, empreintes sources, `validated_at`, environnement et uniquement `verdict: valid`. Pour une source locale, recalculer toujours l’empreinte avant cache hit; pour URL, accepter le hit avant 48 h. Une écriture cache échouée conserve le verdict courant mais rend le prochain tour non caché.

- [ ] **Step 4: Implémenter retrieval sans mémoire stale**

Charger seulement les cinq YAML sélectionnés, les reparsing après sélection, évaluer cache/oracle, appliquer atomiquement toute invalidation, puis rendre :

```rust
pub struct RetrievalReport {
    pub injected: Vec<InjectedMemory>,
    pub omitted: Vec<OmittedMemory>,
    pub omitted_by_limit: usize,
}
```

`InjectedMemory` contient kind, statement, sources résumées et âge du verdict; `OmittedMemory` contient id, code, question humaine éventuelle et effet `not_applied`.

- [ ] **Step 5: Exécuter le GREEN**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml \
  --test memory_oracle --test memory_cache --test memory_retrieval
```

Résultat attendu : PASS, avec snapshots YAML inchangés sur tous les verdicts transitoires.

- [ ] **Step 6: Commit**

```bash
git add tooling/arnes/src/memory tooling/arnes/tests/memory_oracle.rs \
  tooling/arnes/tests/memory_cache.rs tooling/arnes/tests/memory_retrieval.rs
git commit -m "feat(memory): verify freshness before retrieval"
```

### Tâche 7 : admission orchestrée et CLI stable

**Fichiers :**

- Créer : `tooling/arnes/src/memory/admission.rs`
- Créer : `tooling/arnes/src/memory_cli.rs`
- Créer : `tooling/arnes/tests/memory_admission.rs`
- Créer : `tooling/arnes/tests/memory_cli.rs`
- Modifier : `tooling/arnes/src/cli.rs`
- Modifier : `tooling/arnes/src/main.rs`

**Interfaces :**

- Consomme : draft YAML sur stdin, query UTF-8 sur stdin, IDs et conclusions via options typées.
- Produit : `arnes memory admit|retrieve|confirm|audit|hook`, JSON stable sur stdout, diagnostics redacted sur stderr, exit `0` succès/duplicate, `2` usage/rejet, `3` conflit, `4` indisponibilité.

- [ ] **Step 1: Écrire les tests RED d’admission**

Prouver l’ordre validation → scope → source → oracle → duplicate → commit. Chaque échec de la matrice globale vérifie snapshot store inchangé, sauf l’échec index explicitement post-commit.

Tester demande projet par défaut, `--user-scope-authorized` obligatoire pour user, confirmation humaine absente, source changée entre résolution et lock, retry après commit YAML/index échoué et concurrence divergente.

- [ ] **Step 2: Écrire les tests RED CLI**

Tester ces formes exactes :

```bash
printf '%s' "$draft" | arnes memory admit --format json
printf '%s' "$query" | arnes memory retrieve --query-stdin --format json
printf '%s' "$reason" | arnes memory confirm --id "$id" --status confirmed --reason-stdin
arnes memory audit --include-terminal --format json
```

Exiger une seule valeur JSON stdout, stderr vide au succès, codes stables, stdin vide/surdimensionné, option inconnue, scope user non autorisé et aucun contenu sensible dans les erreurs.

- [ ] **Step 3: Implémenter l’orchestration**

`admit` remplit ID, scope key, fingerprints et timestamps; le modèle ne peut pas les fournir. `retrieve` résout toujours le cwd courant et interroge scope projet + user. `confirm` ne permet qu’un terminal compatible et exige la réponse utilisateur par la skill. `audit` lit sans réactiver ni réparer les YAML.

- [ ] **Step 4: Exécuter le GREEN CLI**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test memory_admission --test memory_cli
cargo test --manifest-path tooling/arnes/Cargo.toml
```

Résultat attendu : PASS dans un `ARNES_MEMORY_ROOT` temporaire.

- [ ] **Step 5: Commit**

```bash
git add tooling/arnes/src/cli.rs tooling/arnes/src/main.rs tooling/arnes/src/memory_cli.rs \
  tooling/arnes/src/memory tooling/arnes/tests/memory_admission.rs \
  tooling/arnes/tests/memory_cli.rs
git commit -m "feat(memory): expose admission and retrieval commands"
```

### Tâche 8 : hooks synchrones Codex et Claude

**Fichiers :**

- Créer : `tooling/arnes/src/memory/hook.rs`
- Créer : `tooling/arnes/tests/memory_hooks.rs`
- Modifier : `tooling/arnes/src/manifest/hooks.rs`
- Modifier : `tooling/arnes/src/hooks.rs`
- Modifier : `tooling/arnes/src/hooks/adapters.rs`
- Modifier : `tooling/arnes/src/hooks/reconcile.rs`
- Modifier : `tooling/arnes/tests/hooks_setup.rs`
- Modifier : `tooling/arnes/tests/hooks_reconciliation/installation.rs`
- Modifier : `tooling/arnes/tests/hooks_reconciliation/ownership.rs`

**Interfaces :**

- Consomme : payload natif `UserPromptSubmit` Codex/Claude.
- Produit : `arnes memory hook --agent codex|claude`, réponse host avec `additionalContext`; aucune installation mémoire Cursor à cette frontière.

- [ ] **Step 1: Capturer les contrats officiels dans des fixtures RED**

Ajouter un payload minimal et un payload complet pour chaque agent. Exiger query/cwd, refuser mauvais événement, stdin vide, JSON > 1 Mio et champ requis absent. Tester une mémoire pertinente, aucune correspondance, oracle unavailable et cinq plus un résultats.

- [ ] **Step 2: Tester le contexte rendu**

Le texte injecté commence par un marqueur versionné `ARNES_MEMORY_CONTEXT_V1`, liste chaque mémoire avec kind, statement, preuve et âge, liste séparément les omissions, puis impose : annoncer ces éléments avant application, ne rien appliquer d’omis, et proposer sans écrire toute nouvelle connaissance admissible.

Le test vérifie que le texte ne contient jamais l’index complet, cache, chemin absolu du store ou source brute.

- [ ] **Step 3: Étendre la réconciliation des hooks**

Ajouter `HookKind::Memory` et une matrice événements par kind. Installer la commande absolue `~/.local/bin/arnes memory hook --agent <agent>` seulement sur `UserPromptSubmit` Codex/Claude. Préserver ordre et handlers tiers, supprimer les anciennes occurrences possédées, refuser configuration invalide et conserver l’idempotence byte à byte.

- [ ] **Step 4: Exécuter les tests hooks**

```bash
cargo test --manifest-path tooling/arnes/Cargo.toml --test memory_hooks
cargo test --manifest-path tooling/arnes/Cargo.toml --test hooks_setup \
  --test hooks_reconciliation
```

Résultat attendu : PASS; Cursor ne reçoit aucun faux handler pré-prompt.

- [ ] **Step 5: Commit**

```bash
git add tooling/arnes/src/memory tooling/arnes/src/manifest/hooks.rs \
  tooling/arnes/src/hooks.rs tooling/arnes/src/hooks tooling/arnes/tests/memory_hooks.rs \
  tooling/arnes/tests/hooks_setup.rs tooling/arnes/tests/hooks_reconciliation
git commit -m "feat(memory): inject retrieval through supported hooks"
```

### Tâche 9 : skill commune, règle Cursor et déploiement

**Fichiers :**

- Modifier : `harness/skills/memory-governance/SKILL.md`
- Créer : `harness/skills/memory-governance/references/entry-contract.md`
- Modifier : `harness/skills/memory-governance/evals/trigger-queries.json`
- Modifier, généré : `harness/skills/README.md`
- Créer : `harness/rules/memory-governance-cursor.mdc`
- Modifier : `home/.arnes.yaml`
- Modifier : `Makefile`
- Modifier : `tooling/deployment-links.test.ts`
- Modifier : `tooling/deployment-codex-wiring.test.ts`
- Modifier : `tooling/arnes/src/rules.rs`
- Modifier : `tooling/arnes/tests/rules.rs`
- Modifier : `tooling/arnes/tests/manifest_rules.rs`

**Interfaces :**

- Consomme : contexte hook Codex/Claude, règle Cursor, commandes CLI de la tâche 7.
- Produit : un seul workflow conversationnel d’admission/proposition; installations user sur trois agents; Cursor exécute la retrieval avant le travail sous oracle comportemental.

- [ ] **Step 1: Écrire les tests de déploiement RED**

Étendre les tableaux attendus afin que `memory-governance` soit lié depuis `harness/skills/memory-governance` vers `.agents/skills`, `.claude/skills` et `.cursor/skills`. Exiger la règle Cursor liée depuis `harness/rules/memory-governance-cursor.mdc`, et `HookKind::Memory` seulement pour Codex/Claude dans `.arnes.yaml`. Étendre le doctor Arnes afin que `cursor + user + rules` soit une combinaison supportée, avec destination canonique `~/.cursor/rules/<name>.mdc`; toutes les autres nouvelles combinaisons restent `unsupported`.

```bash
bun test tooling/deployment-links.test.ts tooling/deployment-codex-wiring.test.ts
```

Résultat attendu : FAIL sur les liens et déclarations absents.

- [ ] **Step 2: Migrer la skill via `skill-manager fix`**

La description distingue quatre triggers : demande explicite, acceptation, connaissance durable détectée, et début de toute tâche Cursor. Le body impose : retrieval avant travail Cursor; annonce des mémoires appliquées; présentation d’une proposition complète sans écriture implicite; admission immédiate après autorisation; confirmation humaine; refus et redaction.

Déplacer le schéma, exemples de draft et commandes détaillées dans `references/entry-contract.md`; garder `SKILL.md` sous 500 lignes et sans placeholder shell positionnel.

- [ ] **Step 3: Ajouter la règle Cursor minimale**

Écrire un frontmatter Cursor `alwaysApply: true` et uniquement ce contrat d’adapter : avant toute analyse ou outil, charger `memory-governance`, lui fournir le prompt actif, attendre `arnes memory retrieve`, puis suivre son résultat; en cas d’échec, annoncer l’indisponibilité sans appliquer de mémoire. Ne dupliquer ni schéma, ni ranking, ni politique de confidentialité.

- [ ] **Step 4: Déployer les surfaces**

Déclarer la skill pour les trois agents, le hook mémoire pour Codex/Claude et la règle user Cursor dans le manifeste et le Makefile. Les cibles agent dépendent du binaire Arnes avant hooks/règle. Toute destination existante inattendue reste un échec sans écrasement.

- [ ] **Step 5: Doctor et index déterministe**

```bash
command -v skills-ref || true
prettier --write harness/skills/memory-governance/SKILL.md \
  harness/skills/memory-governance/references/entry-contract.md \
  harness/skills/memory-governance/evals/trigger-queries.json \
  harness/rules/memory-governance-cursor.mdc home/.arnes.yaml
```

Exécuter `skill-manager sync-index user`, enregistrer le hash, l’exécuter une seconde fois puis vérifier le hash :

```bash
shasum -a 256 harness/skills/README.md > /tmp/memory-skills-index.sha256
shasum -a 256 -c /tmp/memory-skills-index.sha256
```

Exécuter ensuite le doctor complet. Résultat attendu : PASS local; si `skills-ref` manque, rapporter exactement `Standard validation: unavailable (skills-ref not installed)`.

- [ ] **Step 6: Vérifier le déploiement sans mutation globale**

```bash
make -n codex claude-code cursor
bun test tooling/deployment-links.test.ts tooling/deployment-codex-wiring.test.ts
cargo test --manifest-path tooling/arnes/Cargo.toml --test rules --test manifest_rules
```

Résultat attendu : une skill par agent, deux hooks mémoire synchrones, une règle Cursor, aucun install réel depuis le worktree.

- [ ] **Step 7: Commit**

```bash
git add harness/skills/memory-governance harness/skills/README.md \
  harness/rules/memory-governance-cursor.mdc home/.arnes.yaml Makefile \
  tooling/deployment-links.test.ts tooling/deployment-codex-wiring.test.ts \
  tooling/arnes/src/rules.rs tooling/arnes/tests/rules.rs \
  tooling/arnes/tests/manifest_rules.rs
git commit -m "feat(memory): deploy governance to every agent"
```

### Tâche 10 : oracle end-to-end multi-agent

**Fichiers :**

- Créer : `tooling/agent-memory-eval.ts`
- Créer : `tooling/agent-memory-eval.test.ts`
- Créer : `tooling/agent-memory-eval-scenarios.json`
- Remplacer : `docs/memory-governance-validation.md`

**Interfaces :**

- Consomme : binaires agents explicitement résolus, Arnes buildé, fixture Git neutre, `ARNES_MEMORY_ROOT` temporaire, scénarios versionnés.
- Produit : résultats bruts sous un `mktemp -d` hors Git et rapport normalisé sans contenu privé; aucun PASS dérivé de la réponse finale seule.

- [ ] **Step 1: Écrire les tests RED du runner**

Avec trois faux binaires, tester process frais, timeout 120 s, exit non nul, JSONL malformé, nonce absent, ordre d’événements incorrect, mutation en contrôle, store personnel référencé, version absente et cleanup interrompu. Le runner refuse de démarrer si le store résolu n’est pas sous sa racine temporaire.

- [ ] **Step 2: Implémenter quatre scénarios composés**

Le JSON versionné contient :

1. connaissance durable implicite → proposition visible → aucune écriture;
2. acceptation explicite → `stored` → nouveau processus pertinent → annonce preuve/fraîcheur → nouveau processus sans rapport → aucune injection;
3. secret évident et transcript brut → `rejected` → store inchangé;
4. entrée seedée → source indisponible omise sans mutation → source contradictoire transitionnée `invalidated`.

Chaque scénario a condition `control` sans skill/hook/règle et `deployed`, trois réplicats par agent. Chaque réplicat utilise son store et son dépôt fixture; aucune session ne partage le contexte agent d’une autre.

- [ ] **Step 3: Implémenter les oracles par surface**

- Codex : journal prouve `UserPromptSubmit`, fin du retrieval, `additionalContext`, puis premier événement modèle portant le nonce.
- Claude : `--include-hook-events` prouve le même ordre.
- Cursor : journal prouve lecture de la règle et de `SKILL.md`, appel `arnes memory retrieve` terminé, puis première analyse/action utilisant le nonce; cela reste un oracle comportemental de version, pas une garantie native.
- Tous : inspecter directement le store temporaire pour permissions, YAML, index, cache et absence de mutation implicite.

- [ ] **Step 4: Exécuter le runner unitairement**

```bash
bun test tooling/agent-memory-eval.test.ts
bun run typecheck
bun run lint
bun run format:typescript:check
```

Résultat attendu : PASS avec faux agents sur macOS et Linux.

- [ ] **Step 5: Exécuter les agents réels sur macOS**

Résoudre et enregistrer `uname -mrs` et chaque `--version`, puis lancer sans modifier les configurations user :

```bash
bun tooling/agent-memory-eval.ts --agent codex --replicates 3
bun tooling/agent-memory-eval.ts --agent claude --replicates 3
bun tooling/agent-memory-eval.ts --agent cursor --replicates 3
```

Le runner construit des fixtures projet contrôle/déployé, utilise les commandes fraîches documentées dans la spec, conserve les raw JSONL hors Git et échoue au premier oracle absent. Pour Cursor, aucun `--ephemeral` ni exclusion totale des settings n’est revendiqué; nouveau processus, fixture et nonce isolent seulement la mesure.

- [ ] **Step 6: Écrire le rapport versionné**

`docs/memory-governance-validation.md` nomme date, OS/architecture, versions, commandes, nombre de réplicats, résultat par capacité et agent, limites Cursor et Linux, chemins des artefacts temporaires et nettoyage. Il remplace explicitement les 26 checks candidate-only; aucun transcript, statement privé ou taux inventé n’y paraît.

Si un agent n’atteint pas `3/3` sur une capacité requise, arrêter la PR : ne pas dégrader le critère, ne pas ajouter une promesse, ne pas créer de ticket de report pour cette capacité.

- [ ] **Step 7: Commit**

```bash
git add tooling/agent-memory-eval.ts tooling/agent-memory-eval.test.ts \
  tooling/agent-memory-eval-scenarios.json docs/memory-governance-validation.md
git commit -m "test(memory): prove durable memory across agents"
```

### Tâche 11 : barrières portables et vérification finale

**Fichiers :**

- Modifier : `.github/workflows/test-arnes.yml`
- Modifier : `.github/workflows/lint.yml`
- Vérifier : tous les fichiers des tâches 1–10

**Interfaces :**

- Consomme : implémentation et rapport E2E commités.
- Produit : gates couvrant chaque extension touchée sur ses plateformes supportées, puis head exact prêt pour revue.

- [ ] **Step 1: Écrire le RED de couverture CI**

Étendre le test de contrat workflow pour exiger une matrice `ubuntu-latest` et `macos-latest` sur Arnes avec `cargo fmt`, `cargo clippy --all-targets -- -D warnings` et `cargo test`. Ajouter les nouveaux Markdown/YAML/JSON au `prettier --check` et `cspell`; le TypeScript est déjà couvert par les scripts Bun.

- [ ] **Step 2: Exécuter toutes les barrières locales**

```bash
cargo fmt --manifest-path tooling/arnes/Cargo.toml --check
cargo clippy --manifest-path tooling/arnes/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path tooling/arnes/Cargo.toml
bun test
bun run lint
bun run typecheck
bun run format:typescript:check
prettier --check docs/adr/README.md docs/adr/042-memoire-durable-locale-partagee.md \
  docs/superpowers/specs/2026-08-28-durable-agent-memory-design.md \
  docs/memory-governance-validation.md home/.arnes.yaml \
  harness/skills/memory-governance/SKILL.md \
  harness/skills/memory-governance/references/entry-contract.md \
  harness/skills/memory-governance/evals/trigger-queries.json \
  harness/rules/memory-governance-cursor.mdc tooling/agent-memory-eval-scenarios.json
git diff --check 25860b4..HEAD
```

Résultat attendu : PASS sur macOS observé; la CI reproduit le cœur Rust/Bun sur macOS et Linux. Les agents réels restent prouvés seulement sur le macOS et les versions du rapport.

- [ ] **Step 3: Vérifier taille, commentaires et état local**

Inspecter chaque fonction de production > 50 lignes logiques et fichier manuscrit > 250 lignes; scinder par responsabilité ou justifier dans la delivery note. Exécuter :

```bash
rg -n '^\s*(//|/\*|\*)' tooling/arnes/src/memory tooling/agent-memory-eval.ts || true
git status --short
```

Résultat attendu : aucun commentaire de code ajouté; uniquement les fichiers attendus avant le commit de barrière.

- [ ] **Step 4: Commit des gates**

```bash
git add .github/workflows/test-arnes.yml .github/workflows/lint.yml
git commit -m "ci(memory): verify the durable memory system"
```

- [ ] **Step 5: Rejouer les oracles sur le head exact**

Rejouer les commandes de Step 2, les trois runs d’agent de la tâche 10 et `git diff --check 25860b4..HEAD`. Enregistrer le SHA exact dans le rapport si les résultats diffèrent du commit de preuve; sinon vérifier que le SHA documenté est toujours ancêtre de HEAD et que seuls les gates ont changé.

- [ ] **Step 6: Revue indépendante**

Utiliser `superpowers:requesting-code-review`, puis une passe adversariale `enforcement-code` ciblée sur traversal, symlink, TOCTOU, collision, lock, URL/redirect, secrets, bypass user-scope et hooks absents. Toute correction rejoue le test ciblé, la suite complète et les oracles affectés avant livraison.

## Self-review du plan

- Couverture spec : admission/proposition (tâches 7/9/10), YAML/permissions/concurrence (4), index/recherche/isolation (5), preuves/sources (3), oracles/cache/transitions (6), hooks et visibilité (8/9), trois agents et A/B frais (10), plateformes et gates (11).
- Limite certaine conservée : Cursor ne possède pas d’injection pré-prompt native; seule l’observation `3/3` de la règle + skill autorise la capacité sur la version mesurée.
- Chemins externes explicités : Git, fichiers, curl/URL, stdin hook, horloge, filesystem, processus agents et configuration user résiduelle Cursor.
- Aucun placeholder, migration SQLite, capacité future, double écriture ou édition d’état généré agent n’est prévu.
- Commentaires ajoutés par ce plan : aucun commentaire de code; les futurs implementers doivent conserver cette liste vide.
