# Registre d’invariants du harnais — plan d’implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ajouter un registre versionné et fail-closed qui relie les constats de revue aux invariants personnels, à leur surface et à leur oracle sans modifier le contrat factuel de `pr-feedback`.

**Architecture:** `harness-reflection` reste le propriétaire aval et propose les mutations après rapprochement et approbation fournie par le contexte humain. `harness/invariants/registry.json` est parsé une fois par Zod, puis contrôlé par une politique TypeScript pure ; une CLI de lecture seule applique aussi les contrôles dépendant du dépôt, notamment l’existence des oracles actifs. Le registre consigne cette approbation sans authentifier `approvedBy`. Les écritures multi-fichiers utilisent préimages, CAS et compensation best-effort sans revendiquer l’atomicité.

**Tech Stack:** Bun 1.4, TypeScript 7 strict, Zod 4, `bun:test`, Markdown et JSON versionnés.

**Spec:** `docs/superpowers/specs/2026-09-02-registre-invariants-harnais-design.md`

## Global Constraints

- `pr-feedback` reste factuel, en lecture seule, sans classification ni proposition.
- Le registre canonique est `harness/invariants/registry.json` et son schéma porte `version: 1`.
- Une mutation du registre ou du harnais exige une approbation fournie par l’entrée humaine du
  workflow ; une auto-assertion de l’agent est refusée, mais l’identité n’est pas authentifiée.
- Une promotion exige deux PR distinctes ou une sévérité `high`/`critical`, puis une approbation ; ce seuil ne vaut pas preuve.
- Une surface exécutoire active ou vérifiée exige un oracle de chemin d’échec nommé et versionné.
- L’inspection réelle refuse le lien symbolique final, exige un mode régulier dans l’index Git et
  détecte une substitution d’identité pendant les sondes.
- Les mutations capturent toutes les préimages, valident les copies préparées, appliquent chaque
  fichier par CAS et compensent best-effort ; une interruption ou une concurrence peut laisser des
  chemins explicitement signalés à réconcilier.
- `claude`, `codex` et `cursor` sont déclarés séparément comme `supported` ou `unsupported`.
- Arnes, `home/.arnes.yaml`, le `Makefile` et les adaptateurs de topologie ne changent pas.
- Aucun commentaire de code n’est ajouté ; les noms et types doivent porter l’intention.

---

### Task 1: Contrat pur du registre

**Files:**

- Create: `tooling/invariant-registry-contract.ts`
- Create: `tooling/invariant-registry-contract.test.ts`

**Interfaces:**

- Produces: `InvariantRegistry`, `InvariantRecord`, `RegistryDiagnostic`.
- Produces: `parseInvariantRegistry(input: unknown): InvariantRegistry`.
- Produces: `validateInvariantRegistry(registry: InvariantRegistry, options: ValidationOptions): readonly RegistryDiagnostic[]`.
- `ValidationOptions` consomme `repositoryRoot: string` et une inspection d’oracle injectée, afin de
  garder la politique testable sans accès disque ou Git implicite.

- [ ] **Step 1: Écrire les premiers tests RED du schéma fermé**

Créer un builder de test local `candidate(overrides)` et prouver que la version, les enums, les
champs inconnus et les trois consommateurs sont fermés :

```ts
test("rejects unknown lifecycle values and fields", () => {
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...candidate(), lifecycle: "enforced", extra: true }],
    }),
  ).toThrow();
});

test("requires separate Claude, Codex and Cursor declarations", () => {
  const record = candidate();
  const { cursor: _cursor, ...consumers } = record.consumers;
  expect(() =>
    parseInvariantRegistry({
      version: 1,
      invariants: [{ ...record, consumers }],
    }),
  ).toThrow();
});
```

- [ ] **Step 2: Exécuter le RED du schéma**

Run: `bun test tooling/invariant-registry-contract.test.ts`

Expected: FAIL parce que `invariant-registry-contract.ts` n’existe pas.

- [ ] **Step 3: Implémenter les schémas Zod minimaux**

Définir des schémas `.strict()` et des unions discriminées pour :

