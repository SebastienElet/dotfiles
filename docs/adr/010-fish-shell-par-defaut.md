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

Fish devient le shell par défaut, installé par le profil minimal. La configuration zsh et son
déploiement sont supprimés ; les processus non interactifs utilisent explicitement leur
interpréteur.

## Conséquences

- Complétion et historique fournis sans plugins ; démarrage plus rapide.
- Fish n'est pas POSIX : les extraits d'installation copiés depuis une
  documentation doivent être traduits, ce dont témoignent plusieurs correctifs
  de syntaxe.
- Aucun second shell interactif n'est maintenu par le dépôt.

## Alternatives écartées

- zsh avec prezto ou antigen : remplacés, coût d'entretien et lenteur au
  démarrage.
- bash : aucune des fonctionnalités interactives recherchées.
