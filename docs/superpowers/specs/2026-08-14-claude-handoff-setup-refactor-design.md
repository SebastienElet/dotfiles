# Refactor du setup du hook de handoff Claude

## Objectif

Rendre lisibles la recette `claude-handoff-hook` et son test CI sans modifier leur comportement.
Le setup du hook reste distinct du hook runtime `scripts/agent_handoff`.

## Structure

- `scripts/setup/claude_handoff_hook` configure uniquement l'entrée Stop dans
  `~/.claude/settings.json`.
- `scripts/setup/claude_handoff_hook_test` vérifie directement ce script avec un `HOME` temporaire.
- `scripts/agent_handoff` reste le programme exécuté par Claude Code lors d'un événement Stop.
- `claude-handoff-hook` reste la façade publique du Makefile et délègue au script de setup.
- Le workflow CI appelle directement le script de test, sans target Make dédiée.

Le script de setup reçoit le chemin absolu du hook runtime en argument. Le Makefile lui transmet
`${DOTFILES_PATH}/scripts/agent_handoff`, ce qui conserve la sémantique actuelle lorsque le dépôt
n'est pas installé sous `~/.dotfiles`.

## Comportement et erreurs

Le setup conserve les propriétés existantes : fusion idempotente, préservation des autres réglages,
écriture atomique dans le même répertoire, permissions `0600` et refus d'un JSON invalide sans
remplacer le fichier source. Un argument absent, `jq` indisponible ou une entrée illisible provoque
un échec explicite.

## Tests et Makefile

Le test nomme séparément les scénarios de création, préservation, idempotence, permissions et JSON
invalide. Il travaille sous un `HOME` temporaire et appelle le script de setup, tandis que le dry-run
de `claude-code` vérifie séparément le câblage Makefile.

`.PHONY: claude-code` et `.PHONY: claude-handoff-hook` sont placés immédiatement devant leurs
targets. Les targets Codex voisines restent hors périmètre : les modifier élargirait la PR sans
servir la lisibilité demandée.
