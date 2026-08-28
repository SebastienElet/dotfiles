# ADR-002 — Brewfiles comme source unique des paquets

- **Statut** : accepté
- **Date** : 2026-08
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

Les recettes Homebrew unitaires du `Makefile` dispersaient l’inventaire et obligeaient un gate à
parser leur structure. Les profils macOS ont besoin d’une source déclarative directement comprise
par le gestionnaire de paquets.

## Décision

`Brewfile` porte les formules et casks du profil minimal. `Brewfile.optional` porte ceux du profil
optionnel, ses taps approuvés et les applications Mac App Store. Homebrew Bundle installe et vérifie
ces deux inventaires.

Une source non prise en charge par Bundle reste dans le `Makefile` seulement lorsqu’elle possède
une logique distincte : Volta/npm, installateur éditeur, build Rust, image Docker, téléchargement
avec intégrité ou symlink. Ces exceptions sont décrites dans
[`docs/software-source-exceptions.md`](../software-source-exceptions.md) sans gate miroir.

## Conséquences

- Un paquet Homebrew apparaît dans un seul Brewfile.
- `brew bundle check --quiet --no-upgrade` est le probe de convergence.
- Les taps tiers sont qualifiés dans le manifeste au niveau de la formule concernée.
- L’inventaire déclaratif n’est ni parsé ni recopié par un test du dépôt.

## Alternatives écartées

- Recettes `brew install` unitaires dans Make : double source de vérité.
- Inventaire TypeScript dérivé du Makefile : test miroir sans comportement propre.
- Lockfile Homebrew : Bundle ne fournit pas ce contrat de versions figées.