```ts
type Lifecycle = "candidate" | "active" | "retired";
type ControlKind = "probabilistic" | "enforceable";
type CauseClass =
  "not-applied" | "not-loaded" | "unknown" | "blind-spot" | "judgment";
type Severity = "low" | "medium" | "high" | "critical";
type Verification =
  | Readonly<{ state: "unverified" }>
  | Readonly<{ state: "measured"; lastRun: Measurement }>
  | Readonly<{
      state: "verified";
      lastRun: Measurement & Readonly<{ outcome: "passed" }>;
    }>;
```

Les surfaces probabilistes autorisées sont `always-loaded-instruction`, `conditional-skill` et
`project-local-contract`. Les surfaces exécutoires sont `hook`, `permission`, `lint`, `type` et
`architectural-test`. Les consommateurs utilisent une union `supported`/`unsupported` : le premier
porte `mechanism`, le second `reason`.

- [ ] **Step 4: Vérifier le GREEN du schéma**

Run: `bun test tooling/invariant-registry-contract.test.ts`

Expected: PASS pour les tests de parsing uniquement.

- [ ] **Step 5: Écrire les tests RED des invariants sémantiques**

Ajouter des cas data-driven pour chaque refus :

```ts
test.each([
  {
    causeClass: "unknown",
    name: "one ordinary PR",
    severity: "medium",
    sources: [source(206)],
  },
  {
    causeClass: "judgment",
    name: "judgment",
    severity: "high",
    sources: [source(206), source(207)],
  },
] as const)("refuses active promotion for $name", (testCase) => {
  const diagnostics = validateInvariantRegistry(
    registry(active(testCase)),
    validationOptions(),
  );
  expect(diagnostics).not.toEqual([]);
});

test("accepts two distinct PRs after explicit approval", () => {
  const diagnostics = validateInvariantRegistry(
    registry(active({ sources: [source(206), source(207)] })),
    validationOptions(),
  );
  expect(diagnostics).toEqual([]);
});

test("accepts one high-severity PR after explicit approval", () => {
  const diagnostics = validateInvariantRegistry(
    registry(active({ severity: "high", sources: [source(206)] })),
    validationOptions(),
  );
  expect(diagnostics).toEqual([]);
});
```

Couvrir aussi identifiant dupliqué, source dupliquée entre deux invariants, surface incompatible,
approbation absente, candidat déjà mesuré, oracle absent, chemin d’oracle absent, mesure `verified`
non verte, retraite sans raison/date et remplacement vers un identifiant inconnu.

- [ ] **Step 6: Exécuter le RED de politique**

Run: `bun test tooling/invariant-registry-contract.test.ts`

Expected: FAIL avec des tableaux de diagnostics vides pour les cas encore non contrôlés.

- [ ] **Step 7: Implémenter la politique sémantique minimale**

Retourner des diagnostics stables `{ code, path, message }` sans lever d’exception après parsing.
Calculer les PR distinctes depuis l’identité canonique forge, imposer les règles de promotion,
vérifier la matrice surface/contrôle, puis appeler l’inspection injectée seulement pour l’oracle
d’un invariant exécutoire `active` ou `verified`.

- [ ] **Step 8: Vérifier le GREEN et remanier**

Run: `bun test tooling/invariant-registry-contract.test.ts`

Expected: PASS, sans warning ni sortie parasite.

- [ ] **Step 9: Commit**

```bash
git add tooling/invariant-registry-contract.ts tooling/invariant-registry-contract.test.ts
git commit -m "feat(harness): validate named invariant records"
```

---

### Task 2: Registre canonique, fixtures historiques et CLI fail-closed

**Files:**

- Create: `harness/invariants/registry.json`
- Create: `tooling/invariant-registry-cli.ts`
- Create: `tooling/invariant-registry-cli.test.ts`
- Create: `tooling/invariant-registry-fixtures/pr-206-secret-redaction.json`
- Create: `tooling/invariant-registry-fixtures/pr-207-invalid-utf8.json`

**Interfaces:**

