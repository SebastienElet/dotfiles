# ADR-024 — Instructions IA versionnées dans le dépôt

- **Statut** : accepté
- **Date** : 2026-01
- **Commits** : `242b577`

## Contexte

Chaque agent stocke ses instructions à son emplacement propre
(`~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md`, règles Cursor). Ces fichiers
répétaient les mêmes attentes, divergeaient à la première modification et
n'étaient pas versionnés.

## Décision

Les instructions vivent dans le dépôt, sous `harness/`, et sont distribuées à
chaque agent depuis là — par symlink, ou par assemblage quand l'agent ne suit
pas les imports ([ADR-003](003-deploiement-par-symlinks.md)).

## Conséquences

- Les instructions sont versionnées, relues et révocables comme du code.
- Une modification profite à tous les agents simultanément.
- Chaque nouvel agent ajoute une cible de distribution au `Makefile`.

## Alternatives écartées

- Instructions par agent : divergence garantie.
- Instructions par projet uniquement : ne couvre pas les attentes transverses.
