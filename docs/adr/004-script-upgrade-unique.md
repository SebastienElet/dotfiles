# ADR-004 — Script `upgrade` unique pour toutes les mises à jour

- **Statut** : accepté
- **Date** : 2017-09
- **Révision** : 2026-09-10
- **Issue** : [#198](https://github.com/SebastienElet/dotfiles/issues/198)
- **Commits** : `2e9c796`, `6c70ca1`

## Contexte

Les mises à jour se répartissent entre plusieurs gestionnaires : Homebrew
(formules et casks), paquets npm globaux, plugins Neovim, plugins d'agents,
runtime Node. Les lancer séparément conduit à en oublier.

## Décision

`tooling/upgrade` orchestre l'ensemble en une commande : `git pull` du dépôt,
redéploiement du socle par `moon exec --quiet repository:install` selon l’ADR-001,
`brew upgrade` (avec `--greedy` pour les casks),
mise à jour des paquets npm globaux, des plugins Neovim — dont le lockfile
modifié n’est committé qu’après un sync contrôlé sur la branche principale —,
des plugins d'agents et du Node LTS géré par Volta.

Le point d’entrée reste unique ; sa politique est portée par Bun/TypeScript selon l’ADR-041.
Le redéploiement appelle directement Moon, sans passer par l’adaptateur transitoire
`make minimal`. Cette révision aligne la prescription historique sur l’ADR-001 révisé
le 7 septembre 2026.

## Conséquences

- Une seule commande regroupe les tentatives de mise à jour ; les opérations ignorées ou
  échouées restent visibles et peuvent laisser le poste partiellement mis à jour.
- Les mises à jour sont groupées, donc une régression est plus difficile à
  imputer à un composant précis.
- Le script doit tolérer l'absence d'un outil : les échecs partiels ont donné
  lieu à plusieurs correctifs de robustesse.

## Alternatives écartées

- Mise à jour automatique planifiée : perte de contrôle sur le moment où
  l'environnement de travail change.