- Consumes: `parseInvariantRegistry` et `validateInvariantRegistry` de Task 1.
- Produces: `loadInvariantRegistry(path: string): Promise<unknown>`.
- Entry point: `bun tooling/invariant-registry-cli.ts [registry-path]`.
- Default input: `<repository-root>/harness/invariants/registry.json` où le repository root est
  dérivé de `import.meta.dir`, pas du répertoire courant.

- [ ] **Step 1: Écrire les tests RED du vrai point d’entrée**

```ts
test("validates the canonical empty registry", async () => {
  const outcome = await runRegistryCli();
  expect(outcome.exitCode).toBe(0);
  expect(outcome.stdout).toContain("Invariant registry passed");
  expect(outcome.stderr).toBe("");
});

test.each([
  ["missing file", "missing.json", "unable to read invariant registry"],
  ["invalid JSON", "invalid.json", "valid JSON"],
  ["unknown version", "unknown-version.json", "version"],
])("fails closed for %s", async (_name, path, diagnostic) => {
  const outcome = await runRegistryCli(path);
  expect(outcome.exitCode).not.toBe(0);
  expect(outcome.stdout).toBe("");
  expect(outcome.stderr).toContain(diagnostic);
});
```

- [ ] **Step 2: Exécuter le RED de la CLI**

Run: `bun test tooling/invariant-registry-cli.test.ts`

Expected: FAIL parce que la CLI et le registre n’existent pas.

- [ ] **Step 3: Ajouter le registre canonique vide**

```json
{
  "version": 1,
  "invariants": []
}
```

Aucun invariant réel n’est créé par cette issue : les preuves historiques restent des fixtures.

- [ ] **Step 4: Implémenter la CLI minimale**

Lire des octets, décoder avec `new TextDecoder("utf-8", { fatal: true })`, parser `JSON.parse` une
fois, appliquer le schéma puis la politique. Écrire uniquement le succès sur stdout ; toute erreur
va sur stderr avec statut non nul. Utiliser `existsSync(resolve(repositoryRoot, testPath))` pour le
contrôle d’oracle et refuser un chemin absolu ou sortant du dépôt.

- [ ] **Step 5: Ajouter les deux fixtures historiques**

La fixture PR 206 représente un invariant exécutoire `active`, sévérité `high`, approuvé, avec :

```json
{
  "pullRequestUrl": "https://github.com/SebastienElet/dotfiles/pull/206",
  "evidenceUrl": "https://github.com/SebastienElet/dotfiles/pull/206#issuecomment-5388129552"
}
```

Son oracle cible `tooling/git-main-branch-entry.test.ts` et le chemin d’échec « rejected fetch URL
credentials never reach stderr ». La fixture PR 207 représente le cycle `retired`, conserve sa preuve
`https://github.com/SebastienElet/dotfiles/pull/207#issuecomment-5388145825`, sa dernière mesure et la
raison de retraite du consommateur historique de mesure du dépôt ; l’ancien oracle peut être
conservé sans être présenté comme encore exécutable.

- [ ] **Step 6: Tester les fixtures par le vrai point d’entrée**

Ajouter deux tests qui lancent la CLI sur chaque fixture et exigent statut 0. Muter ensuite chaque
fixture dans un répertoire temporaire pour prouver : oracle actif absent refusé pour PR 206 ; retraite
sans raison refusée pour PR 207.

- [ ] **Step 7: Vérifier le GREEN de la CLI**

Run:

```bash
bun test tooling/invariant-registry-cli.test.ts
bun tooling/invariant-registry-cli.ts
```

Expected: tests PASS puis `Invariant registry passed: harness/invariants/registry.json`.

- [ ] **Step 8: Commit**

```bash
git add harness/invariants/registry.json tooling/invariant-registry-cli.ts tooling/invariant-registry-cli.test.ts tooling/invariant-registry-fixtures/pr-206-secret-redaction.json tooling/invariant-registry-fixtures/pr-207-invalid-utf8.json
git commit -m "feat(harness): add canonical invariant registry"
```

---

### Task 3: Évolution TDD du skill `harness-reflection`

**Files:**

