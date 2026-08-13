# ADR-012 — Abbreviations Fish plutôt qu'alias git

- **Statut** : accepté
- **Date** : 2026-01
- **Commits** : `8349aa4` (#40)

## Contexte

Les raccourcis git étaient des alias : l'historique du shell ne conservait que
`gcb`, jamais la commande réellement exécutée, ce qui complique le diagnostic
et la relecture d'une session.

## Décision

Convertir les raccourcis git en abbreviations Fish, regroupées dans
`git-abbreviations.fish`. Le commit `8349aa4` en donne la raison :
« Abbreviations expand only when typed, showing full command in history […]
This improves debugging and follows Fish shell conventions ». Tous les
raccourcis statiques, dont `gp`, sont des abbreviations. Seul `gpsup`, dont la
commande dépend de la branche courante, reste un alias.

## Conséquences

- L'historique contient les commandes complètes, réutilisables hors contexte.
- Le raccourci est visible au moment de la frappe, donc vérifiable avant
  exécution.
- Mécanisme propre à Fish : ces raccourcis ne sont pas portables vers un autre
  shell.

## Alternatives écartées

- Alias git côté `~/.gitconfig` : ne couvre pas les raccourcis qui ne sont pas
  des sous-commandes git.
