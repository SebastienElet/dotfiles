# ADR-002 — Homebrew comme gestionnaire des paquets

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-10-02
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

Les recettes Homebrew unitaires du `Makefile` dispersaient l’inventaire et obligeaient un gate à
parser leur structure. Les profils macOS ont besoin d’une source déclarative directement comprise
par le gestionnaire de paquets.

## Décision

Homebrew reste le gestionnaire des paquets du poste. `Brewfile` porte les formules et casks non
migrés du profil minimal. `Brewfile.optional` porte ceux du profil optionnel, ses taps approuvés et
les applications Mac App Store. Homebrew Bundle installe et vérifie ces deux inventaires.

Une installation migrée vers une tâche Moon autonome appelle directement Homebrew et déclare ses
prérequis dans le graphe Moon. La formule quitte alors le Brewfile : une seule source déclare son
installation. La migration avance par dépendance validée, sans généraliser aux autres paquets.

Rust utilise rustup pour partager une version exacte entre le poste et la CI. Le fichier racine
`rust-toolchain.toml` est la source unique du canal et des composants requis. Moon utilise les
proxies rustup pour ses tâches Cargo ; les commandes Cargo directes utilisent la même sélection
depuis le dépôt. Moon ne synchronise ni ce fichier ni les contraintes MSRV des paquets.

La tâche `repository:rust` amorce rustup sur macOS avec son installateur officiel lorsqu'il est
absent, puis laisse rustup installer la toolchain du dépôt. Elle ne désinstalle pas une ancienne
formule Homebrew ; les proxies de `${CARGO_HOME:-$HOME/.cargo}/bin` passent avant son compilateur
dans les tâches Moon. La configuration Fish existante privilégie `~/.cargo/bin` pour les commandes
interactives. La migration n'installe pas de toolchain par défaut hors du dépôt.

Une source non prise en charge par Bundle est exécutée directement par Moon une fois migrée :
Volta/npm, installateur éditeur, build Rust, téléchargement avec intégrité ou symlink. Les opérations
optionnelles non migrées restent transitoirement dans Make. Ces exceptions sont décrites dans
[`docs/software-source-exceptions.md`](../software-source-exceptions.md) sans gate miroir.

Les contrôles Rust sous Linux nécessitent le rustup du runner et installent la même toolchain depuis
`rust-toolchain.toml` ; cette portée ne transforme pas le profil du poste en installateur Linux.
Les dépendances locales du paquet de développement, dont les outils de contrôle, restent gérées
par Bun avec son lockfile.

## Conséquences

- Un paquet Homebrew est déclaré dans un seul Brewfile ou dans sa tâche Moon, jamais les deux.
- `brew bundle check --quiet --no-upgrade` reste le probe des inventaires Bundle ; les formules
  migrées utilisent les contrôles natifs Homebrew de leur tâche Moon.
- Les mutations Homebrew du graphe Moon partagent un mutex pour sérialiser les installations.
- Les taps tiers sont qualifiés dans le manifeste au niveau de la formule concernée.
- L’inventaire déclaratif n’est ni parsé ni recopié par un test du dépôt.

## Alternatives écartées

- Recettes `brew install` unitaires dans Make : maintiennent un second graphe d'installation.
- Formule déclarée à la fois dans un Brewfile et une tâche Moon : double source de vérité.
- Version Rust suivant Homebrew ou le runner : leurs mises à jour indépendantes peuvent exposer
  des diagnostics Clippy différents en local et en CI.
- Version Rust recopiée dans Moon ou les workflows : introduit plusieurs sources à maintenir.
- Inventaire TypeScript dérivé du Makefile : test miroir sans comportement propre.
- Lockfile Homebrew : Bundle ne fournit pas ce contrat de versions figées.
