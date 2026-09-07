# ADR-016 — Volta comme gestionnaire Node

- **Statut** : accepté
- **Date** : 2021-05
- **Révision** : 2026-09-05
- **Commits** : `498aacd`

## Contexte

nvm (2017) puis fnm (2020) exigeaient un changement de version explicite à
chaque projet et alourdissaient le démarrage du shell. Un oubli de bascule
signifie exécuter un projet sur la mauvaise version de Node.

## Décision

Adopter Volta : la version est déclarée dans le `package.json` du projet et
appliquée automatiquement par des shims, sans hook de shell. Moon installe Volta via Homebrew,
puis la version Node exacte déclarée, puis pnpm ; les cibles Make restantes délèguent à ces tâches.
Volta quitte le Brewfile conformément à l'ADR-002. Le script d'upgrade résout
la LTS courante dans le pin projet avant d'installer cette même version comme
défaut utilisateur.

## Conséquences

- Plus de bascule manuelle ni de coût au démarrage du shell.
- Les shims ajoutent une indirection qui s'est révélée fragile pour les
  paquets globaux ([ADR-017](017-npm-pour-paquets-globaux.md)), notamment en
  CI.
- Les dépendances Moon ordonnent Homebrew, Volta, Node et pnpm.
- Le contrôle Node exécute le shim hors du projet pour comparer le défaut utilisateur au pin,
  sans confondre ce défaut avec la sélection automatique de la version du projet.
- Le cache des installations est désactivé ; les contrôles portent sur l'état installé,
  avec un mutex Homebrew pour Volta et un mutex Volta pour Node et pnpm.
- pnpm conserve son contrôle de présence du shim et sa commande d'installation existante,
  sans activer le support expérimental natif de Volta.

## Alternatives écartées

- nvm : lent au démarrage, bascule manuelle.
- fnm : plus rapide, mais bascule manuelle également.
- Node installé par Homebrew : une seule version pour tous les projets.
