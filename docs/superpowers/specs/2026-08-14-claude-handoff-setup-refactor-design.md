# Refactor du setup du hook de handoff Claude

## Objectif

Installer le hook de handoff et les hooks de mesure Claude dans une seule mutation sûre des
réglages. Le setup reste distinct du hook runtime `tooling/agent-handoff`.

## Structure

- Arnes valide les chemins et installe en même temps ses hooks de mesure et l'entrée Stop de
  handoff.
- `claude-handoff-hook` reste une façade phony du Makefile et appelle directement Arnes.
- `tooling/makefile-test` vérifie l'appel Arnes complet depuis un répertoire extérieur au dépôt.
- Le lint CI appelle directement le script de test, sans target Make de test.

Le Makefile transmet à Arnes les chemins absolus des hooks runtime courant et historique, résolus
depuis `DOTFILES_PATH`, y compris lorsqu'il est appelé hors du dépôt.

## Garanties

Arnes crée `~/.claude/settings.json` si nécessaire, préserve les réglages sans rapport avec ses
entrées, migre l'ancien chemin et les doublons vers une entrée unique avec `args: []`, puis reste
idempotent. Il refuse les structures ambiguës ou invalides et les chemins non réguliers sans
mutation. Son échange atomique compare le fichier remplacé au snapshot lu et restaure une écriture
concurrente au lieu de l'écraser.

Les événements et variantes Claude connus sont validés avant mutation; les extensions inconnues
restent opaques et sont conservées.

## Tests et périmètre

Les scénarios couvrent la création, la préservation, la migration, les doublons, l'idempotence, le
refus sans mutation d'un contenu ou chemin invalide et les écritures concurrentes avant et après
publication. Une target Make de test reste hors périmètre.

`.PHONY: claude-code` et `.PHONY: claude-handoff-hook` restent immédiatement devant leurs targets.
Les targets Codex voisines ne sont pas modifiées et aucun script Shell ne lit ou n'écrit les
réglages Claude.
