# Sélection CI — #407

Cette matrice consolide les contributions de
[#491](https://github.com/SebastienElet/dotfiles/pull/491),
[#501](https://github.com/SebastienElet/dotfiles/pull/501),
[#466](https://github.com/SebastienElet/dotfiles/pull/466),
[#499](https://github.com/SebastienElet/dotfiles/pull/499),
[#500](https://github.com/SebastienElet/dotfiles/pull/500) et
[#505](https://github.com/SebastienElet/dotfiles/pull/505) pour
[#407](https://github.com/SebastienElet/dotfiles/issues/407).
Elle couvre les 16 workflows actuels, dont Herdr et Secret scanning ajoutés au
périmètre historique des 14 familles. Les entrées Moon restent canoniques ; les
tables en résument les frontières sans définir un second sélecteur. Le
[relevé #272](moon-gates-272.md) est historique.

## Contrat commun

Les ADR acceptées [001](adr/001-makefile-installateur.md),
[023](adr/023-ci-lint-et-installation.md) et
[041](adr/041-frontiere-automatisation-typescript-rust.md) restent en vigueur.
`tooling:smoke-minimal` conserve son job macOS à chaque PR et push `main`,
indépendamment de la sélection spécialisée.

Les workflows sélectionnés commencent par `select` sur Ubuntu : checkout complet et
blobless, puis le [setup Moon local](../.github/actions/setup-moon/action.yml), la validation native des cibles avec
`moon action-graph --json` (sans exécution), et `moon query tasks --affected remote --upstream none --downstream none`, filtré
sur son projet et ses tâches. Les expressions natives Actions lisent la sortie
JSON de Moon. Un projet absent du résultat valide évite l'allocation de sa suite ;
une commande en échec ou une sortie JSON invalide fait échouer la sélection.
Une cible ou un projet absent échoue lors de la construction du graphe avant
l'interrogation affectée. Aucun filtre de chemins parallèle ne décide à la place de Moon.

`MOON_BASE` est le SHA de base de la PR, ou `before` pour un push `main` ;
`MOON_HEAD` est le SHA checkout (`github.sha`, merge synthétique pour une PR).
Sélection et exécution utilisent ces mêmes révisions. Le push couvre ainsi tous
ses commits, pas uniquement le dernier. Une révision introuvable reste un échec.

Les 15 agrégats `… required` s'exécutent avec `always()` sur Ubuntu. Une sélection
doit réussir et déclarer explicitement chaque suite affectée réussie ou chaque
suite non affectée skipped ; une sortie absente, un échec ou une annulation ne
donne pas un agrégat vert. Herdr et le smoke exigent leur job amont réussi.
`Secret scan` reste un contrôle continu distinct. Le blocage effectif de fusion
nécessite les Settings manuels décrits dans #491 ; ces statuts seuls ne l'activent pas.

## Matrice hors des cinq familles Rust

Chaque suite sélectionnée inclut son workflow et ses dépendances Moon dans ses
entrées de contrôle. Le setup actuel installe Moon 2.5.3 par `npm ci` avec son
lockfile, depuis [#515](https://github.com/SebastienElet/dotfiles/pull/515) ; il
remplace le cache toolchain des observations historiques. Les runtimes et paquets
des suites restent préparés par leurs tâches Moon.

| Famille / statut continu           | Tâches / plateformes spécialisées                                                                                                | Entrées directes ; transversales                                                                                                                                                                                                                                  | Préparation spécialisée                                                                       |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Text / `Text required`             | `repository:{cspell-check,prettier-check}` / Ubuntu                                                                              | Texte/YAML/JSON et Markdown des skills indexés ; configuration/dictionnaire CSpell, helpers des gates, exclusions Prettier, package/lockfile et configuration Moon                                                                                                | Bun/CSpell dans la suite retenue ; README affecte Prettier                                    |
| Static / `Static required`         | `repository:typescript-{lint,typecheck,format-check}` / Ubuntu                                                                   | TypeScript suivi, sources typées et `oxfmt.config.ts` ; configuration lint/TypeScript, package/lockfile et Moon                                                                                                                                                   | Bun dans la suite retenue                                                                     |
| TypeScript / `TypeScript required` | `repository:{node-runtime-check,typescript-test}` / Ubuntu                                                                       | Entrées des tests généraux, données plugins/skills consommées, sources Rust Memory/Herdr ; installateurs et toolchain/configuration partagés                                                                                                                      | Volta/Node, Bun, Fish et builds Rust dans la suite retenue ; Docker smoke conserve son opt-in |
| Harness / `Harness required`       | `harness:{validate-evals,validate-evidence}` / Ubuntu                                                                            | Skills/evals/evidence et instructions Harness, contrats locaux d'activation indexés ; sources/lockfile Cargo Arnes, package/lockfile et configuration Moon/Rust                                                                                                   | Rust dans la suite retenue                                                                    |
| Lua / `Lua required`               | `repository:{neovim-lint,wezterm-lint}` / macOS et Ubuntu, sélection séparée                                                     | `home/.config/nvim/**/*.lua`, `home/.config/wezterm/wezterm.lua` ; `.luacheckrc` optionnel, `.prototools`, Moon Lua/workspace/toolchains/paquets/poste et bootstrap Homebrew                                                                                      | Luacheck par matrice retenue ; lockfile Neovim exclu                                          |
| Fish / `Fish required`             | `repository:{fish-syntax,fish-format,fish-test}` / macOS et Ubuntu                                                               | `home/.config/fish/**/*.fish` ; package/lockfile, Moon et bootstrap Homebrew ; `tooling/check-scripts.ts` pour syntaxe/format, `.gitignore` et `tooling/git-main-branch*` pour les tests                                                                          | Fish et Bun dans la matrice retenue                                                           |
| Shell / `Shell required`           | `repository:{shell-ci,workflows-lint}`, `tooling:{installer-test,upgrade-test}` / macOS                                          | Helpers sans extension de `tooling/` et `harness/plugins/`, `*.sh`, workflows, installateur/bootstrap et upgrade ; contrat Node, résolution de branche Git, package/lockfile et Moon                                                                              | ShellCheck, Actionlint et Bun dans la suite retenue                                           |
| Deployment / `Deployment required` | `tooling:{deployment-test,hunspell-test}` / macOS et Ubuntu ; `tooling:worker-native-test` / macOS, sélection séparée            | Sources déployées `home/`, `harness/`, `.agents/` dont Markdown, helpers/fixtures de liens/skills/instructions/cleanup ; dictionnaires et `home/moon.yml` pour Hunspell ; worker/inspection/marker/chemins/support pour le worker ; configuration Moon/toolchains | Bun dans les suites retenues ; tests de déploiement/Hunspell/worker sans cache de résultat    |
| Smoke / `Smoke required`           | `tooling:{moon-bootstrap-smoke,smoke-minimal}` / macOS, inconditionnels ; `tooling:upgrade-neovim-test` conditionnel après smoke | Installation publique, nettoyage et rejeu à chaque PR et push `main` ; upgrade/fixtures, dépendances Node/Git et toute configuration Neovim, lockfile inclus, pour l'intégration                                                                                  | Installation minimale complète ; smoke et intégration sans cache de résultat                  |
| Herdr / `Herdr required`           | `herdr-issue:{fmt,clippy,check,test,doc}` / macOS et Ubuntu                                                                      | Sources/tests/configuration Cargo du projet et entrées Rust communes ci-dessous ; workflow Herdr                                                                                                                                                                  | Matrice allouée sans `select`, tâches affectées via `moon ci` ; préparation Rust              |
| Secret scanning / `Secret scan`    | `repository:{secret-test,secret-scan}` / Ubuntu                                                                                  | Historique Git complet, configuration `.gitleaks.toml` et tests Gitleaks ; exécution inconditionnelle                                                                                                                                                             | Gitleaks et Bun ; aucun cache de résultat                                                     |

Les preuves et mesures détaillées restent dans les PR contributrices : #501
(Text/Static/TypeScript/Harness), #499 (Lua/Fish), #500 (Shell/Deployment/Smoke),
#491 (statuts et Herdr/Secrets) et #505 (préparation/cache).

## Matrice Rust

Les entrées communes sont les manifests/lockfiles Cargo, `build*.rs`,
`src/**/*`, `tests/**/*` (fixtures incluses), examples et benches, `moon.yml` et
`.cargo/**/*`, les noms `clippy.toml`/`.clippy.toml`,
`rustfmt.toml`/`.rustfmt.toml` et `rust-toolchain`/`rust-toolchain.toml` du projet ; à la racine : `.cargo/**/*`, `clippy.toml`, `.clippy.toml`,
`rustfmt.toml`/`.rustfmt.toml`, `rust-toolchain`/`rust-toolchain.toml`, `.prototools`, `moon.yml`, `LICENSE` et
`tooling/RUST.md`. Moon prend aussi en compte sa configuration workspace,
toolchains et héritage Rust. `.moon/tasks/workstation.yml` est une entrée explicite
car il déclare le prérequis `repository:rust`. Chaque famille déclare son propre workflow.

| Workflow               | Projet / tâches sélectionnées                               | Entrées transversales supplémentaires                                                                                                                                                                                                                                                    | Runners spécialisés / statuts             |
| ---------------------- | ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
| Arnes tests            | `arnes:{fmt,clippy,check,test,doc}`                         | `home/.arnes.yaml`, `harness/**/*`, `home/moon.yml`, héritages Codex/Claude/JavaScript, helpers `assemble-agent-instructions.ts`, `install-agent-skills.ts`, `deploy-link.ts`, `package.json`, `bun.lock`, `tsconfig.json` pour les tests du dépôt                                       | Ubuntu / `arnes`                          |
| Bitbucket Linear tests | `bitbucket-linear-sync:{fmt,clippy,check,test,doc}`         | `home/.config/fish/conf.d/aliases.fish`                                                                                                                                                                                                                                                  | macOS et Ubuntu / `bitbucket-linear (os)` |
| Agent Handoff tests    | `agent-handoff:{fmt,clippy,check,test,doc,deployment-test}` | Déploiement : helpers `deployment-{agent-handoff.test,moon-runner,moon-test-support,test-support,test-command}.ts`, `deploy-link.ts`, `clean-deployment*.ts`, `clean-deployment-worker-inspect.js`, `home/.arnes.yaml`, `package.json`, `bun.lock`, `tsconfig.json`, héritage JavaScript | macOS et Ubuntu / `agent-handoff (os)`    |

Chaque suite retenue garde son checkout, son setup toolchain et ses commandes
`moon ci --downstream none`. Les matrices désactivent `fail-fast` pour conserver un résultat propre à chaque
plateforme lorsqu'une autre échoue. Les cinq contrôles Rust restent disponibles, avec
les flags et mutex existants ; Handoff et Memory conservent leurs tests de
déploiement. `select` ne prépare ni Cargo ni Bun par exécution de tâche.
Depuis #505, seule Arnes restaure `tooling/arnes/target` : clé exacte OS/architecture,
toolchain Rust, manifests/lockfile/configuration Cargo, YAML Moon, `.prototools`
et workflow, sans préfixe de repli ; `CARGO_INCREMENTAL=0`. Cargo vérifie encore
la fraîcheur et exécute les contrôles/tests après restauration ; un cache absent
reprend le graphe normal. Aucun résultat de tâche Moon ni installation utilisateur
n'est stocké. Les autres suites et fixtures gardent leur politique existante.

Les statuts spécialisés peuvent être skipped. Leurs agrégats continus sont
`Arnes required`, `Bitbucket Linear required`, `Agent Handoff required`,
`Agent Memory required` et `Proof Integrity required` (#491).

## Observations

Les preuves distinguent une interrogation native du graphe d'une exécution
GitHub Actions. Un résultat skipped n'est pas une preuve de tests exécutés.
Les mesures Actions doivent nommer SHA, événement, versions, plateformes, URLs,
attente avant démarrage et durée d'exécution. Les secondes runner observées ne
constituent pas une mesure du coût facturé.

### Validation locale historique — 8 octobre 2026

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
  `package.json`, `bun.lock` et `tsconfig.json` sélectionnent Arnes, Handoff et Memory.
- Une interrogation distante avec `MOON_BASE=missing-420-revision` et
  `MOON_HEAD=HEAD` retourne un échec Git 128, sans résultat de sélection valide.
- `moon exec --no-actions --upstream none` a exécuté fmt, Clippy, check, test et
  doc pour les cinq projets, les deux `deployment-test`, `repository:prettier-check`
  et `repository:workflows-lint` : 29 tâches terminées, dont une en cache.
  Les déploiements utilisent les destinations isolées de leurs tests existants.

Cette observation locale n'exerce pas Ubuntu, le contexte Actions ni le smoke
minimal public ; ceux-ci nécessitent leurs runners CI.
