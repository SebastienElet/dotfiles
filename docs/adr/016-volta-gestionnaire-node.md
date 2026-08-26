# ADR-016 — Volta comme gestionnaire Node

- **Statut** : accepté
- **Date** : 2021-05
- **Commits** : `498aacd`

## Contexte

nvm (2017) puis fnm (2020) exigeaient un changement de version explicite à
chaque projet et alourdissaient le démarrage du shell. Un oubli de bascule
signifie exécuter un projet sur la mauvaise version de Node.

## Décision

Adopter Volta : la version est déclarée dans le `package.json` du projet et
appliquée automatiquement par des shims, sans hook de shell. Le `Makefile`
installe Volta puis la version Node exacte déclarée. Le script d'upgrade résout
la LTS courante dans le pin projet avant d'installer cette même version comme
défaut utilisateur.

## Conséquences

- Plus de bascule manuelle ni de coût au démarrage du shell.
- Les shims ajoutent une indirection qui s'est révélée fragile pour les
  paquets globaux ([ADR-017](017-npm-pour-paquets-globaux.md)), notamment en
  CI.
- L'ordre des cibles `make` doit garantir que Volta précède toute installation
  Node, ce qu'attestent plusieurs correctifs de dépendances.

## Alternatives écartées

- nvm : lent au démarrage, bascule manuelle.
- fnm : plus rapide, mais bascule manuelle également.
- Node installé par Homebrew : une seule version pour tous les projets.
