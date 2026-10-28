# Arnes

## Usage

Arnes compare le manifeste [`home/.arnes.yaml`](../home/.arnes.yaml) aux ressources
locales de Claude Code, Codex et Cursor. Lancer les commandes depuis la racine du
dépôt à examiner ; `HOME` doit être absolu et contenir `.arnes.yaml`.
Moon reste l'orchestrateur d'installation. L'aide `arnes <commande> --help`
décrit les options du binaire utilisé.
Dans les dotfiles, Arnes sert seulement à vérifier la configuration du harnais ;
les hooks s'installent à la main.

## Procédure

Vérifier les hooks après leur installation manuelle : `moon run harness:doctor`, hors du profil d'installation.

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

Pour observer les exécutions collectées et déclarer séparément leur résultat :

```sh
arnes measure list --without-result
arnes measure report --format json
arnes measure outcome RUN_ID --status pass --oracle cargo-test
```

Le silence mesure le temps depuis le dernier événement observable, sans prouver
un blocage ou un processus vivant. Un outcome nomme un oracle externe ; Arnes
ne l'exécute pas. Un changement d'outcome exige `--replace` et conserve l'historique.
La rétention des mesures suit l'[ADR-043](adr/043-telemetrie-arnes-minimale-et-bornee.md).

## Limites et vérification

Doctor observe des fichiers et configurations, sans démarrer les agents, hooks ou
serveurs MCP. Une projection conforme ne prouve pas son chargement en session.
Les capacités externes dépendent des racines et registres observables ; les
inventaires indisponibles peuvent rester absents. Pour un plugin Cursor obligatoire,
la présence structurelle suffit au contrôle et l'activation reste `unobserved`.

`moon run arnes:test` exerce la CLI dans des dépôts et HOME temporaires sur les
cibles macOS et Ubuntu. Ces tests ne prouvent pas l'activation des agents réels.
Depuis un worktree, appliquer les [précautions de validation](../README.md#contributing-from-a-worktree).
