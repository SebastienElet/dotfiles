# Validation locale de CodeGraph

- **Date** : 2026-08-14
- **Système exercé** : macOS 26.5.2 (build 25F84), arm64
- **CodeGraph** : 1.5.0
- **Codex** : 0.147.0
- **Claude Code** : 2.1.232
- **Cursor Agent** : 2026.08.11-e8db854
- **Linux non exercé**

## Barrières

- `make -Bn codegraph` réussit et montre l'installation épinglée de
  `@colbymchenry/codegraph@1.5.0`, les trois configurations MCP et les liens de skill.
- `make codegraph-test` réussit sur macOS et exécute la mesure de dépôt, la configuration réelle des
  trois agents, la matrice MCP de fraîcheur et l'audit réseau récursif.
- `bash scripts/codegraph_network_test` prouve d'abord que le moniteur détecte un socket ouvert par
  un petit-enfant, puis suit récursivement les processus d'initialisation, de synchronisation et du
  serveur MCP ainsi que le daemon. `lsof -nP -a -p PID -i` n'observe ensuite aucun socket réseau et
  le daemon est arrêté en fin de probe. L'échantillonnage toutes les 50 ms ne couvre pas une
  connexion plus brève.

## Mesures sur la fixture publique

L'indexation initiale prend 1 s à la granularité entière de `SECONDS`, 0,64 s de CPU utilisateur et
0,22 s de CPU système. Le maximum RSS vaut 309 542 912 octets et l'index occupe 164 Kio. La
resynchronisation explicite prend 165 ms.

| Requête                          | Latence |
| -------------------------------- | ------: |
| Dépendances initiales            |  199 ms |
| Passage à la branche alternative |    9 ms |
| Retour à la branche principale   |    4 ms |
| Édition                          |    3 ms |
| Renommage                        |   13 ms |
| Suppression                      |   10 ms |
| Redémarrage du serveur           |    3 ms |
| Interruption du watcher          |  275 ms |
| Réconciliation                   |    4 ms |

| Scénario              | Résultat |
| --------------------- | -------- |
| Index initial         | frais    |
| Changement de branche | frais    |
| Édition               | frais    |
| Renommage             | frais    |
| Suppression           | frais    |
| Redémarrage           | frais    |
| Watcher interrompu    | frais    |
| Réconciliation        | fraîche  |
| Arrêt du daemon       | confirmé |

## Routage des agents

Les six smokes utilisent uniquement la fixture publique versionnée. Les assertions portent sur les
événements d'appel réels des JSONL, pas sur la présence de mots dans le prompt ou la seule qualité
de la réponse.

| Agent        | Exploration structurelle                                                                            | Littéral exact                                                                                 |
| ------------ | --------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| Claude Code  | événement `tool_use` vers `mcp__codegraph__codegraph_explore` ; trois dépendances retournées        | événement `Bash` exécutant `rg --fixed-strings` ; aucun appel CodeGraph                        |
| Codex        | événement `mcp_tool_call` terminé vers `codegraph/codegraph_explore` ; trois dépendances retournées | événement `command_execution` terminé exécutant `rg -F` ; aucun appel CodeGraph                |
| Cursor Agent | événement `mcpToolCall` réussi vers `codegraph_explore` ; trois dépendances retournées              | événement `grepToolCall` réussi pour le littéral, avec backend ripgrep ; aucun appel CodeGraph |

## Écarts retenus par rapport à l'issue

- L'index reste dans `.codegraph/` au sein de chaque dépôt ou worktree, sans symlink externe.
- La surface MCP reste limitée à l'unique outil upstream `codegraph_explore`.
- Aucun wrapper, plafond de résultats ou délai d'attente maison n'est ajouté.
- Aucune donnée ni mesure issue d'un dépôt privé n'entre dans cette validation.
