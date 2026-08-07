# ADR-027 — `SOUL.md` et `USER.md` séparés de `AGENTS.md`

- **Statut** : accepté
- **Date** : 2026-08
- **Commits** : `3005717` (#47)

## Contexte

`AGENTS.md` mélangeait trois natures d'information : les règles du dépôt, la
voix attendue de l'agent et les préférences de travail de l'utilisateur. Le
mélange produisait des contradictions — le commit `3005717` relève une règle
« ASK rather than assume » incompatible avec le principe « assume and
declare » — et empêchait de charger la voix hors du dépôt.

## Décision

Trois fichiers de portées distinctes : `ai/SOUL.md` pour l'identité (langue,
registre, priorités), `ai/USER.md` pour les biais et attentes de travail,
`AGENTS.md` pour les règles du dépôt. `SOUL.md` et `USER.md` sont importés dans
les instructions globales et valent donc pour tous les projets.

## Conséquences

- Voix et préférences s'appliquent hors de ce dépôt.
- Une contradiction se tranche par la portée du fichier concerné.
- Trois fichiers à tenir cohérents, avec un risque de recouvrement à
  surveiller.

## Alternatives écartées

- Tout garder dans `AGENTS.md` : contradictions et portée limitée au dépôt.
- Configuration propre à chaque agent : contraire à
  [ADR-025](025-agents-md-source-unique.md).