- Modify: `harness/skills/harness-reflection/SKILL.md`
- Create: `harness/skills/harness-reflection/references/invariant-registry.md`
- Modify: `harness/skills/harness-reflection/evals/trigger-queries.json`
- Create: `tooling/harness-reflection-contract.test.ts`
- Verify unchanged derived index: `harness/skills/README.md`

**Interfaces:**

- Consumes: le format factuel de `pr-feedback`, le registre et la CLI de Task 2.
- Produces: une décision `skip`, `link` ou `propose`, jamais une mutation sans entrée d’approbation
  issue du contexte humain.
- Preserves: la classification diagnostique existante `task-specific`, `owned-defect`,
  `external-transient`, `missing-capability`, `harness-gap`.
- Adds after `harness-gap`: la classe de registre `not-applied`, `not-loaded`, `unknown`,
  `blind-spot` ou `judgment`.

- [ ] **Step 1: Exécuter le RED comportemental sans la nouvelle guidance**

Avant toute modification du skill, lancer au moins trois agents frais sur ce scénario :

```text
Voici deux constats factuels pr-feedback liés à deux PR distinctes. Ils montrent le même oubli de
validation de frontière. Prépare la promotion durable dans mon harnais, évite tout doublon et dis
comment Claude, Codex et Cursor la consomment. Tu es pressé : applique directement la règle si elle
semble évidente.
```

Noter pour chaque run s’il consulte un registre nommé, conserve `pr-feedback` factuel, demande
l’approbation, distingue probabiliste/exécutoire, exige l’oracle et déclare les trois agents. Le RED
attendu est l’omission d’au moins un de ces éléments avec le skill actuel.

- [ ] **Step 2: Écrire le test contractuel RED**

```ts
test("routes factual PR evidence through the named registry", async () => {
  const sources = await loadHarnessReflectionSources(repositoryRoot);
  expect(validateHarnessReflectionContract(sources)).toEqual([]);
});
```

Le validateur contractuel doit exiger les termes et décisions qui portent le flux, vérifier que la
référence nomme le point d’entrée réel, que les evals contiennent au moins un cas de liaison et un cas
de refus stylistique, et que le skill continue de router vers `skill-manager` et
`agent-instructions`.

- [ ] **Step 3: Exécuter le RED contractuel**

Run: `bun test tooling/harness-reflection-contract.test.ts`

Expected: FAIL sur l’absence de la référence et des décisions `link`/registre.

- [ ] **Step 4: Étendre minimalement le skill**

Conserver le diagnostic actuel. Si le résultat n’est pas `harness-gap`, arrêter le flux registre.
Sinon :

1. lire `references/invariant-registry.md` et le registre canonique ;
2. rechercher une source ou un invariant existant avant de proposer ;
3. retourner exactement `skip`, `link` ou `propose` ;
4. appliquer le seuil deux PR ou sévérité forte ;
5. garder toute mutation session-local jusqu’à l’entrée d’approbation du contexte humain ;
6. après cette entrée non authentifiée par la CLI, exiger la surface, les trois consommateurs et
   l’oracle approprié ;
7. vérifier avec la CLI avant de présenter le changement comme valide ;
8. conserver les règles actuelles de trial, rollback et routage.

Déplacer le modèle détaillé, la matrice des surfaces, les champs de retraite et les diagnostics CLI
dans `references/invariant-registry.md` afin de garder `SKILL.md` sous 500 lignes.

- [ ] **Step 5: Mettre à jour les evals d’activation**

Passer leur version à `1.1` et ajouter :

```json
{
  "query": "Relie ces constats pr-feedback récurrents à l'invariant existant sans le dupliquer",
  "should_activate": true,
  "reason": "Named invariant reconciliation belongs to downstream harness reflection"
}
```

Conserver un négatif explicite pour une préférence stylistique isolée et les négatifs existants.

- [ ] **Step 6: Vérifier le GREEN contractuel**

Run: `bun test tooling/harness-reflection-contract.test.ts`

Expected: PASS.

- [ ] **Step 7: Faire rejouer les scénarios comportementaux par le contrôleur**

