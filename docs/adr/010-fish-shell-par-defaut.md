# ADR-010 — Fish comme shell par défaut

- **Statut** : accepté
- **Date** : 2025-01
- **Commits** : `b697242`, `5cb3a76`, `5375b5c`, `c7bec47`

## Contexte

La configuration zsh reposait sur des gestionnaires de plugins successifs
(prezto puis antigen) qui pesaient sur le temps de démarrage et demandaient un
entretien constant, pour des fonctionnalités — complétion contextuelle,
suggestion à la frappe, coloration syntaxique — que Fish fournit nativement.

## Décision

Fish devient le shell par défaut, installé et déclaré comme shell de connexion
par le `Makefile`. La configuration zsh est supprimée (`5375b5c`), à
l'exception d'un `.zshrc` minimal conservé pour les outils qui présument un
shell POSIX ou injectent leur initialisation dans zsh (`c7bec47`).

## Conséquences

- Complétion et historique fournis sans plugins ; démarrage plus rapide.
- Fish n'est pas POSIX : les extraits d'installation copiés depuis une
  documentation doivent être traduits, ce dont témoignent plusieurs correctifs
  de syntaxe.
- Le `.zshrc` minimal doit rester minimal, sous peine de reconstituer une
  seconde configuration de shell.

## Alternatives écartées

- zsh avec prezto ou antigen : remplacés, coût d'entretien et lenteur au
  démarrage.
- bash : aucune des fonctionnalités interactives recherchées.
