# ADR-017 — npm pour les paquets globaux

- **Statut** : accepté
- **Date** : 2026-01
- **Commits** : `8e91200`

## Contexte

Les paquets npm globaux avaient été confiés à pnpm. L'articulation entre le
répertoire global de pnpm (`PNPM_HOME`, `pnpm setup`) et les shims Volta a
demandé une série de correctifs successifs sur l'ordre des cibles et
l'exportation des variables, sans jamais devenir fiable en CI.

## Décision

Revenir à npm pour les installations globales. Le commit `8e91200` l'explique :
« Replace pnpm with npm for global package installation to avoid Volta shim
issues in CI. npm is bundled with Node and works seamlessly. » pnpm reste
installé pour le développement de projets locaux.

## Conséquences

- Aucun répertoire global ni variable d'environnement supplémentaire à
  configurer.
- Installations globales plus lentes et plus volumineuses qu'avec pnpm.
- La CI installe la chaîne Node sans étape de contournement.

## Alternatives écartées

- pnpm en global : source du problème, retiré.
- Installation par Homebrew des CLI Node : version découplée du runtime du
  projet.
