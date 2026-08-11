# Verdict template

Fill this into `verdict.md`, then publish with the commands in `references/forges.md`. A slot marked
REQUIRED that cannot be filled means the phase it belongs to was not done — go back to it instead of
publishing an incomplete verdict. This file is English; the verdict itself follows the language of
the PR.

## Skeleton

```text
<!-- merge-verdict:<pr>:<head-sha-12> -->
## Independent verdict — <changes required | approved with reservations | approved>

<Anchor sentence — REQUIRED. Whose work, on which head SHA, against which base: the real one,
naming the parent PR when the branch is stacked. Then the CI state, open tasks and conflicts,
or "none observed".>

<Blocking paragraph — one clause per blocker: the mechanism, then the invariant it breaks. Close
with one sentence stating what must become true to lift them. Omit this paragraph entirely when
the verdict is "approved".>

<Barrier paragraph — REQUIRED. Open with "Authenticated local validation on this exact head:"
and give counts, never adjectives. Then, in the same paragraph, REQUIRED: what this evidence does
not cover. The verdict is invalid without that second half.>

<Non-blocking remarks — one line each, each prefixed "Non-blocking:". Drop the section rather
than pad it.>

Fix: <url>                REQUIRED when the verdict blocks
Re-review: <url>          REQUIRED when the verdict blocks
Initial review: <url>     REQUIRED when a previous verdict exists on this PR

<Closing sentence — REQUIRED, one line, executable:
  changes required          → "Do not approve or merge this head."
  approved with reservations → "Mergeable once <criterion>."
  approved                   → "Approved on this head.">
```

## Filled example

A *changes required* verdict on a French PR. Every identifier and figure below is invented — a
committed skill must carry nothing from the repositories it was exercised on. Note what the barrier
paragraph does: it gives numbers, then immediately spends a sentence dismantling its own green.

```text
<!-- merge-verdict:1042:a1b2c3d4e5f6 -->
## Verdict indépendant — changements requis

Review de la PR #1042 sur a1b2c3d4e5f6, base empilée feat/ledger-read-side@9f8e7d6c5b4a.
Pipeline #318 vert, aucune tâche ni conflit observé.

Blocages : le snapshot et les contrôles précèdent la transaction de clôture, donc une écriture
concurrente peut disparaître du successeur ; deux clôtures simultanées peuvent créer deux
successeurs car le retry ne relit pas le résultat gagnant et les unicités nécessaires manquent.
Levée : des tests concurrents sur PostgreSQL, un contrat explicite pour les contrôles différés et
pour l'autorisation tenant, puis un reciblage vers develop après la fusion de la PR parente.

Validation locale authentifiée sur ce head exact : lint global vert (18/18 builds, 0 erreur, seuil
de 145 avertissements respecté), typecheck des deux paquets touchés vert, 7/7 tests unitaires de
clôture verts. Ces tests restent séquentiels : ils ne couvrent aucune des courses concurrentes qui
motivent les blocages ci-dessus.

Non bloquant : la documentation des codes 409/412 ne correspond plus au comportement réel.

Correction : https://tracker.example/ISSUE-158
Re-review : https://tracker.example/ISSUE-159
Review initiale : https://tracker.example/ISSUE-155

Ne pas approuver ni fusionner ce head.
```

## Self-check before publishing

- The marker is the first line, and its SHA is the head you actually checked out.
- Every clause in the blocking paragraph names a sequence of steps, not a quality judgement.
- The barrier paragraph contains digits, and a sentence saying what those digits do not prove.
- The closing sentence tells the reader what to do, not how the reviewer feels.
- Total under about thirty lines. Past that, preferences have leaked into the blocking section.
