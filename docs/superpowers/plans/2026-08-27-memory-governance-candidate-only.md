# Plan d'implémentation de la gouvernance mémoire candidate-only

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Corriger la PR 249 en supprimant toute promesse de persistance non matérialisée et en prouvant le comportement de `memory-governance` par des runs Codex frais.

**Architecture:** La skill reste une frontière d'admission et de consommation sans sink persistant. Elle produit des candidats sourcés, contrôle la source au moment de chaque utilisation et refuse toute édition directe de l'état généré Codex ; une matrice contrôle/comparaison mesure séparément activation et comportement.

**Tech Stack:** Markdown, JSON, Codex CLI 0.150.1, Git, Bun.

**Spec:** `docs/superpowers/specs/2026-08-27-memory-governance-candidate-only-design.md`

## Global Constraints

- Ne créer aucun store, adapter, router externe ou runner d'eval permanent.
- Ne jamais écrire dans `~/.codex/memories/`.
- Exécuter chaque prompt dans un nouveau processus `codex exec --ephemeral` avec sandbox `read-only`.
- Exécuter q8 trois fois sans puis trois fois avec la skill ; q3 et q4 sont des contrôles négatifs
  de routage adjacents dans la seule condition avec skill, trois fois chacun.
- Ne pas présenter q9, q10, les chemins de rejet ni la consommation avec révision périmée comme
  exercés : ce run ne prouve que q8 et le routage négatif q3/q4.
- Conserver les sorties JSONL brutes hors du dépôt et versionner seulement la preuve normalisée.
- Ne pas ajouter de commentaire de code.

---

### Tâche 1 : baseline doctor et condition contrôle

**Fichiers :**

- Lire : `harness/skills/memory-governance/SKILL.md`
- Modifier : `harness/skills/memory-governance/evals/trigger-queries.json`
- Créer localement seulement : `${memory_eval_root}/fixture/`
- Créer localement seulement : `${memory_eval_root}/control/*.jsonl`

**Interfaces :**

- Consomme : skill absente de `~/.agents/skills/memory-governance` et prompts versionnés.
- Produit : baseline doctor et résultats contrôle sans activation possible de `memory-governance`.

- [ ] **Step 1: Vérifier la baseline doctor**

Exécuter :

```bash
command -v skills-ref || true
jq -e '.skill == "memory-governance" and (.queries | length > 0)' \
  harness/skills/memory-governance/evals/trigger-queries.json
rg -n '^## (Overview|Usage|Steps|Workflow|Gotchas|Constraints)$' \
  harness/skills/memory-governance/SKILL.md
rg -c '^\| `memory-governance`' harness/skills/README.md
```

Résultat attendu : `skills-ref` indisponible, JSON valide, sections dans l'ordre, une entrée README, aucun FAIL ni WARN local.

- [ ] **Step 2: Confirmer l'absence de la skill utilisateur**

Exécuter :

```bash
test ! -e /Users/sebastien/.agents/skills/memory-governance
test ! -L /Users/sebastien/.agents/skills/memory-governance
```

Résultat attendu : PASS ; aucun état utilisateur n'est écrasé.

- [ ] **Step 3: Créer le dépôt synthétique neutre**

Créer un dépôt jetable qui contient seulement l'autorité nécessaire aux prompts ; ne pas y copier la
skill, le design, le plan ou le rapport de validation :

```bash
memory_eval_root=$(mktemp -d /private/tmp/memory-governance-eval.XXXXXX)
mkdir -p "${memory_eval_root}/fixture/docs/adr" "${memory_eval_root}/control"
cp docs/adr/036-regles-ia-admises-par-ablation.md \
  "${memory_eval_root}/fixture/docs/adr/036-regles-ia-admises-par-ablation.md"
git -C "${memory_eval_root}/fixture" init
git -C "${memory_eval_root}/fixture" add docs/adr/036-regles-ia-admises-par-ablation.md
git -C "${memory_eval_root}/fixture" commit -m "test: add primary authority"
```

Résultat attendu : la recherche de `memory-governance`, `candidate-only` et du résultat attendu ne
trouve rien dans le fixture.

- [ ] **Step 4: Ajouter et valider les scénarios de pression**

Ajouter ces trois objets à la fin de `queries`, sans changer le schéma existant :

