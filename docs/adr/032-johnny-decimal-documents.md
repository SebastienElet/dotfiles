# ADR-032 — Johnny Decimal pour `~/Documents`

- **Statut** : accepté
- **Date** : 2025-08
- **Commits** : `135f67c`, `83b0f78`

## Contexte

Un classement libre de documents personnels dérive : catégories qui se
recouvrent, mêmes documents rangés à deux endroits, noms de fichiers
hétérogènes qui rendent la recherche inopérante.

## Décision

Appliquer Johnny Decimal, hybridé avec PARA, à `~/Documents` : catégories
numérotées, un préfixe par document. Le script `tooling/jdl` vérifie la
structure et renomme les fichiers non conformes avec `--fix`. La skill
`johnny-decimal` porte la convention pour les agents ; `para-organizer`
couvre les arborescences hors `~/Documents`.

## Conséquences

- Emplacement d'un document déductible de son numéro, sans recherche.
- La convention est vérifiable automatiquement plutôt que tenue de mémoire.
- Toute nouvelle catégorie exige une décision de numérotation explicite.

## Alternatives écartées

- Classement libre par dossiers thématiques : dérive constatée.
- PARA seul : actionnable, mais sans identifiant stable par document.
