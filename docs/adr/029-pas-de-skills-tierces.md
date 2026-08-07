# ADR-029 — Skills tierces refusées par défaut

- **Statut** : accepté
- **Date** : 2026-05
- **Commits** : `e43f299`, `265f17e`, `39530c3`, `1ecef61`

## Contexte

Début 2026, une vingtaine de cibles installaient des skills tierces
(typescript, supabase, shadcn, next, turborepo, prisma, bash-pro, caveman,
rtk…). Chacune occupe du contexte à chaque session, se périme au rythme d'un
tiers et recouvre partiellement les autres.

## Décision

N'installer par défaut que les skills maintenues dans ce dépôt. Les paquets de
skills tierces ont été retirés par vagues successives, jusqu'à caveman et rtk
en 2026-08. Une skill tierce doit justifier sa place, comme toute dépendance.

## Conséquences

- Contexte d'agent plus léger et déclenchements plus prévisibles.
- Le savoir-faire d'un domaine doit être réécrit localement quand il est
  nécessaire.
- Décision cohérente avec la vigilance sur les dépendances inscrite dans
  `USER.md`.
- Les deux retraits les mieux documentés font l'objet d'une ADR propre :
  [ADR-033](033-pas-de-rtk.md) et [ADR-034](034-pas-de-caveman.md).

## Alternatives écartées

- Installer largement et laisser l'agent trier : coût de contexte permanent,
  déclenchements parasites.
