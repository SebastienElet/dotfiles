# Arnes Doctor : référence du diagnostic actuel

Cette référence est commune aux livrables documentaires de
[#126](https://github.com/SebastienElet/dotfiles/issues/126) et
[#124](https://github.com/SebastienElet/dotfiles/issues/124).
Arnes est la CLI locale du harnais partagé entre Claude Code, Cursor et Codex.
Doctor compare les déclarations du manifeste aux ressources observables : une
configuration installée ne prouve ni son chargement ni son exécution par un agent.

Les [ADR-001](adr/001-makefile-installateur.md),
[ADR-003](adr/003-deploiement-par-symlinks.md) et
[ADR-038](adr/038-frontieres-home-harness-tooling.md), acceptées et révisées le
7 septembre 2026, séparent orchestration, déploiement et diagnostic. La
[carte du dépôt](ARCHITECTURE.md) conserve la topologie ; le
[manifeste versionné](../home/.arnes.yaml) conserve les déclarations gérées.
L'aide `arnes --help` et `arnes doctor --help` reste la référence syntaxique du
binaire installé ; les exemples ci-dessous expliquent comment interpréter ses résultats.

## Lecture seule et limites d'observation

Doctor ne répare, ne synchronise et n'installe aucune ressource gérée. Il inspecte
les fichiers, liens, représentations et commandes déclarés, sans exécuter les
hooks ni démarrer les serveurs MCP. Les snapshots des fixtures décrites plus bas
vérifient l'absence de modification de leur dépôt et de leur HOME.

Doctor ne lance plus le résolveur externe Codex : `doctor skills` lit la
configuration locale et signale l'inventaire actif des plugins comme `unsupported`.
Un réglage `enabled=true` conserve son diagnostic de politique mais ne prouve
ni disponibilité ni activation ; aucun artefact actif n'est déduit du cache.
La [frontière externe](arnes-capacites-externes.md#frontières-de-lecture) précise
la capacité d'observation ainsi réduite, conformément à la décision D1 de #111.

L'absence d'exécution du résolveur est exercée par un témoin placé hors des arbres
snapshotés, sur Doctor direct et agrégé, dans les deux formats. La lecture seule
du chemin possédé repose sur la suppression de cette exécution ; des snapshots
identiques ne suffisent toujours pas à exclure toute écriture temporaire effacée.
Aucun résultat ne certifie une session réelle d'agent.

Cette distinction concerne Doctor : Arnes expose aussi `setup hooks`, `export`,
`eval` et `measure`, dont certains parcours écrivent. `arnes setup hooks --agent
claude` réconcilie explicitement les hooks user déclarés ; `doctor hooks` observe
leur état. Setup cible un seul agent obligatoire (`claude`, `cursor` ou `codex`),
avec `--scope user` par défaut ; `project` est refusé. Il ne constitue pas une
synchronisation générale. Aucune commande `arnes sync` n'est disponible : une
synchronisation éventuelle n'est pas un comportement livré.

## Sélection, environnement et valeurs par défaut

Lancer Doctor depuis la racine du dépôt à examiner : le répertoire courant est
sa racine projet, sans remontée automatique à la racine Git. `HOME` doit être un
chemin absolu non vide ; le manifeste est lu dans `$HOME/.arnes.yaml`, sans
repli implicite. Un manifeste absent, illisible ou invalide empêche le diagnostic
des ressources. Les filtres ne dispensent pas de valider le manifeste entier ;
`doctor manifest` ne filtre pas cette validation.

Si le manifeste est un lien vers `home/.arnes.yaml` dans les dotfiles, sa cible
identifie aussi le dépôt de déploiement utilisé pour les instructions et skills
gérés de portée user. Cela ne rebascule pas toutes les ressources vers ce dépôt :
les sources des rules et les ressources project restent relatives au répertoire courant.
Les commandes locales sont inspectées selon leur portée et le `PATH` reçu.

- Sans ressource, `arnes doctor` agrège les dix familles ci-dessous, dans cet ordre.
  Après validation du manifeste, une erreur d'une famille n'empêche pas les suivantes.
- Une ressource explicite limite le contrôle à cette famille, par exemple
  `arnes doctor manifest` ou `arnes doctor hooks`.
- `--agent claude|cursor|codex` sélectionne un agent ; absent, le filtre ne
  restreint pas les agents déclarés. Les capacités prises en charge restent
  différentes selon l'agent et la portée.
- `--scope user|project` sélectionne une portée. Sans cette option, `config`,
  `instructions`, `skills`, `prompts`, `commands`, `rules` et `hooks` prennent
  `user`. Dans l'agrégat, MCP et statusline examinent toutes leurs portées
  déclarées. En appel direct, **`doctor mcp` prend `user`**, tandis que
  **`doctor statusline` conserve toutes les portées déclarées**. Un contrôle
  explicite `--scope project` complète donc le passage par défaut.

## Ressources et différences entre agents

Le tableau décrit les contrôles implémentés, sous réserve des déclarations du
manifeste ; il ne constitue pas un inventaire de l'installation. Une combinaison
absente ou non prise en charge peut produire `unsupported`. Certaines absences
restent silencieuses, notamment une statusline sans déclaration correspondante
et des capacités externes sans inventaire observable.

| Ressource      | Ce que Doctor observe                                                                                                                                                         | Limites par agent ou portée                                                                                                                                                                                                                                   |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `manifest`     | Lecture, version et validité des déclarations.                                                                                                                                | Commun aux trois agents ; ne vérifie pas leur installation.                                                                                                                                                                                                   |
| `config`       | Présence et syntaxe JSON/TOML de la configuration native ; comparaison des valeurs user déclarées.                                                                            | Claude Code, Cursor et Codex, user/project. Cursor utilise des fichiers CLI distincts selon la portée. Les valeurs user ne sont pas imposées au projet ; le parseur ne certifie pas tout le schéma natif de l'agent.                                          |
| `instructions` | Sources, includes et conformité des projections déclarées.                                                                                                                    | Claude user : liens ; Claude project : includes ; Codex user : contenu assemblé. Cursor et Codex project ne sont pas pris en charge par ce Doctor.                                                                                                            |
| `skills`       | Skills gérés, projections et références locales ; inventaire externe observable.                                                                                              | Trois agents : liens feuille user et racine projet selon les déclarations. Les plugins et skills système ont les limites distinctes décrites dans la [référence externe](arnes-capacites-externes.md). Aucun diagnostic ne prouve leur activation en session. |
| `prompts`      | Source, includes, variables et projection du contenu.                                                                                                                         | Claude user/project et Cursor project. Codex et Cursor user non pris en charge ; une représentation par symlink reste `unsupported`.                                                                                                                          |
| `commands`     | Liaison déclarée d'une commande à un prompt, description et projection.                                                                                                       | Claude user/project seulement, même si `prompts` prend aussi en charge Cursor project. Aucune invocation de la commande.                                                                                                                                      |
| `rules`        | Source et cible des symlinks déclarés.                                                                                                                                        | Claude et Cursor user seulement ; ni Codex ni portée project. Ne prouve pas l'application de la règle.                                                                                                                                                        |
| `hooks`        | Configuration, événements, commandes et réglages d'exécution attendus.                                                                                                        | Trois agents, user seulement. Événements propres à chaque adaptateur. Cursor : `measurement` seulement ; `memory` non pris en charge, `handoff` et `output-discipline` refusés dès le manifeste. Ne prouve pas qu'un hook a tourné ou réussi.                 |
| `mcp`          | Enregistrements déclarés, commande, arguments, références de variables d'environnement et état `enabled` déclaré ; collisions entre portées et disponibilité de l'exécutable. | Trois agents, user/project selon les déclarations. Pas de démarrage, de connexion ni de vérification du service distant.                                                                                                                                      |
| `statusline`   | Liste ordonnée `tui.status_line` comparée au manifeste.                                                                                                                       | Codex user/project seulement ; Claude et Cursor ne sont pas audités. Aucun test du rendu TUI ; une sélection vide peut rendre `[]`.                                                                                                                           |

Un filtre limite la sélection, pas nécessairement toutes les lectures : MCP
peut inspecter l'autre portée pour détecter une collision d'enregistrement.
Les [capacités externes](arnes-capacites-externes.md) détaillent séparément les
registres Claude, la configuration Codex, les plugins locaux Cursor et les inventaires
non observables. Leurs contenus ne sont pas reproduits ici.

## Formats, états et sorties

`--format human`, valeur par défaut, compte les diagnostics `healthy`, masque
leur détail et affiche les problèmes et limites `unsupported`. L'agrégat omet
les sections vides ; les skills sont regroupés par agent.
`-v` ou `--verbose` rétablit les détails `healthy` ; il est désactivé par défaut.

`--color auto|always|never` règle la sortie humaine (`auto` par défaut).
En mode `auto`, stdout doit être un TTY et `NO_COLOR` absent ou vide ; `always`
prime sur `NO_COLOR`. Le JSON et `never` ne produisent pas de séquences ANSI.

`--format json` restitue tous les diagnostics produits, dans l'ordre canonique,
sous forme d'un tableau d'objets `resource`, `state`, `message`. Il n'invente
pas d'entrée pour un inventaire absent. Agent et portée figurent dans les messages,
pas dans des champs JSON dédiés. Le JSON est incompatible avec `--verbose` et
`--color always` : ces combinaisons échouent avant diagnostic, sur stderr, code `2`.

| État          | Sens dans le périmètre observé                                                                | Contribution au code de sortie                  |
| ------------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| `healthy`     | Le contrôle effectué correspond à l'attendu.                                                  | `0`                                             |
| `unsupported` | Contrôle non déclaré, non pris en charge ou observation indisponible.                         | `0`, non bloquant, sans garantie de conformité. |
| `drift`       | Écart observé, par exemple une destination manquante ou différente.                           | `1`                                             |
| `error`       | Entrée invalide ou impossibilité d'exploiter une ressource, par exemple un fichier illisible. | `2`                                             |

Le code final est le maximum des contributions : `error` prime sur `drift`.
Un tableau vide sort aussi avec `0` (`No diagnostics` en humain). Un code `0`
ne signifie donc ni couverture complète ni fonctionnement des agents.
Les erreurs d'arguments et d'écriture de sortie valent également `2` ; elles
peuvent produire stderr sans rapport JSON. Un diagnostic `error` après parsing
est, lui, restitué dans le format choisi.

## Exemples d'utilisation

Depuis la racine du dépôt à examiner, avec le manifeste déployé dans HOME :

```sh
arnes doctor
arnes doctor --scope project
```

Le premier passage est l'agrégat par défaut, pas un audit de tous les scopes de
toutes les ressources. Pour une famille puis un filtre agent/portée :

```sh
arnes doctor manifest
arnes doctor hooks --agent claude
arnes doctor config --agent cursor --scope user
```

Pour conserver tous les diagnostics ou détailler un contrôle humain :

```sh
arnes doctor --format json
arnes doctor skills --agent claude --scope user -v --color never
```

Le JSON aide à distinguer les états ; son code de sortie reste celui du
diagnostic. Examiner les entrées `unsupported` et les omissions avant de tirer
une conclusion. Pour les hooks, une configuration conforme n'est pas une preuve
d'activité ou de résultat : voir l'[ADR-043 acceptée](adr/043-telemetrie-arnes-minimale-et-bornee.md)
et la [mesure locale](arnes-mesure.md).

## Vérification et portée des preuves

Vérification locale du 16 septembre 2026 contre `5be53d6` : macOS 26.6.2,
Darwin arm64, Rust/Cargo 1.98.0, binaire Arnes 0.1.0 compilé depuis ce checkout.
L'aide CLI et les modules [Doctor](../tooling/arnes/src/doctor.rs),
[CLI](../tooling/arnes/src/cli.rs) et
[diagnostics](../tooling/arnes/src/diagnostic.rs) ont été confrontés aux exécutions.

Les sept commandes des exemples ont été exécutées dans un dépôt Git temporaire
et un HOME isolé. La fixture reprend celle de
[`doctor_aggregate`](../tooling/arnes/tests/support/doctor_aggregate.rs), avec en
plus les configurations Cursor user/project. Les dix familles apparaissent au
passage par défaut : Claude user porte les projections, commandes, rules et
hooks ; Claude project porte MCP ; Codex project porte la statusline ; Cursor
porte le contrôle de configuration. Les combinaisons non couvertes produisent
des limites, pas une parité artificielle.

Le passage par défaut sort `0` avec `healthy` et `unsupported`. La suppression
d'une projection donne `1` ; un manifeste invalide donne `2`. Le passage project
sort `1` dans cette fixture partielle. Les snapshots avant/après chaque commande
(fichiers, contenus, permissions et cibles des liens) sont identiques, y compris
dans les cas d'échec ; ils ne détectent pas une écriture temporaire ensuite effacée.
La préparation `setup hooks` précède ces snapshots. Le PATH des exemples ne
contient aucun agent réel ; aucune installation globale n'a été effectuée.

Les tests existants [`doctor_aggregate`](../tooling/arnes/tests/doctor_aggregate.rs)
exercent en complément ordre, déterminisme, filtres, dérives des neuf familles
après manifest, absence de mutation et poursuite après une erreur de ressource.
Les suites de ressources exercent des représentations de fichiers et des doubles
de résolveurs : elles ne valident pas le chargement par Claude Code, Cursor ou
Codex réels. La validation locale ne couvre pas Linux ni l'installation complète
du poste ; la configuration de CI seule ne fournit pas ces preuves.
