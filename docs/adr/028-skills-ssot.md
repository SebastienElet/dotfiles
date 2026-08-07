# ADR-028 — `.agents/skills/` comme source unique des skills

- **Statut** : accepté
- **Date** : 2026-05
- **Commits** : `170c966` (#42), `293c6e1`, `3133e06`

## Contexte

Les procédures réutilisables — conventions du dépôt, Neovim, scripts, Johnny
Decimal — existaient sous forme de règles Cursor, donc liées à un agent et
invisibles des autres. Leur qualité variait, et une skill mal décrite n'est
jamais déclenchée.

## Décision

`.agents/skills/` est la source unique des skills, chaque skill étant un
répertoire portant son `SKILL.md`, distribué aux agents par symlink.
`skill-manager` encadre la création, la validation et la synchronisation de
l'index `README.md`. Le budget de description a été réduit de moitié
(`3133e06`), la description conditionnant le déclenchement.

## Conséquences

- Une skill écrite une fois est utilisable par tous les agents.
- La contrainte de description force à énoncer le déclencheur plutôt que le
  contenu.
- Toute modification, fût-ce un champ de frontmatter, passe par
  `skill-manager`.

## Alternatives écartées

- Règles Cursor : liées à un agent, non réutilisables.
- Instructions inline dans `AGENTS.md` : fichier illisible et chargé en
  totalité à chaque session.
