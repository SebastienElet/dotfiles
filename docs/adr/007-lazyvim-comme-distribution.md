# ADR-007 — LazyVim comme distribution Neovim

- **Statut** : accepté
- **Date** : 2023-12
- **Commits** : `c7ad20e` (#29), `2bfac4b` (#30)

## Contexte

La configuration Neovim maison, puis NvChad et NvChad 2, imposaient de suivre
manuellement l'évolution de chaque plugin et les ruptures de la distribution.
Le coût de maintenance dépassait la valeur d'une configuration sur mesure.

## Décision

Adopter LazyVim : les réglages par défaut et les « extras » de la distribution
font foi, la configuration locale se limite aux surcharges. La reconstruction à
neuf de 2024-06 (#30) a supprimé les vestiges de la configuration précédente
plutôt que de les porter.

## Conséquences

- Les mises à jour de plugins deviennent une opération de routine
  ([ADR-004](004-script-upgrade-unique.md)).
- La surcharge doit rester minimale : plusieurs commits retirent des réglages
  redondants avec les extras LazyVim.
- Dépendance au rythme et aux choix d'un projet tiers.

## Alternatives écartées

- NvChad et NvChad 2 : remplacés, ruptures de version coûteuses.
- Configuration entièrement maison : le point de départ, abandonné pour son
  coût de maintenance.
