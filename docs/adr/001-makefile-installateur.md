# ADR-001 — Moon comme orchestrateur du poste

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-09-03
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

La cible historique `all` mélangeait socle de développement, applications optionnelles, outils
payants et composants dépendants de Docker. La CI devait reconstruire sa portée en analysant le
`Makefile`, et l’inventaire des paquets était dispersé entre des recettes impératives.

## Décision

Moon est l'orchestrateur cible unique pour l'installation et les tâches de développement.
La migration avance par dépendances validées, jusqu'à la suppression du `Makefile`.
Une tâche migrée appelle directement sa commande d'installation, jamais une cible Make.
Les tâches d'installation simples résident dans le `moon.yml` racine ; aucun répertoire projet
n'est créé uniquement pour donner un préfixe à une commande.

La première étape porte `homebrew` et `applications-install`, cette dernière dépendant de
la première. Les `checks` natifs de Moon vérifient l'état installé avant les commandes ; le cache
de tâches est désactivé, car l'état Homebrew n'est pas un artefact de build du dépôt.

`Brewfile` et `Brewfile.optional` restent les sources canoniques des paquets non migrés. Une formule
migrée vers une tâche Moon autonome quitte son Brewfile selon l'ADR-002. Installer les paquets
n'implique pas déployer leurs configurations.

## Transition

Les chemins non encore migrés conservent provisoirement les profils Make historiques :

- `minimal` installe le poste de développement de référence ;
- `optional` converge d’abord `minimal`, puis installe les composants utilisés hors de ce socle.

`install.sh` amorce Moon avant d'appeler `make minimal`. Le profil Make délègue les opérations
migrées à Moon et conserve les artefacts non encore migrés. L'installation de Moon lui-même reste
hors de son graphe, car elle doit fonctionner avant que son exécutable soit disponible.
`tooling/upgrade` utilise encore ce profil de transition.

Les profils ferment leur entrée standard après l’amorçage. Ils exécutent
`brew bundle check --quiet --no-upgrade` avant toute installation, afin qu’un passage convergé
reste silencieux sans demander de mise à niveau globale.

`harness:install` agrège uniquement les capacités du harnais appartenant au profil minimal.
L'installation de Semctx reste une action explicite via `harness:semctx` et n'est pas une dépendance
de `repository:install` tant que ses prérequis hôtes ne font pas partie du graphe.

## Conséquences

- Le socle et les optionnels sont lisibles dans deux manifestes déclaratifs.
- Les installations spécifiques restent locales au processus ou à l’artefact qu’elles possèdent.
- Le smoke test macOS existant passe par Moon pour les opérations migrées, puis par les recettes
  Make restantes ; il vérifie l'état installé et le second passage du profil de transition.
- L'ajout d'un composant Homebrew modifie le Brewfile de son profil, sauf décision explicite de
  migrer son installation vers une tâche Moon autonome.

## Alternatives écartées

- Conserver `all` : sa portée ambiguë est précisément le défaut corrigé.
- Conserver Make derrière Moon : maintient deux graphes pour les mêmes tâches.
- Créer un projet Moon par commande : ajoute des répertoires sans responsabilité propre.
- Un script shell orchestrateur : dupliquerait le graphe de Moon.
- Ansible ou un gestionnaire de dotfiles : disproportionné pour un poste unique.
