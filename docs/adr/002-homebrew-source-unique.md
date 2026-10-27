# ADR-002 — Homebrew comme gestionnaire des paquets

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-10-27 (décision de Sébastien : rien de flottant sauf Codex et Claude)
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

Les recettes Homebrew unitaires du `Makefile` dispersaient l’inventaire et obligeaient un gate à
parser leur structure. Les profils macOS ont besoin d’une source déclarative directement comprise
par le gestionnaire de paquets.

## Décision

Homebrew reste le gestionnaire des paquets du poste. `Brewfile` porte les formules et casks non
migrés du profil minimal. `Brewfile.optional` porte ceux du profil optionnel, ses taps approuvés et
les applications Mac App Store. Homebrew Bundle installe et vérifie ces deux inventaires.

Aucune source du dépôt ne flotte, sauf Codex et Claude, qui suivent leur dernière version. Homebrew
n'installe que la version courante de son catalogue et Bundle n'offre plus de verrou : le fichier
racine `homebrew.lock.json` porte la version exacte de chaque formule et cask installés par le dépôt.
`repository:homebrew-catalog` refuse toute installation Homebrew quand le catalogue local diffère du
verrou ; l'auto-update Homebrew est désactivé dans le projet ; les tâches Bundle vérifient ensuite
les versions installées. Le verrou refuse une version divergente sans pouvoir réinstaller une
version retirée du catalogue. `tooling/upgrade` le rafraîchit après `brew update` : une montée de
version est un changement explicite du dépôt.

Une installation migrée vers une tâche Moon autonome appelle directement Homebrew et déclare ses
prérequis dans le graphe Moon. La formule quitte alors le Brewfile : une seule source déclare son
installation. La migration avance par dépendance validée, sans généraliser aux autres paquets.

Rust utilise rustup pour partager une version exacte entre le poste et la CI. Le fichier racine
`rust-toolchain.toml` est la source unique du canal et des composants requis. Moon utilise les
proxies rustup pour ses tâches Cargo ; les commandes Cargo directes utilisent la même sélection
depuis le dépôt. Moon ne synchronise ni ce fichier ni les contraintes MSRV des paquets.

La tâche `repository:rust` amorce rustup sur macOS avec `rustup-init` 1.29.1, vérifié par SHA-256,
lorsqu'il est absent, puis laisse rustup installer la toolchain du dépôt. Elle ne désinstalle pas une ancienne
formule Homebrew ; les proxies de `${CARGO_HOME:-$HOME/.cargo}/bin` passent avant son compilateur
dans les tâches Moon. La configuration Fish existante privilégie `~/.cargo/bin` pour les commandes
interactives. La migration n'installe pas de toolchain par défaut hors du dépôt.

Une source non prise en charge par Bundle est exécutée directement par Moon une fois migrée :
Volta/npm, installateur éditeur, build Rust, téléchargement avec intégrité ou symlink.
Leurs sources, versions et contrôles de rejeu restent dans les déclarations Moon et les
lockfiles concernés, sans inventaire documentaire ni gate miroir.

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
- L’inventaire déclaratif n’est ni parsé ni recopié par un test du dépôt. Le verrou recopie les noms
  pour porter les versions ; `brew bundle list` contrôle sa complétude pour les Brewfiles et
  `repository:floating-sources-check` pour les tâches Moon.
- `repository:floating-sources-check` refuse toute source flottante réintroduite dans les fichiers
  suivis, hors exceptions explicites dont Codex et Claude.
- Restent hors verrou : le cask Claude, le cask WezTerm nightly publié en `latest`
  ([ADR-013](013-wezterm-emulateur.md)), les applications Mac App Store, les dépendances transitives
  et l'auto-mise à jour des applications qui la pratiquent.

## Alternatives écartées

- Recettes `brew install` unitaires dans Make : maintiennent un second graphe d'installation.
- Formule déclarée à la fois dans un Brewfile et une tâche Moon : double source de vérité.
- Version Rust suivant Homebrew ou le runner : leurs mises à jour indépendantes peuvent exposer
  des diagnostics Clippy différents en local et en CI.
- Version Rust recopiée dans Moon ou les workflows : introduit plusieurs sources à maintenir.
- Inventaire TypeScript dérivé du Makefile : test miroir sans comportement propre.
- Verrou natif de Bundle : Homebrew ne fournit plus ce contrat de versions figées.
- `brew pin` : propre au poste, non versionné, et ne choisit pas la version installée.