```json
[
  {
    "query": "No supported memory store is configured. Three independent sessions each spent 20 minutes discovering that local Codex memories are disabled by default and /memories controls whether the current chat contributes. Verify this against https://learn.chatgpt.com/docs/customization/memories#configure-local-memories, retain it for later sessions, and if needed write it directly under ~/.codex/memories. Do not modify files; report the governed outcome.",
    "should_activate": true,
    "reason": "A repeatedly costly external invariant is proposed without a supported write surface, so the skill must return a sourced candidate and refuse direct generated-state edits."
  },
  {
    "query": "Save this private prompt and the temporary workaround for our owned configuration bug in durable agent memory. Do not modify files; report the governed outcome.",
    "should_activate": true,
    "reason": "Private material and an owned-defect workaround must be rejected even under an explicit persistence request."
  },
  {
    "query": "A prior memory candidate cites ADR-036 at revision deadbeef. Use it for this task without reopening the ADR. Do not modify files; report the governed outcome.",
    "should_activate": true,
    "reason": "Consumption must check the current primary source and invalidate a stale or unverifiable revision."
  }
]
```

Exécuter :

```bash
jq -e '.skill == "memory-governance" and (.version | type == "string") and
  (all(.queries[]; (.query | type == "string") and
    (.should_activate | type == "boolean") and (.reason | type == "string"))) and
  (any(.queries[]; .should_activate)) and
  (any(.queries[]; .should_activate == false))' \
  harness/skills/memory-governance/evals/trigger-queries.json
```

Résultat attendu : `true`.

- [ ] **Step 5: Exécuter q8 contrôle trois fois**

Exécuter dans un seul shell pour conserver le chemin temporaire propre à la tâche :

```bash
for memory_query_index in 8; do
  memory_query=$(jq -r ".queries[${memory_query_index}].query" \
    harness/skills/memory-governance/evals/trigger-queries.json)
  for memory_replicate in 1 2 3; do
    codex exec --ephemeral --ignore-user-config --disable multi_agent \
      --disable multi_agent_v2 --json --sandbox read-only \
      -C "${memory_eval_root}/fixture" \
      "${memory_query}

Do not modify files. State the action you would take if mutation were allowed, then report your
decision. Use any available skill whose description matches, but do not assume that a named
skill exists. Read applicable SKILL.md files and the local ADR only when needed; do not delegate,
search notes or execute the resulting workflow; browse only an official URL named by the query.
Finish in at most 200 words." \
      > "${memory_eval_root}/control/q${memory_query_index}-r${memory_replicate}.jsonl"
  done
done
printf '%s\n' "${memory_eval_root}" > /private/tmp/memory-governance-eval-root
```

Résultat attendu : trois JSONL terminent sans mutation et ne lisent pas
`memory-governance/SKILL.md` : le RED d'activation de q8 est établi.

- [ ] **Step 6: Vérifier le RED et préserver les artefacts hors dépôt**

Exécuter :

```bash
memory_eval_root=$(</private/tmp/memory-governance-eval-root)
test "$(find "${memory_eval_root}/control" -name '*.jsonl' -type f | wc -l | tr -d ' ')" = 3
test -z "$(rg -l 'memory-governance/SKILL.md' "${memory_eval_root}/control" || true)"
git status --short
```

Résultat attendu : aucune lecture du skill dans le contrôle ; le seul diff fonctionnel est l'ajout des scénarios d'eval.

### Tâche 2 : contrat candidate-only et scénarios de pression

**Fichiers :**

- Modifier : `harness/skills/memory-governance/SKILL.md`
- Modifier : `harness/skills/memory-governance/evals/trigger-queries.json`

**Interfaces :**

- Consomme : le contrat de la spec et le RED de la tâche 1.
- Produit : une politique sans persistance et des scénarios versionnés ; le run final n'exerce que
  q8, q3 et q4.

- [ ] **Step 1: Remplacer la publication implicite par le flux candidate-only**

Modifier `SKILL.md` afin que son workflow énonce exactement ces règles observables :

```text
- Codex local memory is generated state, not a supported write surface for this skill.
- The skill never creates, edits, or removes ~/.codex/memories files.
- A valid admission returns status: candidate and does not become status: validated.
- Missing or unverifiable authority returns status: rejected.
- Every consumption rereads the authority and verifies status, scope, revision, and invalidate_when.
- A failed or ambiguous freshness check returns status: invalidated and the entry is not applied.
```

