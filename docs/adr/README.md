# Architecture Decision Records

Décisions structurantes de ce dépôt, reconstituées depuis l'historique git
(934 commits, 2014-08 → 2026-08). Format inspiré de
[MADR](https://adr.github.io/madr/).

## Portée

Seules les décisions **en vigueur** sont enregistrées. Les décisions
remplacées ne font pas l'objet d'une ADR ; elles apparaissent dans la section
« Alternatives écartées » de celle qui les a remplacées (NvChad, prezto puis
antigen, gestion des plugins Vim par cibles Makefile, nvm puis fnm,
docker-machine, iTerm2 puis Alacritty, chunkwm puis yabai, Copilot, pnpm pour
les paquets globaux).

Exception : un retrait dont la justification est mesurée et réutilisable fait
l'objet d'une ADR propre, la décision en vigueur étant alors « ne pas
utiliser » (ADR-033, ADR-034).

## Langue

Les ADR sont rédigées en français, par exception à la règle « documentation en
anglais » qui régit le reste du dépôt : elles consignent un raisonnement
personnel, non une interface publique. L'exception vaut pour tout `docs/` et
est portée par `docs/AGENTS.md`. Le rappel qui la doublait dans l'`AGENTS.md`
racine a été retiré : mesuré, il ne changeait rien
([ADR-035](035-agents-md-elague-par-mesure.md)).

## Fiabilité des motivations

Le champ **Contexte** est une reconstitution a posteriori, sauf lorsqu'il cite
le corps d'un commit — dans ce cas la citation est explicite. L'historique a
été réécrit : les dates de commit valent toutes 2026-08-06, les dates citées
ici sont les dates d'auteur.

## Index

| #                                                      | Titre                                                            | Date    |
| ------------------------------------------------------ | ---------------------------------------------------------------- | ------- |
| [001](001-makefile-installateur.md)                    | Profils hybrides comme installateur de poste                     | 2026-08 |
| [002](002-homebrew-source-unique.md)                   | Brewfiles comme source unique des paquets                        | 2026-08 |
| [003](003-deploiement-par-symlinks.md)                 | Déploiement depuis un état propre                                | 2026-08 |
| [004](004-script-upgrade-unique.md)                    | Script `upgrade` unique pour toutes les mises à jour             | 2017-09 |
| [005](005-elagage-des-cibles.md)                       | Élagage périodique des cibles inutilisées                        | 2025-12 |
| [006](006-neovim-editeur-par-defaut.md)                | Neovim comme éditeur par défaut                                  | 2019-11 |
| [007](007-lazyvim-comme-distribution.md)               | LazyVim comme distribution Neovim                                | 2023-12 |
| [008](008-lockfile-plugins-versionne.md)               | Lockfiles versionnés et Dependabot pour Bun                      | 2024-10 |
| [009](009-lsp-natif-et-conform.md)                     | Lint et format par LSP natif et conform                          | 2026-07 |
| [010](010-fish-shell-par-defaut.md)                    | Fish comme shell par défaut                                      | 2025-01 |
| [011](011-starship-comme-prompt.md)                    | Starship comme prompt unique                                     | 2025-01 |
| [012](012-abbreviations-fish.md)                       | Abbreviations Fish plutôt qu'alias git                           | 2026-01 |
| [013](013-wezterm-emulateur.md)                        | WezTerm comme émulateur de terminal                              | 2026-01 |
| [014](014-theme-catppuccin-unique.md)                  | Thème Catppuccin unique et bascule automatique                   | 2026-01 |
| [015](015-cli-modernes.md)                             | CLI modernes en remplacement des historiques                     | 2026-01 |
| [016](016-volta-gestionnaire-node.md)                  | Volta comme gestionnaire Node                                    | 2021-05 |
| [017](017-npm-pour-paquets-globaux.md)                 | npm pour les paquets globaux                                     | 2026-01 |
| [018](018-orbstack-runtime-conteneurs.md)              | OrbStack comme runtime de conteneurs                             | 2024-02 |
| [019](019-pas-de-tiling-window-manager.md)             | Aucun gestionnaire de fenêtres en tuiles                         | 2025-12 |
| [021](021-branche-main-detectee.md)                    | `main` détectée dynamiquement                                    | 2023-12 |
| [022](022-conventional-commits.md)                     | Conventional Commits imposés                                     | 2025-12 |
| [023](023-ci-lint-et-installation.md)                  | CI de lint et smoke minimal macOS                                | 2026-08 |
| [024](024-instructions-ia-versionnees.md)              | Instructions IA versionnées dans le dépôt                        | 2026-01 |
| [025](025-agents-md-source-unique.md)                  | `AGENTS.md` comme source unique agent-agnostique                 | 2026-02 |
| [026](026-agents-cli-plutot-que-copilot.md)            | Agents CLI plutôt que Copilot                                    | 2025-10 |
| [027](027-soul-et-user-separes.md)                     | `SOUL.md` et `USER.md` séparés de `AGENTS.md`                    | 2026-08 |
| [028](028-skills-ssot.md)                              | `.agents/skills/` comme source unique des skills projet          | 2026-05 |
| [029](029-pas-de-skills-tierces.md)                    | Skills tierces refusées par défaut                               | 2026-05 |
| [031](031-mcp-en-conteneurs-nommes.md)                 | MCP en conteneurs Docker nommés                                  | 2026-08 |
| [033](033-pas-de-rtk.md)                               | Ne pas utiliser RTK                                              | 2026-08 |
| [034](034-pas-de-caveman.md)                           | Ne pas utiliser les skills Caveman                               | 2026-08 |
| [035](035-agents-md-elague-par-mesure.md)              | `AGENTS.md` élagué par mesure du no-op                           | 2026-08 |
| [036](036-regles-ia-admises-par-ablation.md)           | Règles d'instructions IA admises par ablation marginale          | 2026-08 |
| [037](037-commentaire-absent-par-defaut.md)            | Commentaire absent par défaut, comptabilisé dans le compte rendu | 2026-08 |
| [038](038-frontieres-home-harness-tooling.md)          | Frontières `home/`, `harness/` et `tooling/`                     | 2026-08 |
| [039](039-code-search.md)                              | Recherche exacte et conceptuelle dans les checkouts Git          | 2026-08 |
| [040](040-skills-user-dans-harness.md)                 | Skills user dans `harness/`                                      | 2026-08 |
| [041](041-frontiere-automatisation-typescript-rust.md) | Frontière d'automatisation entre Shell, Moon, TypeScript et Rust | 2026-08 |
