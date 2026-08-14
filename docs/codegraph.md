# CodeGraph

## Installation

Depuis le checkout canonique uniquement :

```bash
make codegraph
```

La cible épingle CodeGraph 1.5.0, configure Codex, Claude Code et Cursor, désactive les sorties
réseau automatiques et distribue la skill.

## Activation

Les agents utilisent `codegraph-repository-size .` uniquement avant une exploration structurelle
sans index. `initialize: true` autorise `codegraph init`. Une recherche exacte ne déclenche jamais
la mesure.

## Santé et fraîcheur

```bash
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 codegraph status --json
CODEGRAPH_TELEMETRY=0 CODEGRAPH_NO_UPDATE_CHECK=1 CODEGRAPH_NO_DOWNLOAD=1 codegraph sync
```

Une seule synchronisation est tentée. Un second échec impose un repli explicite vers `rg` et `fd`.

## Cycle de vie

```bash
codegraph daemon
codegraph uninit
codegraph uninstall --target=codex,claude,cursor --location=global --yes --keep-cli
```

`codegraph daemon` permet d'arrêter un daemon identifié. `codegraph uninit` supprime l'index du
dépôt courant après confirmation. La désinstallation retire les configurations agent mais conserve
le CLI ; la suppression du CLI reste gérée par Volta.

## État et confidentialité

L'index réside dans `.codegraph/` et n'est pas versionné. CodeGraph ne nécessite ni Ollama, ni
serveur de modèle, ni embedding distant. Les exclusions du dépôt et les exclusions upstream
écartent les dépendances et sorties générées ; un répertoire sensible suivi doit être exclu par le
dépôt avant initialisation.

## Limites

La surface MCP par défaut contient `codegraph_explore`. `trace` n'existe plus séparément et
`impact` n'est pas réactivé. CodeGraph ne remplace ni LSP ni DAP.
