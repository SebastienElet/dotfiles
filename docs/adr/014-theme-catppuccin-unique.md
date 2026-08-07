# ADR-014 — Thème Catppuccin unique et bascule automatique

- **Statut** : accepté
- **Date** : 2026-01
- **Commits** : `93db313`, `d7b14c9`, `2919a49`

## Contexte

Chaque outil portait son propre thème — Nord ici, TokyoNight là, thème par
défaut ailleurs — avec des variantes claires incohérentes. En extérieur ou
selon l'heure, la bascule clair/sombre devait être faite outil par outil, et
certains thèmes référencés n'existaient plus en amont.

## Décision

Retenir Catppuccin comme palette unique, déclinée sur WezTerm, bat, git-delta
et Neovim, avec installation automatique des thèmes bat par le `Makefile`. La
bascule suit le mode système via `auto-dark-mode`, l'intervalle de scrutation
étant réduit à cinq secondes.

## Conséquences

- Rendu homogène entre terminal, pager, diff et éditeur.
- Une seule décision de thème à reprendre lors d'un changement de palette.
- Les variantes claires demandent des ajustements de contraste spécifiques,
  visibles dans l'historique.

## Alternatives écartées

- Un thème par outil : incohérence visuelle et bascules manuelles.
- Thème fixe sombre : illisible en forte luminosité.
