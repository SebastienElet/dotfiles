# ADR-002 — Homebrew comme gestionnaire des paquets

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-09-07
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

Rust inaugure cette exception pour permettre la compilation d'Arnes sans installer tout le profil
minimal. Homebrew fournit Rust et Cargo ; aucune toolchain Rust distincte n'est téléchargée par
Moon. La présence de la formule est vérifiée par Homebrew avant installation ; la version suit
Homebrew et ses mises à jour explicites.

Une source non prise en charge par Bundle est exécutée directement par Moon une fois migrée :
Volta/npm, installateur éditeur, build Rust, téléchargement avec intégrité ou symlink. Les opérations
optionnelles non migrées restent transitoirement dans Make. Ces exceptions sont décrites dans
[`docs/software-source-exceptions.md`](../software-source-exceptions.md) sans gate miroir.

Les contrôles Rust sous Linux utilisent le runtime du runner, vérifié explicitement ; cette portée
ne transforme pas le profil du poste en installateur Linux. Les dépendances locales du paquet de
développement, dont les outils de contrôle, restent gérées par Bun avec son lockfile.

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
- Installation de Rust par `rustup` ou une toolchain téléchargée par Moon : change le gestionnaire
  de paquets sans besoin établi de plusieurs versions Rust.
- Inventaire TypeScript dérivé du Makefile : test miroir sans comportement propre.
- Lockfile Homebrew : Bundle ne fournit pas ce contrat de versions figées.
