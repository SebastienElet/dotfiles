# ADR-017 — Bun pour les paquets globaux

- **Statut** : accepté
- **Date** : 2026-01
- **Commits** : `8e91200`
- **Révision** : 2026-11-06 (Bun comme cible des installations globales)

## Contexte

Les paquets npm globaux avaient été confiés à pnpm. L'articulation entre le
répertoire global de pnpm (`PNPM_HOME`, `pnpm setup`) et les shims Volta a
demandé une série de correctifs successifs sur l'ordre des cibles et
l'exportation des variables, sans jamais devenir fiable en CI.

## Décision

Adopter Bun comme gestionnaire cible des paquets npm installés globalement par le dépôt.
La migration des installations existantes est progressive : chaque paquet conserve son
installation actuelle jusqu'à sa migration explicite. Volta reste le gestionnaire de Node
conformément à l'ADR-016 ; installer un paquet avec Bun ne signifie pas qu'il s'exécute avec Bun.

Les paquets globaux installés par le dépôt portent une version exacte. `tooling/upgrade` ne met à
jour que Codex, qui suit sa dernière version par décision ; les autres changent par le dépôt.

## Conséquences

- Chaque migration vérifie le runtime requis, la version et le binaire effectivement résolu dans
  le `PATH`, ainsi que le comportement de réinstallation, avant de retirer l'ancien chemin.
- Les paquets non migrés continuent d'utiliser leurs mécanismes existants ; aucun nettoyage global
  n'est implicite.
- Les mutations globales Bun sont sérialisées par le mutex Moon `bun-global`.

## Alternatives écartées

- pnpm en global : source du problème, retiré.
- npm ou Volta pour les nouveaux paquets globaux : maintient plusieurs mécanismes cibles.
- Installation par Homebrew des CLI Node : version découplée du runtime du
  projet.
