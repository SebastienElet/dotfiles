# Arnes

## Usage

Arnes compare le manifeste [`home/.arnes.yaml`](../home/.arnes.yaml) aux ressources
locales de Claude Code, Codex et Cursor. Lancer les commandes depuis la racine du
dépôt à examiner ; `HOME` doit être absolu et contenir `.arnes.yaml`.
Les tâches Moon des dotfiles installent les hooks du manifeste ; Arnes diagnostique
les ressources du harnais sans les modifier.
L'aide `arnes <commande> --help`
décrit les options du binaire utilisé.

## Procédure

Diagnostiquer sans réparation, puis cibler explicitement l'agent et la portée :

```sh
arnes doctor
arnes doctor skills --agent codex --scope user --format json
arnes doctor instructions --agent claude --scope project
```

Le passage sans filtre ne couvre pas toutes les portées de toutes les ressources.
Le JSON expose `resource`, `state`, `message`. Examiner aussi `unsupported` et les
ressources absentes du rapport : le code `0` n'atteste pas une couverture complète.
`drift` contribue au code `1`, `error` au code `2`, qui prime.

Pour installer les hooks déclarés, lancer la tâche Moon de l'agent concerné :

```sh
moon run harness:claude-hooks
moon run harness:codex-hooks
moon run harness:cursor-hooks
```

Ces tâches acquièrent les exécutables nécessaires, puis fusionnent les hooks gérés
dans le fichier de l'agent. Les autres réglages et les handlers dont la commande
n'est pas reconnue comme gérée restent conservés. Une
configuration déjà identique n'est pas réécrite. Le Doctor ciblé s'exécute après
l'installation ; son résultat conforme reste silencieux. Pour consulter son rapport :

```sh
arnes doctor hooks --agent claude --color never
```

Moon construit Arnes et `agent-measure` depuis leurs dépôts officiels, aux commits
épinglés dans les tâches. Le hook `output-discipline-hook` et `agent-handoff`
proviennent des dépendances Bun verrouillées. Arnes n'exécute aucun de ces hooks.

Pour observer les exécutions collectées et déclarer séparément leur résultat :

```sh
agent-measure list --without-result
agent-measure report --format json
agent-measure outcome RUN_ID --status pass --oracle cargo-test
```

Le silence mesure le temps depuis le dernier événement observable, sans prouver
un blocage ou un processus vivant. Un outcome nomme un oracle externe ; `agent-measure`
ne l'exécute pas. Un changement d'outcome exige `--replace` et conserve l'historique.
La rétention des mesures suit l'[ADR-043](adr/043-telemetrie-arnes-minimale-et-bornee.md).

## Limites et vérification

Doctor observe des fichiers et configurations, sans démarrer les agents, hooks ou
serveurs MCP. Une projection conforme ne prouve pas son chargement en session.
Les capacités externes dépendent des racines et registres observables ; les
inventaires indisponibles peuvent rester absents. Pour un plugin Cursor obligatoire,
la présence structurelle suffit au contrôle et l'activation reste `unobserved`.

L'installateur refuse les fichiers malformés, surdimensionnés, symboliques ou liés
plusieurs fois, ainsi qu'un répertoire d'agent symbolique. Un verrou refuse une
écriture concurrente par cet utilitaire ; un verrou abandonné exige une inspection
explicite. La publication
remplace un fichier à la fois, sans transaction entre agents ni protection contre un
processus hostile qui échange les chemins pendant l'écriture. Le Doctor s'exécute
après la publication ; ne pas considérer ces étapes comme une transaction.

`moon run tooling:deployment-test` exerce les installations et leur rejeu dans des
HOME temporaires. `moon run arnes:test` exerce la copie Rust encore présente dans le
dépôt, pas le binaire officiel déployé. Ces tests ne prouvent pas le chargement des
hooks dans les sessions réelles de Claude, Codex et Cursor.
Depuis un worktree, appliquer les [précautions de validation](../README.md#contributing-from-a-worktree).
