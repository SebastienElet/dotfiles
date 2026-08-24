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
`master` ou `trunk` le cas échéant. Le mode local reste une heuristique destinée
aux usages interactifs.

Le mode Bitbucket Cloud établit l'identité du dépôt depuis les URL Git de fetch,
interroge le `HEAD` publié par les remotes et la réponse du fournisseur, puis
exige leur accord. Les références distantes locales ne font pas autorité dans
ce mode. Une preuve absente, invalide, ambiguë, indisponible ou discordante est
une erreur plutôt qu'un repli sur un nom plausible.

## Conséquences

- Les mêmes raccourcis fonctionnent sur les dépôts anciens et récents.
- Une indirection de plus dans chaque fonction concernée.
- La logique de détection est centralisée en un point unique.
- La résolution Bitbucket Cloud dépend explicitement de `git`, de `bkt`, du
  réseau et d'un unique contexte Bitbucket Cloud configuré.

## Alternatives écartées

- Nom en dur : casse sur les dépôts restés en `master`.
- Variable de configuration par dépôt : à renseigner manuellement à chaque
  clone.
