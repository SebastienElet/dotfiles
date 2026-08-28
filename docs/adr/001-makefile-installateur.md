# ADR-001 — Profils hybrides comme installateur de poste

- **Statut** : accepté
- **Date** : 2026-08
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

La cible historique `all` mélangeait socle de développement, applications optionnelles, outils
payants et composants dépendants de Docker. La CI devait reconstruire sa portée en analysant le
`Makefile`, et l’inventaire des paquets était dispersé entre des recettes impératives.

## Décision

Le `Makefile` reste l’orchestrateur public avec deux profils :

- `minimal` installe le poste de développement de référence ;
- `optional` converge d’abord `minimal`, puis installe les composants utilisés hors de ce socle.

`Brewfile` et `Brewfile.optional` sont les sources canoniques des formules, casks, taps et
applications Mac App Store. Le `Makefile` conserve l’amorçage Homebrew, les installations
non-Homebrew et les artefacts possédés par le dépôt. `install.sh` et `tooling/upgrade` appellent
`make minimal` ; aucun alias `all` n’est conservé.

Les profils ferment leur entrée standard après l’amorçage. Ils exécutent
`brew bundle check --quiet --no-upgrade` avant toute installation, afin qu’un passage convergé
reste silencieux sans demander de mise à niveau globale.

## Conséquences

- Le socle et les optionnels sont lisibles dans deux manifestes déclaratifs.
- Les installations spécifiques restent locales au processus ou à l’artefact qu’elles possèdent.
- Seul le profil minimal reçoit une garantie end-to-end en CI.
- L’ajout d’un composant Homebrew modifie le Brewfile de son profil, pas une recette Make.

## Alternatives écartées

- Conserver `all` : sa portée ambiguë est précisément le défaut corrigé.
- Un script shell impératif : il dupliquerait l’orchestration et les probes de Make et Bundle.
- Ansible ou un gestionnaire de dotfiles : disproportionné pour un poste unique.
