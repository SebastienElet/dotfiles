# ADR-015 — CLI modernes en remplacement des historiques

- **Statut** : accepté
- **Date** : 2026-01
- **Commits** : `2ef0caf`, `3ccc657`, `b4fbc26`, `313c417`, `fb45dca`

## Contexte

Les outils Unix historiques et leurs premiers remplaçants (`ag`, `exa`,
`diff-so-fancy`) sont soit lents sur de gros dépôts, soit non maintenus, soit
dépourvus des intégrations attendues (respect du `.gitignore`, couleurs,
navigation).

## Décision

Remplacer chaque outil par son équivalent moderne maintenu : `rg` pour `ag` et
`grep`, `eza` pour `exa` et `ls`, `git-delta` pour `diff-so-fancy`, `fd`,
`fzf`, `zoxide`, `bat`, `bottom`, `broot`, `lazygit`, `lazydocker`. Les
remplacements se font par substitution complète, sans coexistence durable.

## Conséquences

- Recherche et navigation nettement plus rapides sur les dépôts volumineux.
- Les habitudes reposent sur des outils absents d'un serveur distant ; les
  scripts du dépôt s'en tiennent aux outils POSIX.
- Chaque outil est une dépendance de plus à installer et mettre à jour.

## Alternatives écartées

- S'en tenir aux outils POSIX : perte des intégrations git et de la vitesse.
- Conserver les remplaçants historiques : `ag` et `exa` ne sont plus
  maintenus.
