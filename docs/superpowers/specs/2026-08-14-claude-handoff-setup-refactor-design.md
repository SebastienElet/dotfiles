# Refactor du setup du hook de handoff Claude

## Objectif

Rendre le setup du hook lisible comme une courte procédure : valider les prérequis, lire les
réglages, ajouter notre entrée puis remplacer le fichier. Le setup reste distinct du hook runtime
`scripts/agent_handoff`.

## Structure

- `scripts/setup/claude_handoff_hook` ne connaît que l'entrée Stop qu'il installe.
- Ses petites fonctions suivent les étapes de la procédure et `main` en donne l'ordre.
- `scripts/setup/claude_handoff_hook_test` vérifie uniquement les garanties du setup.
- `claude-handoff-hook` reste une façade phony du Makefile.
- La CI appelle directement le script de test, sans target Make de test.

Le script reçoit le chemin absolu du hook runtime. Le Makefile lui transmet
`${DOTFILES_PATH}/scripts/agent_handoff`, y compris lorsque le dépôt n'est pas installé sous
`~/.dotfiles`.

## Garanties

Le setup crée `~/.claude/settings.json` si nécessaire, préserve les réglages sans rapport avec son
entrée, migre son ancienne entrée vers la forme `args: []`, reste idempotent et refuse un JSON
invalide sans remplacer le fichier. L'écriture passe par un fichier temporaire adjacent puis un
renommage atomique.

Il ne valide ni ne normalise le reste du schéma Claude. Une structure incompatible sur le chemin
qu'il doit modifier fait naturellement échouer `jq`; une structure inconnue ailleurs est conservée.

## Tests et périmètre

Les scénarios couvrent la création, la préservation d'un handler inconnu, la migration,
l'idempotence et le refus d'un JSON invalide. Les validations exhaustives du schéma, les signaux,
les liens symboliques, la sélection individuelle des scénarios et une target Make de test restent
hors périmètre.

`.PHONY: claude-code` et `.PHONY: claude-handoff-hook` restent immédiatement devant leurs targets.
Les targets Codex voisines ne sont pas modifiées.
