# Refactor du setup du hook de handoff Claude

## Objectif

Rendre le setup du hook lisible comme une courte procédure : valider les prérequis, lire les
réglages, ajouter notre entrée puis remplacer le fichier. Le setup reste distinct du hook runtime
`tooling/agent-handoff`.

## Structure

- `tooling/claude-handoff-hook` ne connaît que l'entrée Stop qu'il installe.
- Ses petites fonctions suivent les étapes de la procédure et `main` en donne l'ordre.
- `tooling/claude-handoff-hook-test` vérifie uniquement les garanties du setup.
- `claude-handoff-hook` reste une façade phony du Makefile.
- Le lint CI appelle directement le script de test, sans target Make de test.

Le script reçoit les chemins absolus courant et historique du hook runtime. Le Makefile lui transmet
`${DOTFILES_PATH}/tooling/agent-handoff` et `${DOTFILES_PATH}/scripts/agent_handoff`, y compris
lorsque le dépôt n'est pas installé sous `~/.dotfiles`.

## Garanties

Le setup crée `~/.claude/settings.json` si nécessaire, préserve les réglages sans rapport avec son
entrée, migre l'ancien chemin et les doublons vers une entrée unique avec `args: []`, reste
idempotent et refuse un JSON ambigu, un handler possédé invalide, un `HOME` relatif ou un
`settings.json`/`.claude` non régulier sans remplacer le fichier. L'écriture passe par un fichier
temporaire adjacent puis un renommage atomique. Le setup précède explicitement les hooks de mesure
afin que les deux écritures ne se recouvrent pas sous `make -j`.

Il ne valide ni ne normalise le reste du schéma Claude. Une structure incompatible sur le chemin
qu'il doit modifier fait naturellement échouer `jq`; une structure inconnue ailleurs est conservée.

## Tests et périmètre

Les scénarios couvrent la création, la préservation d'un handler inconnu, la migration, les
doublons, l'idempotence, l'ordre des writers et le refus d'un contenu ou chemin invalide. Les
validations exhaustives du schéma, les signaux, la sélection individuelle des scénarios et une
target Make de test restent hors périmètre.

`.PHONY: claude-code` et `.PHONY: claude-handoff-hook` restent immédiatement devant leurs targets.
Les targets Codex voisines ne sont pas modifiées.
