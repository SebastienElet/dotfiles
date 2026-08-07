# ADR-025 — `AGENTS.md` comme source unique agent-agnostique

- **Statut** : accepté
- **Date** : 2026-02
- **Commits** : `170c966` (#42), `0fb9e89`

## Contexte

Les règles se dupliquaient entre `AGENTS.md`, les règles Cursor et les
instructions Claude. `AGENTS.md` décrivait en outre ses propres adaptateurs :
le commit `0fb9e89` relève « a self-referential CLAUDE.md bullet and a list of
skills symlinks already visible in the tree. Both named specific agents in a
file every agent reads. »

## Décision

`AGENTS.md` est la source unique des règles du dépôt et ne nomme aucun agent.
Les fichiers spécifiques à un agent se réduisent à une redirection explicite —
`CLAUDE.md` contient un import `@AGENTS.md` plutôt qu'un symlink — et la règle
de conflit est écrite : en cas de désaccord, `AGENTS.md` l'emporte.

## Conséquences

- Une règle s'écrit à un seul endroit et vaut pour tous les agents.
- Un agent qui ne suit pas les imports demande un traitement particulier
  ([ADR-003](003-deploiement-par-symlinks.md)).
- La redirection est visible dans le fichier plutôt que cachée dans le système
  de fichiers.

## Alternatives écartées

- Un fichier de règles par agent : duplication et divergence.
- Symlink `CLAUDE.md → AGENTS.md` : redirection invisible à la lecture.
