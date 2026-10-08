# Sélection CI — #407

Cette matrice reçoit les contributions des tranches de
[#407](https://github.com/SebastienElet/dotfiles/issues/407). Les cinq familles Rust
ci-dessous relèvent de [#420](https://github.com/SebastienElet/dotfiles/issues/420).
Les autres familles restent à documenter par leurs tranches respectives ; le
[relevé #272](moon-gates-272.md) est historique.

## Contrat commun

Les ADR acceptées [001](adr/001-makefile-installateur.md),
[023](adr/023-ci-lint-et-installation.md) et
[041](adr/041-frontiere-automatisation-typescript-rust.md) restent en vigueur.
`tooling:smoke-minimal` conserve son job macOS à chaque PR et push `main`,
indépendamment de la sélection spécialisée.

Chaque workflow Rust commence par `select` sur Ubuntu : checkout complet et
blobless, puis `moonrepo/setup-toolchain` et
`moon query tasks --affected remote --upstream none --downstream none`, filtré
sur son projet et ses tâches. Les expressions natives Actions lisent la sortie
JSON de Moon. Un projet absent du résultat valide évite l'allocation de sa suite ;
une commande en échec ou une sortie JSON invalide fait échouer la sélection.
Aucun filtre de chemins parallèle ne décide à la place de Moon.

`MOON_BASE` est le SHA de base de la PR, ou `before` pour un push `main` ;
`MOON_HEAD` est le SHA checkout (`github.sha`, merge synthétique pour une PR).
Sélection et exécution utilisent ces mêmes révisions. Le push couvre ainsi tous
ses commits, pas uniquement le dernier. Une révision introuvable reste un échec.

## Matrice Rust

Les entrées communes sont les manifests/lockfiles Cargo, `build*.rs`,
`src/**/*`, `tests/**/*` (fixtures incluses), examples et benches, `moon.yml` et
`.cargo/**/*` du projet ; à la racine : `.cargo/**/*`, `clippy.toml`,
`rustfmt.toml`, `rust-toolchain.toml`, `.prototools`, `moon.yml`, `LICENSE` et
`tooling/RUST.md`. Moon prend aussi en compte sa configuration workspace,
toolchains et héritage Rust. Chaque famille déclare son propre workflow.

| Workflow               | Projet / tâches sélectionnées                               | Entrées transversales supplémentaires                                                                                                                                                                                                                                   | Runners spécialisés / statuts             |
| ---------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| Arnes tests            | `arnes:{fmt,clippy,check,test,doc}`                         | `home/.arnes.yaml`, `harness/**/*`, `home/moon.yml`, héritages Codex/Claude/JavaScript, helpers `assemble-agent-instructions.ts`, `install-agent-skills.ts`, `deploy-link.ts`, `package.json`, `bun.lock` pour les tests du dépôt                                       | Ubuntu / `arnes`                          |
| Bitbucket Linear tests | `bitbucket-linear-sync:{fmt,clippy,check,test,doc}`         | `home/.config/fish/conf.d/aliases.fish`                                                                                                                                                                                                                                 | macOS et Ubuntu / `bitbucket-linear (os)` |
| Agent Handoff tests    | `agent-handoff:{fmt,clippy,check,test,doc,deployment-test}` | Déploiement : helpers `deployment-{agent-handoff.test,moon-runner,moon-test-support,test-support,test-command}.ts`, `deploy-link.ts`, `clean-deployment*.ts`, `clean-deployment-worker-inspect.js`, `home/.arnes.yaml`, `package.json`, `bun.lock`, héritage JavaScript | macOS et Ubuntu / `agent-handoff (os)`    |
| Agent Memory tests     | `agent-memory:{fmt,clippy,check,test,doc,deployment-test}`  | Même frontière de déploiement avec `deployment-agent-memory.test.ts` ; également `harness/moon.yml`, `harness/rules/memory-governance-cursor.mdc`, `harness/visual-presentation.md`, `.moon/tasks/harness-cursor.yml`                                                   | macOS et Ubuntu / `agent-memory (os)`     |
| Proof Integrity tests  | `proof-integrity:{fmt,clippy,check,test,doc}`               | Skill `proof-integrity-review/SKILL.md`, ses `references/**/*` et `scripts/policy-sources.json`                                                                                                                                                                         | macOS et Ubuntu / `proof-integrity (os)`  |

Chaque suite retenue garde son checkout, son setup toolchain et ses commandes
`moon ci --downstream none`. Les cinq contrôles Rust restent disponibles, avec
les flags et mutex existants ; Handoff et Memory conservent leurs tests de
déploiement. La politique de cache reste inchangée. `select` ne prépare ni Cargo
ni Bun par exécution de tâche ; le setup installe Moon/proto et conserve son
cache toolchain existant.

Les statuts spécialisés peuvent être skipped. La protection de fusion et les
statuts continus appartiennent à #418 ; cette tranche ne les établit pas.

## Observations

Les preuves distinguent une interrogation native du graphe d'une exécution
GitHub Actions. Un résultat skipped n'est pas une preuve de tests exécutés.
Les mesures Actions doivent nommer SHA, événement, versions, plateformes, URLs,
attente avant démarrage et durée d'exécution. Les secondes runner observées ne
constituent pas une mesure du coût facturé.

### Validation locale — 8 octobre 2026

Base `326f5389334097f5644d551ab4e17d20ff8390b0`, modifications de #420
incluses dans le worktree `rust-ci-select-420`, macOS ARM64 ; Moon 2.5.3,
Rust/Cargo 1.98.1, Bun 1.4.0 via Moon, Actionlint 1.7.12.

- `moon query tasks` valide les déclarations et le graphe résolu. En injectant
  les chemins par stdin dans `moon query tasks --affected --upstream none
--downstream none`, README sélectionne zéro famille ; les sources/lockfiles
  de chaque crate sélectionnent leur famille, et Proof Integrity sélectionne
  également Arnes, consommateur de `harness/**/*`.
- Les témoins Cargo racine, toolchain Rust, `.prototools`, workspace Moon et
  héritage Rust sélectionnent les cinq familles ; la configuration Cargo propre
  à Arnes ne sélectionne qu'Arnes. Harness, aliases Fish, helpers de déploiement,
  règles Cursor et références Proof Integrity sélectionnent leurs consommateurs.
  `package.json` et `bun.lock` sélectionnent Arnes, Handoff et Memory.
- Une interrogation distante avec `MOON_BASE=missing-420-revision` et
  `MOON_HEAD=HEAD` retourne un échec Git 128, sans résultat de sélection valide.
- `moon exec --no-actions --upstream none` a exécuté fmt, Clippy, check, test et
  doc pour les cinq projets, les deux `deployment-test`, `repository:prettier-check`
  et `repository:workflows-lint` : 29 tâches terminées, dont une en cache.
  Les déploiements utilisent les destinations isolées de leurs tests existants.

Cette observation locale n'exerce pas Ubuntu, le contexte Actions ni le smoke
minimal public ; ceux-ci nécessitent leurs runners CI.
