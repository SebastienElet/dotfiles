# ADR-003 — Déploiement de la configuration par symlinks

- **Statut** : accepté
- **Date** : 2014-10
- **Commits** : `9160770`, `0fb9e89`, `1772db9`

## Contexte

Les fichiers de configuration doivent rester versionnés dans `~/.dotfiles`
tout en étant lus depuis leur emplacement attendu par chaque outil. Copier les
fichiers imposerait une resynchronisation après chaque modification.

## Décision

Le `Makefile` crée des symlinks de `$(DOTFILES_PATH)/…` vers `~/…` (`~/.config`,
`~/.gitconfig.delta`, `~/.wezterm.lua`, `~/.psqlrc`, instructions d'agents…).
L'édition se fait dans le dépôt, l'effet est immédiat.

Une exception documentée : `~/.codex/AGENTS.md` est **assemblé** par
concaténation à l'installation. Le commit `1772db9` en donne la raison —
« Codex loads AGENTS.md but ignores its @import directives, verified with a
control prompt » —, la concaténation étant alors le seul mécanisme disponible.

## Conséquences

- Aucune étape de synchronisation ; un `git pull` suffit à propager un
  changement.
- Un fichier assemblé se périme silencieusement : il faut relancer la cible
  après modification des sources.
- La cible doit rester idempotente et ne pas se reconstruire à vide, condition
  vérifiée à plusieurs reprises dans l'historique.

## Alternatives écartées

- `stow` ou `chezmoi` : une dépendance de plus pour ce que quelques règles
  `make` couvrent déjà.
- Copie des fichiers à l'installation : divergence garantie entre dépôt et
  poste.
