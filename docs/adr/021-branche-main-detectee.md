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
`tooling/git-main-branch`, source unique de détection. Son mode générique
conserve la détection locale de `main`, `master` puis `trunk` pour les usages
interactifs existants. Ce mode ne constitue pas une preuve de la branche
canonique d'un fournisseur.

Le mode `--bitbucket-cloud` établit en plus l'identité du dépôt depuis les URL
de fetch Bitbucket Cloud, interroge le `HEAD` publié par chaque remote et exige
leur convergence. Il interroge explicitement chaque identité auprès de l'API,
valide `uuid`, `full_name` et `mainbranch.name`, fusionne les remotes de même
UUID, puis exige l'accord entre le `HEAD` Git distant et `mainbranch.name`. Les
références locales et les branches de suivi ne font pas autorité dans ce mode :
elles peuvent être absentes ou périmées. Le dépôt du contexte `bkt` actif
n'est jamais utilisé comme portée implicite.

Le mode `--strict` échoue lorsqu'aucune branche ne peut être établie. Le mode
Bitbucket est toujours strict : dépôt, remotes, identité et branche principale
absents, invalides, ambigus ou discordants sont des erreurs. Hors de ces modes
stricts, `main` reste le repli historique interactif.

## Conséquences

- Les mêmes raccourcis fonctionnent sur les dépôts anciens et récents.
- Une indirection de plus dans chaque fonction concernée.
- La logique de détection est centralisée en un point unique.
- La détection Bitbucket dépend du réseau, de `bkt`, de `jq` et d'une
  authentification Bitbucket Cloud valide.
- Une configuration à plusieurs remotes n'est acceptée que si leurs branches
  principales convergent ; en mode Bitbucket, leurs UUID doivent aussi
  converger.

## Alternatives écartées

- Nom en dur : casse sur les dépôts restés en `master`.
- Variable de configuration par dépôt : à renseigner manuellement à chaque
  clone.
- Première branche locale parmi `main`, `master` et `trunk` comme autorité du
  fournisseur : une branche plausible peut masquer sa valeur canonique. Cette
  heuristique reste limitée au mode générique interactif.
- `refs/remotes/<remote>/HEAD` : cache local facultatif, donc potentiellement
  absent ou périmé.
