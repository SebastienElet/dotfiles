# Finalisation des gates Moon — #272

Relevé du 16 septembre 2026. Base auditée :
`f8a531d2df112e05d429f07f2b608a79939e1e6f` (`main`).
Les ADR acceptées [001](adr/001-makefile-installateur.md),
[023](adr/023-ci-lint-et-installation.md) et
[041](adr/041-frontiere-automatisation-typescript-rust.md), révisées le
7 septembre, restent l'autorité : Moon sélectionne les gates ; le smoke
macOS exerce systématiquement l'installation publique, indépendamment du diff.

## Livraisons vérifiées

L'API `GET /repos/SebastienElet/dotfiles/issues/272/sub_issues` retourne les
sept issues fermées ci-dessous. Les commits de fusion sont présents dans la
base auditée. Les liens des PR contiennent les expériences comportementales,
les versions et les liens CI de leurs révisions finales ; leurs résultats
historiques ne sont pas présentés comme une nouvelle exécution sur `main`.

| Tranche                                                                   | Livraison fusionnée                                                                                                          | Commit    | Preuve conservée                                                                                                                         |
| ------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | --------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| [#273](https://github.com/SebastienElet/dotfiles/issues/273), Arnes       | [#280](https://github.com/SebastienElet/dotfiles/pull/280)                                                                   | `512fb5d` | fmt, Clippy, check et tests Ubuntu ; refus d'unsafe, sélection négative et mutex éprouvés sur macOS                                      |
| [#274](https://github.com/SebastienElet/dotfiles/issues/274), TypeScript  | [#283](https://github.com/SebastienElet/dotfiles/pull/283)                                                                   | `61de2f3` | trois gates statiques, une installation des dépendances dans le job Ubuntu ; tests de déclarations retirés                               |
| [#275](https://github.com/SebastienElet/dotfiles/issues/275), Fish        | [#291](https://github.com/SebastienElet/dotfiles/pull/291)                                                                   | `020401b` | syntaxe, format et comportement sur macOS/Ubuntu ; résolution de branche principale comme dépendance transversale                        |
| [#276](https://github.com/SebastienElet/dotfiles/issues/276), Lua         | [#334](https://github.com/SebastienElet/dotfiles/pull/334)                                                                   | `99c4302` | Neovim/WezTerm séparés sur macOS/Ubuntu ; diagnostics et codes de sortie comparés localement                                             |
| [#277](https://github.com/SebastienElet/dotfiles/issues/277), déploiement | [#335](https://github.com/SebastienElet/dotfiles/pull/335)                                                                   | `5be53d6` | liens, Hunspell, PR-feedback, memory et handoff ; matrices conservées, tests déplacés sans réécriture                                    |
| [#278](https://github.com/SebastienElet/dotfiles/issues/278), Code Search | [#336](https://github.com/SebastienElet/dotfiles/pull/336)                                                                   | `471bdb4` | 44 tests locaux sur chaque plateforme ; intégration réelle ColGrep 1.7.0 sur macOS, cache désactivé et substitution d'exécutable refusée |
| [#279](https://github.com/SebastienElet/dotfiles/issues/279), texte/Shell | [#326](https://github.com/SebastienElet/dotfiles/pull/326), après [#327](https://github.com/SebastienElet/dotfiles/pull/327) | `0878c21` | découverte indexée, canaris Shell, refus des chemins absents/exclus ; Prettier Ubuntu et Shell macOS                                     |

Apple Notes a été retiré par `494152a` : aucune famille obsolète n'est recréée.
Le lint Lua global et le pilote Arnes ont déjà été retirés. Les suites
déploiement et Code Search sont exclues de la suite TypeScript générale.
Les tests installateur/upgrade présents sur Ubuntu et macOS apportent deux
preuves de plateforme et restent conservés.

## Écarts établis et correction

- `test-harness.yml` exécutait `moon run harness:check` sur chaque diff.
  Cet agrégat entraîne fmt/Clippy/check/test Arnes et les gates statiques
  TypeScript/Prettier, déjà exercés par les autres workflows Ubuntu.
  Le workflow appelle désormais uniquement `harness:validate-evals` et
  `harness:validate-evidence` avec `moon ci --downstream none`.
  L'agrégat public `harness:check` reste disponible et inchangé.
- Fish, Shell et texte acceptaient tous les push et toutes les PR. Leurs
  push sont désormais limités à `main`, comme les autres workflows ; une
  branche sans PR n'a donc plus ces trois contrôles automatiques.
- Le wrapper Prettier lit `.gitignore` et `.prettierignore`, mais ces entrées
  ne sélectionnaient pas sa target. Elles sont maintenant déclarées, la
  seconde étant optionnelle.
- `tooling/format-typescript.ts` importe `oxfmt.config.ts` : cette entrée
  sélectionne maintenant aussi le typecheck et les tests généraux.
- La sélection des validations Harness inclut la configuration Cargo
  partagée utilisée pour compiler leur commande.

Aucun changement de règle, de commande métier, de plateforme ou de politique
de cache. Aucun validateur, inventaire de workflow exécutable ou commentaire
de code ajouté. Les caches ne dispensent pas de preuve d'intégration externe.

## Sélection native et comportement

Environnement local : macOS 26.6.2 arm64, Moon **2.5.4** installé localement,
Bun **1.4.0**. `.prototools` déclare 2.5.3 ; les livraisons récentes ont observé
2.5.5 en CI. Ces versions ne sont pas assimilées.

Les 29 entrées représentatives sont passées séparément à
`printf '%s\n' CHEMIN | moon query tasks --affected`, avant et après
correction. La commande native charge et résout les déclarations ; aucun
sélecteur de remplacement n'est utilisé. Exemples, limités aux familles
nommées dans la colonne de résultat :

| Entrée                                    | Résultat observé                                                                     |
| ----------------------------------------- | ------------------------------------------------------------------------------------ |
| source/test Arnes ou `.cargo/config.toml` | cinq gates Arnes ; configuration Cargo : les deux validations Harness également      |
| `tooling/check-prettier.ts`               | lint, typecheck et format TypeScript                                                 |
| `oxfmt.config.ts`                         | avant : lint/format ; après : lint/format/typecheck et tests TypeScript              |
| `README.md`                               | Prettier seul ; aucune gate Arnes, TypeScript, Fish, Lua, déploiement ou Code Search |
| configuration Fish                        | syntaxe, format et tests Fish                                                        |
| `tooling/git-main-branch-core.ts`         | tests Fish, sans syntaxe/format Fish                                                 |
| `home/.config/nvim/init.lua`              | Neovim, sans WezTerm                                                                 |
| `home/.config/wezterm/wezterm.lua`        | WezTerm, sans Neovim                                                                 |
| `tooling/deploy-link.ts`                  | liens, sans Hunspell/PR-feedback/memory/handoff                                      |
| `tooling/install-hunspell-dictionary.ts`  | Hunspell seul parmi les cinq familles de déploiement                                 |
| `tooling/pr-feedback-skill-test`          | PR-feedback seul parmi ces familles                                                  |
| source Rust memory ou handoff             | déploiement du consommateur correspondant                                            |
| test local ColGrep                        | Code Search local, sans intégration                                                  |
| implémentation ou skill Code Search       | local et intégration                                                                 |
| test d'intégration ColGrep                | intégration seule                                                                    |
| `.gitignore` / `.prettierignore`          | avant : aucun Prettier ; après : Prettier                                            |
| `package.json`, `bun.lock`                | gates consommatrices de la toolchain partagée                                        |

Le témoin `home/.config/nvim/lazy-lock.json`, passé à `moon ci --stdin
--downstream none` avec les 27 targets des sept tranches et du Harness,
résout **zéro target**. Les dépendances amont et les actions de préparation
restent activées dans cette expérience : aucune commande métier ni préparation
de ces familles ne démarre. Moon affiche un nœud de pipeline, pas une tâche
de préparation exécutée. L'installation smoke est volontairement hors de ce
jeu de sélection.

Pour les cas positifs locaux, `--no-actions --upstream none --cache off`
exclut les installations globales. Les quatre gates statiques exécutent leurs
commandes ; une exclusion temporaire du skill Code Search dans `.gitignore`
sélectionne Prettier et échoue avec le chemin fautif (exit 1), puis le même
contrôle passe après restauration (exit 0). Le typecheck sélectionné par
`oxfmt.config.ts` passe. Les deux validations Harness s'exécutent : trois cas
comportementaux, seize contrats d'activation et zéro rapport historique.
Zéro rapport n'est pas une preuve d'évaluation live.

Les expériences temporaires ne deviennent pas des tests recopiant Moon.
Les tests comportementaux existants restent la preuve des utilitaires ; les
requêtes seules ne prouvent ni une installation réussie ni l'intégration live.

Validation locale de cette correction : Actionlint et les quatre gates
statiques passent. La target TypeScript sélectionnée par `oxfmt.config.ts`
exécute 328 tests avec succès, zéro échec et une intégration Docker opt-in
ignorée. Les suites ciblées format/Shell/CSpell passent leurs 40 tests, les
gates Markdown leurs 10 tests et `cargo test --locked --manifest-path
tooling/arnes/Cargo.toml eval::` ses 41 tests sélectionnés.
Les tests de chemins utilisent `TMPDIR=/private/tmp` : le premier essai avec
l'alias temporaire macOS avait échoué sur un chemin absolu, limite déjà
documentée par #335. Les tests Markdown passent avec les 15 secondes du
contrat de la suite ; un premier appel avec le défaut Bun de 5 secondes
avait expiré. Aucun délai ni test du dépôt n'a été modifié.

## Mesure des jobs et durées

Sources : API Actions, heads finaux des PR, jobs de leur dernière tentative
au relevé. L'[annexe TSV](moon-gates-272-jobs.tsv) donne chaque job, SHA,
événement, runner, conclusion, timestamps UTC et URL. Les commits
intermédiaires et les push de fusion sur `main` sont exclus.

Une durée est `completed_at - started_at` en secondes, seulement avec runner
attribué. Les sommes incluent setup/teardown ; elles ne sont ni le temps mural
du pipeline, ni un coût de facturation. Les jobs à sélection vide comptent
aussi : Moon sélectionne les tâches à l'intérieur de jobs déjà déclenchés.

| Jalon / changement représentatif                                                                       | Révision                                   | Jobs PR / s runner | Jobs push branche / s runner | Total jobs / s runner |
| ------------------------------------------------------------------------------------------------------ | ------------------------------------------ | -----------------: | ---------------------------: | --------------------: |
| avant les dernières tranches : [#326, texte/Shell](https://github.com/SebastienElet/dotfiles/pull/326) | `b0d7a76f695ba1570539f5f96b244515ef14e010` |          17 / 1425 |                      9 / 709 |             26 / 2134 |
| [#334, Lua](https://github.com/SebastienElet/dotfiles/pull/334)                                        | `f1cacde25b435084d10cb25d3e53162fcb15561a` |          20 / 1773 |                     8 / 1250 |             28 / 3023 |
| [#335, déploiement](https://github.com/SebastienElet/dotfiles/pull/335)                                | `d30a1377aae86383c9d3b704aee6f971f48359fd` |          20 / 1765 |                      4 / 159 |             24 / 1924 |
| après les sept tranches : [#336, Code Search](https://github.com/SebastienElet/dotfiles/pull/336)      | `f3dbe994232f0f78f04e1a5c69525b1d30c10cb8` |          22 / 1138 |                      4 / 125 |             26 / 1263 |
| après les sept tranches : [#338, documentation](https://github.com/SebastienElet/dotfiles/pull/338)    | `ad79399556fbffb161df06399d59e3e6b0246361` |           20 / 801 |                       4 / 79 |              24 / 880 |

Les runners sont `macos-latest` et `ubuntu-latest`, séparés dans l'annexe.
Les quatre jobs push encore doublés après #335 appartiennent à Fish
(deux plateformes), Shell et texte. Sur la PR documentaire #338,
[Harness](https://github.com/SebastienElet/dotfiles/actions/runs/35068709559/job/104704891002)
consomme 137 s, dont 125 s pour l'agrégat inconditionnel.

Limites : #326 comprend un job annulé sans runner ; ses 946 s d'intervalle
API sont exclus du cumul runner. Les échantillons représentatifs sont tous
à la tentative 1 ; l'annexe historique #283 contient une tentative 2 et ne
compte pas sa tentative précédente. Les versions, caches, matrices et diff
changent entre les jalons. Ce relevé avant/après est descriptif : aucun
pourcentage de gain, économie garantie ou comparaison à charge égale n'est
établi. Une sélection négative locale n'est pas une durée CI après correction.

## Smoke, chevauchements et limites

`test.yml` conserve `moon run tooling:smoke-minimal` sur `macos-latest` à
chaque PR et push `main`, puis l'intégration upgrade Neovim. Ce workflow
n'est pas modifié. La preuve historique récente est le
[smoke #336](https://github.com/SebastienElet/dotfiles/actions/runs/35068346707).

La [PR #299](https://github.com/SebastienElet/dotfiles/pull/299), head
`6513d0220fe010cb57bc2467c0db42330e2bce07`, reste un brouillon ouvert en
conflit au relevé. Elle touche notamment `harness/moon.yml`, les agrégats et
le smoke, ajoute `workstation:install` et des feuilles Make, et remplace le
smoke par une installation simple. Ces propositions ne correspondent plus
aux ADR en vigueur ni au `repository:install` déjà livré. Elle ne doit pas
réintroduire ce graphe ni supprimer les contrôles de rejeu du smoke. Aucun
commit de #299 n'est repris ; son éventuelle clôture reste séparée.

#124 reste indépendant : aucun test Doctor ni document Doctor modifié.
`test-arnes.yml` et ses cinq gates Ubuntu restent inchangés.

Les runtimes bootstrap des jobs et le smoke systématique conservent un
coût même pour un diff sans target métier affectée. La suite TypeScript
générale prépare encore le binaire memory parce qu'elle contient ses tests
consommateurs ; aucune nouvelle granularité de cette suite n'est revendiquée.
