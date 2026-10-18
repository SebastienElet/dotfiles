# ADR-021 — `main` détectée dynamiquement

- **Statut** : accepté
- **Date** : 2023-12
- **Révision** : 2026-10-18, lot [#556](https://github.com/SebastienElet/dotfiles/issues/556)
- **Commits** : `cc9e0db`, `7070bc9`

## Contexte

La branche principale est passée de `master` à `main` dans ce dépôt, mais les
dépôts fréquentés au quotidien n'ont pas basculé au même moment. Coder le nom
en dur dans les fonctions de rebase et de revue casse sur la moitié d'entre
eux.

## Décision

Ce dépôt utilise `main`. L'abbreviation `grbm` passe par
`tooling/git-main-branch`, qui interroge le dépôt courant et retombe sur
`master` ou `trunk` le cas échéant. Le mode local reste une heuristique destinée
aux usages interactifs.

La capacité cible est le paquet Bun autonome `@sebastienelet/git-main-branch`.
Le mode fournisseur est retiré, sans remplacement ni compatibilité 1.x.
L'heuristique locale ne prouve pas la branche distante faisant autorité.
Le mode `--strict` conserve les erreurs de sondage Git, mais le repli `main`
reste possible lorsqu'aucune candidate locale n'existe.

Le chemin local et ses consommateurs Fish/upgrade sont conservés jusqu'à
qualification du paquet publié sur macOS ARM64 et Linux x64 glibc, selon le
[plan #548](https://github.com/SebastienElet/dotfiles/issues/548#issuecomment-6249854102).

## Conséquences

- Les mêmes raccourcis fonctionnent sur les dépôts anciens et récents.
- Une indirection de plus dans chaque fonction concernée.
- La logique de détection est centralisée en un point unique.
- Le résolveur local dépend de Bun et de Git, sans requête réseau ni écriture Git.

## Alternatives écartées

- Nom en dur : casse sur les dépôts restés en `master`.
- Variable de configuration par dépôt : à renseigner manuellement à chaque
  clone.
