# ADR-028 — `.agents/skills/` comme source unique des skills projet

- **Statut** : accepté
- **Date** : 2026-05
- **Commits** : `170c966` (#42), `293c6e1`, `3133e06`

## Contexte

Les procédures propres au dépôt — conventions, Neovim et scripts — existaient
sous forme de règles Cursor, donc liées à un agent et invisibles des autres.
Leur qualité variait, et une skill mal décrite n'est jamais déclenchée. Les
skills personnelles partagées avec les autres dépôts relèvent de l'[ADR-040](040-skills-user-dans-harness.md).

## Décision

`.agents/skills/` est la source unique des skills dont la portée est ce dépôt,
chaque skill étant un répertoire portant son `SKILL.md`. Ce chemin reste à la
racine parce que la découverte projet l'impose. `skill-manager` encadre la
création, la validation et la synchronisation de l'index `README.md` depuis sa
source user définie par l'ADR-040.

## Conséquences

- Une skill projet écrite une fois est utilisable par tous les agents dans ce dépôt.
- Une skill installée au niveau user ne vit jamais dans cette collection.
- La contrainte de description force à énoncer le déclencheur plutôt que le
  contenu.
- Toute modification, fût-ce un champ de frontmatter, passe par
  `skill-manager`.

## Alternatives écartées

- Règles Cursor : liées à un agent, non réutilisables.
- Instructions inline dans `AGENTS.md` : fichier illisible et chargé en
  totalité à chaque session.
