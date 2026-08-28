# ADR-004 — Script `upgrade` unique pour toutes les mises à jour

- **Statut** : accepté
- **Date** : 2017-09
- **Commits** : `2e9c796`, `6c70ca1`

## Contexte

Les mises à jour se répartissent entre plusieurs gestionnaires : Homebrew
(formules et casks), paquets npm globaux, plugins Neovim, plugins d'agents,
runtime Node. Les lancer séparément conduit à en oublier.

## Décision

`tooling/upgrade` orchestre l'ensemble en une commande : `git pull` du dépôt,
redéploiement du socle par `make minimal`, `brew upgrade` (avec `--greedy` pour les casks),
mise à jour des paquets npm globaux, des plugins Neovim — dont le lockfile
résultant est committé —, des plugins d'agents et du Node LTS géré par Volta.

## Conséquences

- Une seule habitude à tenir ; le poste ne dérive pas.
- Les mises à jour sont groupées, donc une régression est plus difficile à
  imputer à un composant précis.
- Le script doit tolérer l'absence d'un outil : les échecs partiels ont donné
  lieu à plusieurs correctifs de robustesse.

## Alternatives écartées

- Mise à jour automatique planifiée : perte de contrôle sur le moment où
  l'environnement de travail change.
