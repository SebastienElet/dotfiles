# ADR-030 — `~/Brain` comme mémoire durable partagée

- **Statut** : accepté
- **Date** : 2026-07
- **Commits** : `ff5b87d` (#45)

## Contexte

Chaque agent propose sa propre mémoire persistante. Une information consignée
dans la mémoire de Claude est invisible de Codex et de Cursor, et disparaît
avec l'agent. Les faits, décisions et références réutilisables méritent un
support indépendant de l'outil.

## Décision

`~/Brain`, hébergé sur iCloud Drive (`BRAIN_PATH`) et symlinké par le
`Makefile`, est la cible par défaut de la mémoire durable, avec ses propres
instructions (`~/Brain/AGENTS.md`) que tout agent doit lire avant d'y écrire.
La mémoire propre à un agent reste réservée à ce qui concerne son
comportement.

## Conséquences

- Un fait consigné une fois est disponible pour tous les agents, et survit au
  changement d'outil.
- La règle d'aiguillage doit être rappelée dans `AGENTS.md`, les agents
  privilégiant spontanément leur mémoire native.
- Dépendance à la synchronisation iCloud.

## Alternatives écartées

- Mémoire native de chaque agent : cloisonnée et volatile.
- Dépôt git dédié : synchronisation manuelle, moins immédiate qu'iCloud.
