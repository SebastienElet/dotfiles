# ADR-021 — `main` détectée dynamiquement

- **Statut** : accepté
- **Date** : 2023-12
- **Commits** : `cc9e0db`, `7070bc9`

## Contexte

La branche principale est passée de `master` à `main` dans ce dépôt, mais les
dépôts fréquentés au quotidien n'ont pas basculé au même moment. Coder le nom
en dur dans les fonctions de rebase et de revue casse sur la moitié d'entre
eux.

## Décision

Ce dépôt utilise `main`. L'abbreviation `grbm` passe par
`tooling/git-main-branch`, qui interroge le dépôt courant et retombe sur
`master` le cas échéant.

## Conséquences

- Les mêmes raccourcis fonctionnent sur les dépôts anciens et récents.
- Une indirection de plus dans chaque fonction concernée.
- La logique de détection est centralisée en un point unique.

## Alternatives écartées

- Nom en dur : casse sur les dépôts restés en `master`.
- Variable de configuration par dépôt : à renseigner manuellement à chaque
  clone.
