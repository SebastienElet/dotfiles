# Audit des skills — issue #412

Audit du 7 octobre 2026, complété après rebase sur `672907f1e6b431e6bad946f5dc8eeada43b9b938` :
39 skills personnelles dans `harness/skills/` et 6 skills projet dans `.agents/skills/`.
Les anciens comptes rendus d'évaluation restent des archives, pas des instructions actuelles.

## Constats et traitement

| Source                                              | Instruction obsolète                                                           | Correction                                                                                                                                    |
| --------------------------------------------------- | ------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| `harness/skills/skill-manager/references/create.md` | Ajouter des cibles feuilles `Makefile` pour une skill personnelle.             | Déclarer les installations dans `home/.arnes.yaml` ; les tâches Moon existantes déploient le manifeste via `tooling/install-agent-skills.ts`. |
| `harness/skills/linear-start/SKILL.md`              | Orienter une issue inexécutable vers la future skill `issue-shaping`, absente. | Proposer une demande distincte à `linear-issue-spec`, uniquement sur demande explicite de cadrage.                                            |

Aucune suppression de skill entière n'est justifiée par cet audit. `tdd`, `diagnosing-bugs`,
`code-review` et `proof-integrity-review` conservent leur rôle : AssertLedger qualifie un test
contre une faute déclarée et ne remplace ni l'implémentation, ni le diagnostic, ni la revue,
ni le contrôle des mécanismes de vérification.

## Périmètre examiné

- Lecture des 45 `SKILL.md` préexistants, de leurs déclencheurs, contraintes et références pertinentes,
  y compris la nouvelle skill `herdr-issue-worktree` et les changements de `output-discipline` sur `main`.
- Contrôle des frontmatters, sections locales, placeholders shell, index et références locales.
- Contrôle des 28 fichiers `evals/trigger-queries.json` et des 6 extensions d'activation manuelle.
- Vérification des trois adapters projet et des déclarations d'installation Arnes.
- Recherche des instructions Make et des références vers des skills ou ressources disparues.

Les collections ne partagent aucun slug. Les index correspondent aux frontmatters de la base.
Les références locales contrôlées sont présentes et les adapters projet ciblent
`../.agents/skills`. Aucun autre workflow entièrement obsolète n'a été identifié.
Ces constats de lecture ne démontrent pas l'exécution de chaque workflow ou son activation réelle.
`skills-ref` est absent ; la validation standard automatisée est indisponible et n'a pas été
installée pour cet audit. Les scénarios d'activation n'ont pas été exécutés avec un agent ou un LLM.

## Intégration AssertLedger

La skill personnelle `assertledger` est maintenue localement conformément aux ADR-029/040,
avec des installations pour Claude Code, Codex et Cursor. La tâche `harness:assertledger`
installe le CLI npm publié en version 1.4.0, dépend du Node géré par Volta, partage son mutex,
et appartient au profil minimal. Elle n'initialise aucun projet et ne configure aucun MCP.

Le parcours Git publié exige un candidat JavaScript `node:test` commité sans dépendances runtime,
des tests de base inchangés et des ensembles de chemins identiques hors candidat.
Le diagnostic statique de ce checkout retourne `UNSUPPORTED_REPOSITORY_SYMLINK`.
La skill conserve ce refus et documente les limites ; elle ne qualifie pas les suites
TypeScript/Bun ou Rust des dotfiles avec `check`.

Sources : [issue #412](https://github.com/SebastienElet/dotfiles/issues/412),
[parcours Git 1.4.0](https://github.com/hoklims/assertledger/blob/v1.4.0/docs/git-regression.md),
[référence 1.4.0](https://github.com/hoklims/assertledger/blob/v1.4.0/docs/reference.md),
ADR-017, ADR-028, ADR-029, ADR-038 et ADR-040 acceptées dans `docs/adr/`.