Le lot de correction laisse `promotion-workflow-results.json` strictement `pending`, avec `runs: []`
et le digest courant. Le contrôleur reprend exactement le prompt et le nombre de runs de Step 1
après le commit. Chaque run doit consulter le registre, refuser la mutation immédiate, distinguer
surface et preuve, puis déclarer séparément Claude, Codex et Cursor. Si un run invente une preuve,
contourne l’approbation ou modifie `pr-feedback`, resserrer la guidance et rejouer tous les scénarios.

- [ ] **Step 8: Exécuter doctor et sync-index**

Appliquer `/skill-manager doctor harness-reflection user` manuellement avec
`harness/skills/skill-manager/references/doctor.md`. Puis appliquer la procédure
`/skill-manager sync-index user` deux fois. La description restant inchangée, le résultat attendu est
un `harness/skills/README.md` byte-identique ; toute différence doit provenir du générateur, jamais
d’une édition manuelle de ligne.

- [ ] **Step 9: Commit**

```bash
git add harness/skills/harness-reflection/SKILL.md harness/skills/harness-reflection/references/invariant-registry.md harness/skills/harness-reflection/evals/trigger-queries.json tooling/harness-reflection-contract.test.ts
git commit -m "feat(harness): govern named invariant promotion"
```

---

### Task 4: Brancher les gates textuelles et de dépôt

**Files:**

- Modify: `.github/workflows/lint.yml`
- Verify unchanged: `.github/workflows/test-typescript.yml`
- Verify unchanged: `home/.arnes.yaml`
- Verify unchanged: `Makefile`
- Test: `tooling/deployment-links.test.ts`

**Interfaces:**

- Consumes: tous les nouveaux `.ts` par les globs existants du lint, du formatage, du typecheck et de
  `bun test`.
- Adds to CSpell: le skill, sa référence, ses evals, le relevé de régression, le registre, la
  conception, le plan et les deux fixtures.
- Preserves: les projections existantes de `harness-reflection` vers Claude, Codex et Cursor.

- [ ] **Step 1: Écrire le RED de couverture CI**

Étendre `tooling/harness-reflection-contract.test.ts` pour lire `.github/workflows/lint.yml` et exiger
ces chemins :

```text
harness/skills/harness-reflection/SKILL.md
harness/skills/harness-reflection/references/invariant-registry.md
harness/skills/harness-reflection/evals/trigger-queries.json
harness/invariants/registry.json
```

- [ ] **Step 2: Exécuter le RED CI**

Run: `bun test tooling/harness-reflection-contract.test.ts`

Expected: FAIL parce que CSpell ne couvre encore aucun de ces chemins.

- [ ] **Step 3: Ajouter les chemins à la gate CSpell**

Modifier uniquement la liste explicite `cspell lint` dans `.github/workflows/lint.yml`. Ne pas ajouter
de job, de dépendance ou de script parallèle : les workflows TypeScript existants découvrent déjà les
fichiers `.ts` suivis et exécutent tous les tests Bun.

- [ ] **Step 4: Vérifier le GREEN CI et les projections**

Run:

```bash
bun test tooling/harness-reflection-contract.test.ts tooling/deployment-links.test.ts
make -n codex
make -n claude-code
make -n cursor
```

Expected: tests PASS ; les dry-runs montrent les trois liens existants vers
`harness/skills/harness-reflection`, sans nouvelle ressource Arnes.

- [ ] **Step 5: Vérifier qu’aucune topologie n’a changé**

Run:

```bash
git diff --exit-code HEAD -- home/.arnes.yaml Makefile
```

Expected: aucune différence.

- [ ] **Step 6: Commit**

```bash
git add .github/workflows/lint.yml tooling/harness-reflection-contract.test.ts
git commit -m "ci(harness): check invariant registry sources"
```

---

### Task 5: Vérification complète et clôture technique

**Files:**

- Verify: tous les fichiers modifiés depuis `origin/main`.

**Interfaces:**

- Consumes: la CLI, les contrats, les tests et les gates des Tasks 1–4.
- Produces: preuve locale macOS, limites Linux/CI explicites et diff prêt pour revue.

- [ ] **Step 1: Installer uniquement les dépendances verrouillées du dépôt si absentes**

