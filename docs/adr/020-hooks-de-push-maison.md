# ADR-020 — Hooks de push maison dans `scripts/`

- **Statut** : accepté
- **Date** : 2022-11
- **Commits** : `a99248c`, `c3ee8ca`

## Contexte

Certaines erreurs ne se découvrent qu'en revue ou en CI, alors qu'elles sont
détectables localement en quelques secondes : fautes de frappe, `TODO` oubliés,
fichiers vides commités, erreurs ESLint, copier-coller massif.

## Décision

Des scripts autonomes dans `scripts/` (`git_hook_assert_typos`,
`git_hook_assert_todoes`, `git_hook_assert_empty_files`,
`git_hook_assert_eslint`, `git_hook_detect_copy_paste`) sont enchaînés par
`git_hook_push` et exécutés au push, pas au commit.

## Conséquences

- Le commit reste rapide ; la vérification a lieu au moment où le travail
  devient visible des autres.
- Chaque script est testable et exécutable seul.
- Un push peut échouer pour une raison stylistique, contournable par
  `--no-verify` en cas d'urgence.

## Alternatives écartées

- Hooks de pre-commit : ralentissent chaque commit, y compris les commits
  intermédiaires.
- Framework de hooks (husky, pre-commit) : dépendance supplémentaire pour ce
  que cinq scripts shell couvrent.
