# ADR-006 — Neovim comme éditeur par défaut

- **Statut** : accepté
- **Date** : 2019-11
- **Commits** : `d6c33a5`

## Contexte

Vim était l'éditeur depuis 2014. Neovim, essayé dès 2016, apportait un serveur
LSP intégré, une API Lua et un modèle asynchrone que Vim 8 ne couvrait
qu'imparfaitement, notamment pour le formatage et le diagnostic à la volée.

## Décision

Neovim devient l'éditeur par défaut : `EDITOR` et `VISUAL` pointent sur `nvim`,
l'ancien `vimrc` est retiré et la configuration migre progressivement vers Lua.

## Conséquences

- Accès aux distributions et plugins de l'écosystème Lua
  ([ADR-007](007-lazyvim-comme-distribution.md)).
- Toute la configuration éditeur devient du Lua, donc lintable en CI
  ([ADR-023](023-ci-lint-et-installation.md)).
- Dépendance à un écosystème en évolution rapide, d'où le verrouillage des
  versions ([ADR-008](008-lockfile-plugins-versionne.md)).

## Alternatives écartées

- Rester sur Vim 8 : asynchronisme et LSP moins aboutis.
- Emacs : essayé en 2017, abandonné la même année.
- Un IDE graphique : conserverait deux environnements d'édition à maintenir.
