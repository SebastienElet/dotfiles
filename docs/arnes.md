# Arnes

## Usage

Arnes compare le manifeste [`home/.arnes.yaml`](../home/.arnes.yaml) aux ressources
locales de Claude Code, Codex et Cursor. Lancer les commandes depuis la racine du
dépôt à examiner ; `HOME` doit être absolu et contenir `.arnes.yaml`.
Moon reste l'orchestrateur d'installation. L'aide `arnes <commande> --help`
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

Pour appliquer une déclaration prise en charge, sélectionner une seule ressource :

```sh
arnes sync skills --agent codex --scope user
arnes sync config --agent claude --scope user
arnes sync mcp --agent claude --scope project
arnes setup hooks --agent claude
```

Sync exige ressource, agent et portée ; il n'installe aucun outil ou plugin et ne
supprime aucune ressource. Relancer ensuite le Doctor ciblé. Une projection liée
divergente est refusée. Modifier une inscription MCP ou un fichier divergent
d'instructions, de prompts ou de commandes exige un reçu correspondant à son état ;
une entrée déjà conforme sans reçu n'est pas adoptée. Config et statusline remplacent
les valeurs déclarées sans reçu, en conservant les autres valeurs ; un type
incompatible est refusé.
Les combinaisons non prises en charge et sélections vides retournent un code non nul.

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

Sync publie par fichier ; une erreur ultérieure peut laisser des mutations déjà
effectuées. Il n'existe pas de transaction globale entre configuration et reçu.
Conserver les diagnostics d'échec avant de rejouer ; ne pas effacer une destination
étrangère ou un reçu pour forcer le succès.

`moon run arnes:test` exerce la CLI dans des dépôts et HOME temporaires sur les
cibles macOS et Ubuntu. Ces tests ne prouvent pas l'activation des agents réels.
Depuis un worktree, appliquer les [précautions de validation](../README.md#contributing-from-a-worktree).