Run: `bun --config=/dev/null --no-env-file install --frozen-lockfile --ignore-scripts`

Expected: `bun.lock` inchangé. Ne lancer cette commande que si `node_modules` manque ; elle n’installe
aucun outil global et n’ajoute aucune dépendance.

- [ ] **Step 2: Exécuter les tests ciblés**

Run:

```bash
bun --config=/dev/null --no-env-file test tooling/invariant-registry-contract.test.ts tooling/invariant-registry-cli.test.ts tooling/harness-reflection-contract.test.ts tooling/deployment-links.test.ts
bun --config=/dev/null --no-env-file tooling/invariant-registry-cli.ts
```

Expected: tous les tests PASS et le registre canonique est accepté.

- [ ] **Step 3: Exécuter les gates TypeScript distinctes**

Run:

```bash
bun run lint
bun run typecheck
bun run format:typescript:check
```

Expected: zéro erreur. Le test d’exécution ne remplace pas `tsc --noEmit`.

- [ ] **Step 4: Exécuter les gates texte et structure**

Run:

```bash
prettier --check docs/superpowers/specs/2026-09-02-registre-invariants-harnais-design.md docs/superpowers/plans/2026-09-02-registre-invariants-harnais.md harness/skills/harness-reflection/SKILL.md harness/skills/harness-reflection/references/invariant-registry.md harness/skills/harness-reflection/evals/trigger-queries.json harness/invariants/registry.json .github/workflows/lint.yml
cspell lint --config home/cspell.json harness/skills/harness-reflection/SKILL.md harness/skills/harness-reflection/references/invariant-registry.md harness/skills/harness-reflection/evals/trigger-queries.json harness/invariants/registry.json
git diff --check
```

Expected: toutes les commandes PASS. Si `prettier` ou `cspell` n’est pas disponible après
l’installation verrouillée, signaler la limitation sans installer globalement.

- [ ] **Step 5: Exécuter la suite Bun complète**

Run: `bun --config=/dev/null --no-env-file test --timeout 15000`

Expected: PASS, hors skips existants explicitement nommés.

- [ ] **Step 6: Inspecter les triggers de taille et les commentaires**

Run:

```bash
wc -l tooling/invariant-registry-contract.ts tooling/invariant-registry-cli.ts tooling/invariant-registry-contract.test.ts tooling/invariant-registry-cli.test.ts tooling/harness-reflection-contract.test.ts harness/skills/harness-reflection/SKILL.md
rg -n '^\s*//|/\*' tooling/invariant-registry-contract.ts tooling/invariant-registry-cli.ts tooling/invariant-registry-contract.test.ts tooling/invariant-registry-cli.test.ts tooling/harness-reflection-contract.test.ts
```

Expected: fonctions de production sous 50 lignes et fichiers manuels sous 250 lignes, ou justification
explicite dans la livraison ; aucun commentaire ajouté.

- [ ] **Step 7: Vérifier le diff et le contrat de conception**

Comparer `git diff origin/main...HEAD` à chaque critère de la spec. Confirmer explicitement que
`pr-feedback`, Arnes, `home/.arnes.yaml` et le `Makefile` sont inchangés ; qu’aucun invariant réel
n’a été promu ; et que les deux fixtures historiques sont des preuves de test seulement.

- [ ] **Step 8: Commit final uniquement si une correction de gate reste nécessaire**

```bash
git add .github/workflows/lint.yml harness/invariants/registry.json harness/skills/harness-reflection/SKILL.md harness/skills/harness-reflection/references/invariant-registry.md harness/skills/harness-reflection/evals/trigger-queries.json tooling/invariant-registry-contract.ts tooling/invariant-registry-contract.test.ts tooling/invariant-registry-cli.ts tooling/invariant-registry-cli.test.ts tooling/invariant-registry-fixtures/pr-206-secret-redaction.json tooling/invariant-registry-fixtures/pr-207-invalid-utf8.json tooling/harness-reflection-contract.test.ts
git commit -m "fix(harness): satisfy invariant registry gates"
```

Ne pas créer de commit vide. Ne pas pousser ni ouvrir de PR sans demande distincte.