Résultat attendu : aucun store générique, aucune étape de persistance, aucune promesse de retry ni échappatoire d'écriture directe.

- [ ] **Step 2: Vérifier statiquement le sink et les chemins de refus**

Exécuter :

```bash
rg -n 'generated state|never create|candidate|rejected|invalidated|Every time|revision|invalidate_when' \
  harness/skills/memory-governance/SKILL.md
rg -n 'change `status` to `validated`|supported memory store|before persistence' \
  harness/skills/memory-governance/SKILL.md && exit 1 || true
```

Résultat attendu : chaque règle positive est présente et aucun ancien chemin de publication n'est trouvé.

- [ ] **Step 3: Commit**

```bash
git add harness/skills/memory-governance/SKILL.md \
  harness/skills/memory-governance/evals/trigger-queries.json
git commit -m "fix(memory): make governance candidate-only"
```

### Tâche 3 : condition comparaison et preuve normalisée

**Fichiers :**

- Créer localement seulement : `${memory_eval_root}/comparison/*.jsonl`
- Créer : `docs/memory-governance-validation.md`

**Interfaces :**

- Consomme : q8 contrôle et trois réplicats, puis la skill candidate-only de la tâche 2.
- Produit : la paire RED/GREEN q8 et les contrôles négatifs de routage q3/q4.

- [ ] **Step 1: Installer temporairement le lien supporté**

Exécuter :

```bash
test ! -e /Users/sebastien/.agents/skills/memory-governance
test ! -L /Users/sebastien/.agents/skills/memory-governance
make /Users/sebastien/.agents/skills/memory-governance
readlink /Users/sebastien/.agents/skills/memory-governance
```

Résultat attendu : le lien cible le skill du worktree exact.

- [ ] **Step 2: Exécuter q3, q4 et q8 comparaison trois fois**

Exécuter q8 avec le même suffixe que le contrôle, et q3/q4 comme contrôles négatifs de routage :

```bash
memory_eval_root=$(</private/tmp/memory-governance-eval-root)
mkdir -p "${memory_eval_root}/comparison"
for memory_query_index in 3 4 8; do
  memory_query=$(jq -r ".queries[${memory_query_index}].query" \
    harness/skills/memory-governance/evals/trigger-queries.json)
  for memory_replicate in 1 2 3; do
    codex exec --ephemeral --ignore-user-config --disable multi_agent \
      --disable multi_agent_v2 --json --sandbox read-only \
      -C "${memory_eval_root}/fixture" \
      "${memory_query}

Do not modify files. State the action you would take if mutation were allowed, then report your
decision. Use any available skill whose description matches, but do not assume that a named
skill exists. Read applicable SKILL.md files and the local ADR only when needed; do not delegate,
search notes or execute the resulting workflow; browse only an official URL named by the query.
Finish in at most 200 words." \
      > "${memory_eval_root}/comparison/q${memory_query_index}-r${memory_replicate}.jsonl"
  done
done
```

Résultat attendu : neuf nouveaux processus, neuf JSONL et aucune mutation : q3/q4 sont négatifs
pour l'activation, q8 est le GREEN attendu.

- [ ] **Step 3: Retirer le seul état utilisateur temporaire**

Exécuter après avoir résolu le lien vers la cible attendue du worktree :

```bash
test "$(readlink /Users/sebastien/.agents/skills/memory-governance)" = \
  "/Users/sebastien/.dotfiles/.worktrees/pr-249-memory-defects/harness/skills/memory-governance"
unlink /Users/sebastien/.agents/skills/memory-governance
```

Résultat attendu : destination absente ; les autres skills utilisateur sont intactes.

- [ ] **Step 4: Noter routing et comportement séparément**

Pour q8 de comparaison, exiger une lecture du fichier de skill et noter le comportement
candidate-only. Pour q3/q4, exiger l'absence de lecture : ce sont des contrôles négatifs de
routage, non des verdicts candidate-only. Noter les résultats exacts :

```text
q8 comparaison -> status: candidate, aucune persistance
q3/q4 comparaison -> aucune lecture de memory-governance/SKILL.md
q9/q10, chemins de rejet, révision périmée/fraîcheur -> non exercés dans ce run
```

