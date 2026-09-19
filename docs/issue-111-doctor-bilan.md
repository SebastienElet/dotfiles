# #111 — Bilan de Doctor et décisions restantes

Relevé du 16 septembre 2026 sur `272e8a69781734780e5531701d9fa41c2815ba7c`.
Ce bilan rapproche le contrat ouvert de [#111](https://github.com/SebastienElet/dotfiles/issues/111)
des livraisons existantes. Il ne remplace pas la [référence commune](arnes-doctor.md)
livrée par #126 et ne clôture aucune issue.

Mise à jour du 19 septembre : le mainteneur a approuvé les recommandations D1/D2.
Les observations techniques ci-dessous restent celles du 16 septembre ; aucune
nouvelle exécution Doctor n'est revendiquée par cette mise à jour.

**Conclusion :** les familles Doctor et les preuves sur fixtures sont livrées.
La lecture seule stricte reste contredite par l'exécution du résolveur Codex.
La décision D1 conserve cette exigence, encore à mettre en œuvre et à prouver.
D2 conserve les défauts MCP et limite l'équivalence à une portée explicite identique ;
cette précision reste à reporter dans #111/#123.
Les [brouillons de synchronisation](issue-111-sync-brouillons.md) restent des propositions
à valider puis à publier ; aucune synchronisation n'est implémentée ici.

## Autorités et livraisons réutilisées

Les ADR [001](adr/001-makefile-installateur.md),
[003](adr/003-deploiement-par-symlinks.md),
[023](adr/023-ci-lint-et-installation.md),
[038](adr/038-frontieres-home-harness-tooling.md) et
[041](adr/041-frontiere-automatisation-typescript-rust.md), acceptées et révisées
le 7 septembre, imposent Moon, les frontières du dépôt et les protections de déploiement.
Les ADR [028](adr/028-skills-ssot.md) et [040](adr/040-skills-user-dans-harness.md)
séparent les sources de skills project/user ; les anciennes mentions Make de 040
se lisent avec la transition explicitement décidée par 001/038/041.
L'ADR [043](adr/043-telemetrie-arnes-minimale-et-bornee.md) distingue configuration,
activité observée et résultat. Aucune de ces ADR ne décide une exception de lecture seule
pour le résolveur Codex. Les issues constituent des contrats de travail, pas des ADR.

| Livraison                                                                                                                              | Preuve réutilisée                                                                                                        | Ce qu'elle ne permet pas de conclure                                        |
| -------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------- |
| [#123](https://github.com/SebastienElet/dotfiles/issues/123), [PR #288](https://github.com/SebastienElet/dotfiles/pull/288), `d31681e` | Dix familles, ordre, réemploi des diagnostiqueurs ; tests renforcés ensuite par #339                                     | La formule « strictly read-only » n'est pas satisfaite universellement      |
| [#126](https://github.com/SebastienElet/dotfiles/issues/126), [PR #338](https://github.com/SebastienElet/dotfiles/pull/338), `f8a531d` | Référence unique, aide et sept exemples exécutés sur macOS ; limites Codex et défaut MCP déjà documentés                 | La CI Arnes documentaire à zéro cible n'est pas une exécution Rust          |
| [#124](https://github.com/SebastienElet/dotfiles/issues/124), [PR #339](https://github.com/SebastienElet/dotfiles/pull/339), `272e8a6` | Graphe/recettes Moon inspectés, fixture passant par 0/1/2 en human/JSON, snapshots après chaque appel, PATH Doctor isolé | Fermeture de l'issue ≠ satisfaction de sa promesse stricte de lecture seule |

Le journal [Ubuntu de #339](https://github.com/SebastienElet/dotfiles/actions/runs/35072676955/job/104717640460)
a été relu : Ubuntu 24.04, **5 cibles effectivement exécutées**, `fmt`, `clippy`,
`check`, `test`, `doc`, **742 tests réussis, 0 échec, 0 ignoré** sur `6f4867a`.
Le relevé macOS de #339 nomme macOS 26.6.2 / Darwin arm64, Cargo 1.98.0,
Moon 2.5.4 et la même suite, sous interdiction réseau. Il s'agit d'une preuve
locale rapportée dans cette PR, pas d'une nouvelle exécution de cette suite ici.
Les modifications de #339 sont présentes dans le checkout audité.

## Rattachements et couverture par famille

Lecture fraîche de `GET /repos/SebastienElet/dotfiles/issues/111/sub_issues` :
les **15** enfants attendus sont présents et fermés : #109, #110, #112–#115,
#117–#124 et #126. Aucun rattachement manquant n'a été trouvé.
La requête GraphQL des PR de clôture a été complétée par les timelines et l'historique Git.
Une fermeture manuelle n'est pas assimilée à une PR automatiquement liée.

| Famille / fondation | Sous-issue et livraison retrouvée                                                                                                                               | Représentations exercées, sans session réelle                                                                                                                |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| CLI                 | [#112](https://github.com/SebastienElet/dotfiles/issues/112), [#127](https://github.com/SebastienElet/dotfiles/pull/127), `2139384` dans la timeline de clôture | Arguments, HOME, erreurs de sortie : `tests/cli.rs`                                                                                                          |
| Manifest            | [#113](https://github.com/SebastienElet/dotfiles/issues/113), [#137](https://github.com/SebastienElet/dotfiles/pull/137)                                        | YAML v1 et invalidités ; validation globale, même avec filtres                                                                                               |
| Diagnostics         | [#114](https://github.com/SebastienElet/dotfiles/issues/114), [#139](https://github.com/SebastienElet/dotfiles/pull/139)                                        | Ordre, formats, sévérités et fixture isolée                                                                                                                  |
| Config              | [#115](https://github.com/SebastienElet/dotfiles/issues/115), [#140](https://github.com/SebastienElet/dotfiles/pull/140)                                        | Trois agents user/project ; JSON Claude/Cursor, TOML Codex ; valeurs user non imposées au projet                                                             |
| Instructions        | [#117](https://github.com/SebastienElet/dotfiles/issues/117), [#141](https://github.com/SebastienElet/dotfiles/pull/141)                                        | Liens Claude user, assemblage Codex user ; include Claude project, positif complété ci-dessous ; Cursor et Codex project unsupported                         |
| Skills              | [#118](https://github.com/SebastienElet/dotfiles/issues/118), [#142](https://github.com/SebastienElet/dotfiles/pull/142)                                        | Trois agents : feuilles user, racines project ; registres/fichiers et doubles des résolveurs externes                                                        |
| Prompts             | [#109](https://github.com/SebastienElet/dotfiles/issues/109), [#155](https://github.com/SebastienElet/dotfiles/pull/155)                                        | Claude user rendered, Claude/Cursor project file ; symlink unsupported ; aucun prompt déclaré dans le manifeste livré                                        |
| Commands            | [#110](https://github.com/SebastienElet/dotfiles/issues/110), [#162](https://github.com/SebastienElet/dotfiles/pull/162)                                        | Claude user/project, liaisons et descriptions ; autres agents unsupported ; aucune commande déclarée dans le manifeste livré                                 |
| Rules               | [#119](https://github.com/SebastienElet/dotfiles/issues/119), commit [274bbda](https://github.com/SebastienElet/dotfiles/commit/274bbda)                        | Liens Claude/Cursor user ; Codex/project unsupported ; clôture sans PR automatique retournée                                                                 |
| Hooks               | [#120](https://github.com/SebastienElet/dotfiles/issues/120), [#270](https://github.com/SebastienElet/dotfiles/pull/270), `e26ee44`                             | Claude user, Cursor measurement direct, Codex/Claude output-discipline ; setup memory/handoff distinct du diagnostic ; clôture sans PR automatique retournée |
| MCP                 | [#121](https://github.com/SebastienElet/dotfiles/issues/121), [#282](https://github.com/SebastienElet/dotfiles/pull/282)                                        | Trois agents project, Claude user en CLI ; parseur Codex user ; positifs CLI Cursor/Codex user complétés ci-dessous                                          |
| Statusline          | [#122](https://github.com/SebastienElet/dotfiles/issues/122), [#285](https://github.com/SebastienElet/dotfiles/pull/285)                                        | Codex user/project, liste TOML ordonnée ; autres sélections silencieuses, sans conformité implicite                                                          |

Les sources des oracles sont les [suites Arnes](../tooling/arnes/tests/) :
`config`/`config_defaults`, `instructions`/`instruction_failures`, `skills`/`skill_failures`,
`prompts`/`prompt_failures`, `commands`/`command_failures`, `rules`/`rule_failures`,
`hooks_doctor`/`output_discipline`, `mcp`/`mcp_failures`, `statusline`/`statusline_boundaries`.
Elles exercent les filtres, états et frontières propres aux représentations ; elles
ne forment pas une certification de toute combinaison agent × portée × état.

## Bilan des neuf critères de #111

| Critère                                                                    | Disposition                                   | Preuve et reste exact                                                                                                                                                                                    |
| -------------------------------------------------------------------------- | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. Familles livrées dans des sous-issues rattachées                        | Établi                                        | Inventaire des 15 enfants et livraisons ci-dessus ; aucune réparation de rattachement nécessaire                                                                                                         |
| 2. Sélecteurs, filtres et états sur dépôt/HOME isolés                      | Établi dans les représentations nommées       | Suites de ressources et compléments macOS ci-dessous. `unsupported` et les omissions ne valent pas healthy ; aucune parité d'agents                                                                      |
| 3. Toutes les familles dans l'ordre et mêmes diagnostics individuels       | Équivalence bornée approuvée par D2           | Ordre et filtres explicites verts dans #339 ; D2 conserve les défauts MCP et l'équivalence à portée explicite identique. Précision à reporter dans #111/#123                                             |
| 4. Cohérence human/JSON et sorties                                         | Établi dans la portée CLI testée              | `aggregate_preserves_one_fixture_through_healthy_drift_and_fatal_states`, suites de rendu/CLI : 0 healthy/unsupported/vide, 1 drift, 2 error ; erreurs d'arguments/sortie possibles sur stderr sans JSON |
| 5. Absence de mutation et de contact externe, aucun lancement de capacités | Non établi ; garantie générale contredite     | Snapshots bornés déjà livrés ; nouvelle expérience du résolveur ci-dessous. Aucun oracle de confinement du vrai Codex ; pas de preuve d'absence d'effet externe universelle                              |
| 6. Livraison canonique et documentation                                    | Livré dans la portée demandée de l'inspection | #339 atteste graphe/recettes Moon natifs sans installation globale ; chemins binaire/manifeste identifiés. #126 est la référence commune. Pas de réinstallation du poste ici                             |
| 7. Rust couvert par format/analyse/tests                                   | Établi pour la livraison existante            | Cinq gates et 742 tests Ubuntu relus ; entrées src/tests héritées de `.moon/tasks/rust.yml`. Aucun Rust ajouté/modifié dans ce chantier                                                                  |
| 8. Plateformes et agents nommés sans couverture implicite                  | Établi pour les preuves nommées               | macOS local et Ubuntu 24.04 CI ; fixtures/doubles seulement. Pas de sandbox réseau Linux attesté, pas de session Claude/Cursor/Codex réelle                                                              |
| 9. Sous-issues de synchronisation par ressource                            | Brouillons prêts, non publié                  | Huit périmètres proposés ; hooks déjà réconciliés par setup, manifest livré par Moon. Publication/rattachement après validation, donc critère encore ouvert                                              |

Les tests agrégés réutilisés vérifient aussi que drift et erreur d'une famille
n'empêchent pas les suivantes. Un manifeste invalide arrête le diagnostic global.
Human masque normalement le détail healthy ; JSON le conserve. `--verbose` et
`--color always` sont incompatibles avec JSON et sortent 2 avant diagnostic.
MCP peut lire l'autre portée pour détecter une collision : filtrer le résultat
n'est pas confiner toutes les lectures à une seule portée.

## Compléments exécutés, sans refaire les suites

Environnement : macOS 26.6.2 / Darwin arm64, Cargo 1.98.0, Bun 1.4.0.
Le binaire de ce checkout a été compilé avec `cargo build --locked --offline
--manifest-path tooling/arnes/Cargo.toml`. Les expériences ont été exécutées sous
`sandbox-exec -p '(version 1)(allow default)(deny network*)'`, avec environnement
explicite, HOME et répertoire projet temporaires, PATH privé sans agent réel.
Ces expériences publiques du binaire ne sont pas de nouvelles gates permanentes.

| Complément ciblé            | Préparation                                                                               | Résultat                                                                            |
| --------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Instructions Claude project | Manifeste avec une projection `AGENTS.md → CLAUDE.md`, destination contenant `@AGENTS.md` | `doctor instructions --agent claude --scope project`, human et JSON : 0, un healthy |
| MCP Cursor user             | Déclaration user `managed`, `.cursor/mcp.json` avec la même commande `/usr/bin/true`      | `doctor mcp --agent cursor --scope user`, human et JSON : 0, un healthy             |
| MCP Codex user              | Même déclaration dans `.codex/config.toml`, table `mcp_servers.managed`                   | `doctor mcp --agent codex --scope user`, human et JSON : 0, un healthy              |

Ces six appels complètent des cas positifs CLI non identifiés dans les suites
relues ; ils n'attestent ni Linux pour ces nouveaux cas ni le lancement des agents.
Aucun snapshot supplémentaire n'est revendiqué pour ces six appels.

### Contre-exemple de lecture seule

Un manifeste ne déclare que Codex user/project, `resources: []`, sans plugins ;
`.codex/config.toml` est vide. Le seul `codex` du PATH est ce double :

```sh
#!/bin/sh
printf '%s|%s|%s|%s\n' "$*" "$PWD" "$HOME" "$CODEX_HOME" >> "$PROBE_WITNESS"
printf temporary > "$HOME/transient-probe"
/bin/rm "$HOME/transient-probe"
printf '{"marketplaces":[],"installed":[]}\n'
```

`PROBE_WITNESS` désigne un fichier temporaire frère de HOME et du répertoire
projet, donc hors des deux snapshots. Le double ne fait aucune tentative réseau.
Les deux appels `doctor skills --agent codex --scope user --format json` et
`doctor --agent codex --scope user --format json` sortent **0**, stderr vide.
Chaque appel exécute **deux** commandes : `plugin marketplace list --json`,
puis `plugin list --json`. Le témoin confirme cwd=HOME et CODEX_HOME=HOME/.codex.

Après chaque appel, les snapshots des fichiers, modes et liens de HOME/projet
restent **identiques**, alors que le témoin externe s'allonge. Le fichier temporaire
créé puis supprimé dans HOME leur échappe aussi. Ce résultat réfute l'implication
« snapshots identiques ⇒ Doctor sans effets » ; il ne décrit pas les effets d'un vrai Codex.
Le réseau était interdit par l'environnement d'expérience, pas par Arnes.

Le chemin possédé est [skills/external/codex/state.rs](../tooling/arnes/src/skills/external/codex/state.rs)
puis [command.rs](../tooling/arnes/src/skills/external/codex/command.rs) : subprocessus
avec HOME/cwd redirigés, délai de cinq secondes et 1 Mio par flux, mais aucun
confinement des écritures ou du réseau. [#166](https://github.com/SebastienElet/dotfiles/issues/166)
et [#168](https://github.com/SebastienElet/dotfiles/pull/168) ont livré la résolution
autoritative ; ils ne constituent pas une décision d'abandonner le contrat de #111.
Retirer cette résolution sans décision pourrait perdre une capacité demandée par #166.
La [décision D1](issue-111-sync-brouillons.md#d1--décider-la-frontière-de-lecture-seule-de-doctor-face-au-résolveur-codex),
approuvée le 19 septembre, conserve la lecture seule stricte, quitte à signaler
l'inventaire Codex indisponible. Elle ne constitue pas une preuve de conformité.
La référence #126 expose déjà correctement la limite : aucun correctif documentaire redondant.

### Contre-exemple d'équivalence sans scope MCP

Fixture Claude user/project, configuration user saine, une seule déclaration MCP
**project** dont la configuration est absente. À filtre agent identique :

| Appel JSON                                  | Diagnostic MCP                                                            | Code global |
| ------------------------------------------- | ------------------------------------------------------------------------- | ----------- |
| `doctor mcp --agent claude`                 | unsupported user, aucun enregistrement déclaré                            | 0           |
| `doctor --agent claude`                     | drift project, configuration absente                                      | 1           |
| `doctor mcp --agent claude --scope project` | drift project, configuration absente                                      | 1           |
| `doctor --agent claude --scope project`     | même diagnostic MCP ; les autres familles gardent leurs propres résultats | 1           |

La cause est explicite dans [doctor.rs](../tooling/arnes/src/doctor.rs) : MCP direct
reçoit `user` par défaut, MCP agrégé reçoit la portée absente. Le test existant
`default_doctor_checks_project_mcp_without_changing_other_scope_defaults` et #126
décrivent ce choix. Ce n'est donc pas un défaut borné à corriger silencieusement.
La décision [D2](issue-111-sync-brouillons.md#d2--préciser-léquivalence-mcp-entre-doctor-direct-et-agrégé),
approuvée le 19 septembre, conserve ces défauts et borne l'équivalence à une portée
explicitement identique. La formulation large de #111/#123 reste à mettre à jour.

## Suite et validation du présent dossier

Aucun défaut de production borné n'a été démontré nécessitant une correction dans
ce chantier. Aucun test existant, ADR, contrat CLI ou document de référence n'est modifié.
Les preuves originales de #123/#124/#126 restent réutilisées avec leur portée exacte.

Les brouillons distinguent observation stable et autorité future d'écriture.
Leur validation puis publication ne suffiront pas, seules, à clôturer #111 : il faut
encore appliquer D1 et établir les preuves de lecture seule stricte, puis reporter
la disposition D2 dans les trackers. Les arbitrages eux-mêmes sont désormais tranchés.
Les sessions réelles restent non attestées ; ne les revendiquer que si un oracle
nomme l'agent, sa version, sa représentation, la plateforme et l'observation de session.

Auteur `/root`, auditeur indépendant `/root/audit_doctor_claims` ;
le [registre d'audit](issue-111-doctor-audit.md) conserve les claims et leurs limites.
Le diff est documentaire : `repository:prettier-check` a réussi sur macOS avec
Prettier 3.9.6, après indexation des trois documents ; les 49 cibles de liens locaux
et `git diff --cached --check` ont été vérifiés. Le contrôle a réutilisé les dépendances
du checkout principal, dont `package.json` et `bun.lock` sont identiques, après qu'une
première tentative Moon sans dépendances locales a échoué. Les 742 tests Rust inchangés
n'ont pas été relancés. Commentaires de code ajoutés : aucun.