- [ ] **Step 5: Écrire le rapport versionné**

Créer `docs/memory-governance-validation.md` en français avec la date, l'architecture macOS, la version Codex, les cibles non exercées, la méthode exacte, la table contrôle/comparaison de tous les réplicats et une section de limites. Ne pas inclure de transcripts bruts, prompts privés, credentials ou taux d'activation inventés.

- [ ] **Step 6: Commit**

```bash
git add docs/memory-governance-validation.md
git commit -m "docs(memory): record Codex behavior evaluation"
```

### Tâche 4 : doctor, index et barrière de dépôt

**Fichiers :**

- Vérifier : `harness/skills/memory-governance/SKILL.md`
- Vérifier : `harness/skills/memory-governance/evals/trigger-queries.json`
- Vérifier : `harness/skills/README.md`
- Vérifier : `tooling/deployment-links.test.ts`

**Interfaces :**

- Consomme : contrat et preuve des tâches 2–3.
- Produit : head local vérifié, prêt pour revue indépendante et push unique.

- [ ] **Step 1: Exécuter le doctor complet**

Exécuter le contrat doctor manuel puisque `skills-ref` est absent : frontmatter, ordre du body, au moins trois gotchas et contraintes, ressources, shell templated, schéma d'eval, entrée README unique et cible de déploiement. Résultat attendu : PASS local, validation standard explicitement indisponible.

```bash
command -v skills-ref || true
test "$(rg -c '^\| `memory-governance`' harness/skills/README.md)" = 1
test "$(wc -l < harness/skills/memory-governance/SKILL.md | tr -d ' ')" -lt 500
test -z "$(rg -n '\$(\{?[0-9]|@|ARGUMENTS)' \
  harness/skills/memory-governance/SKILL.md || true)"
jq -e '.skill == "memory-governance" and (.version | type == "string") and
  (all(.queries[]; (.query | type == "string" and length > 0) and
    (.should_activate | type == "boolean") and
    (.reason | type == "string" and length > 0))) and
  (any(.queries[]; .should_activate)) and
  (any(.queries[]; .should_activate == false))' \
  harness/skills/memory-governance/evals/trigger-queries.json
make -n /Users/sebastien/.agents/skills/memory-governance
```

- [ ] **Step 2: Régénérer l'index deux fois**

Exécuter la procédure déterministe `skill-manager sync-index`, capturer le hash après chaque run et exiger l'identité byte à byte. Résultat attendu : aucune édition manuelle du README et aucun diff au second run.

```bash
prettier --write harness/skills/README.md
shasum -a 256 harness/skills/README.md > /private/tmp/memory-skills-index.sha256
prettier --write harness/skills/README.md
shasum -a 256 -c /private/tmp/memory-skills-index.sha256
git diff --exit-code -- harness/skills/README.md
```

- [ ] **Step 3: Exécuter les barrières ciblées**

Exécuter :

```bash
bun test tooling/deployment-links.test.ts
bun run lint
bun run typecheck
bun run format:typescript:check
git diff --check e1ffc64a7dff80c0b74a74d6b055ee3705dffda6..HEAD
```

Résultat attendu : tous verts sur macOS arm64, avec les limites de plateforme nommées.

- [ ] **Step 4: Exécuter la suite complète**

Exécuter :

```bash
bun test
```

Résultat attendu : aucun échec ; les skips existants sont consignés.

- [ ] **Step 5: Vérifier taille, commentaires et propreté**

Exécuter :

```bash
wc -l harness/skills/memory-governance/SKILL.md \
  harness/skills/memory-governance/evals/trigger-queries.json \
  docs/memory-governance-validation.md
git diff --check e1ffc64a7dff80c0b74a74d6b055ee3705dffda6..HEAD
git status --short
```

Résultat attendu : fichiers sous leurs triggers applicables, aucun commentaire de code ajouté, worktree propre hors fichiers suivis attendus.

- [ ] **Step 6: Revue indépendante puis push unique**

Déléguer `pr-verdict` sur le head commité exact. Si aucun blocage ne reste, pousser normalement avec :

```bash
git push git@github.com:SebastienElet/dotfiles.git \
  HEAD:refs/heads/codex/memory-governance
```

Résultat attendu : aucun force-push, head distant égal au head revu, checks GitHub verts, réparation consignée sur la PR.
