# ADR-001 — Moon comme orchestrateur du poste

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-09-07
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

Le profil minimal dépendait de deux graphes : Moon installait les premières dépendances,
puis Make complétait les configurations et le harnais. Les workflows préparaient aussi
certains outils et exécutaient leurs contrôles directement.

## Décision

Moon est l'orchestrateur cible unique pour l'installation et les tâches de développement.
Le point d'entrée `moon exec --quiet install` porte le profil minimal complet ; `check`
et `test` agrègent les contrôles statiques et comportementaux.
Une tâche migrée appelle directement sa commande, jamais une cible Make.

Le fichier racine porte les agrégats publics. Les définitions sont regroupées dans
`.moon/tasks/` avec un héritage ciblé ; `home`, `harness` et `tooling` portent les
responsabilités existantes, et les outils Rust gardent leurs projets. Les contrôles transverses
Lua, Fish et TypeScript restent accessibles depuis le projet racine par défaut.
Aucun répertoire projet n'est créé uniquement pour donner un préfixe à une commande.

Les installations utilisent des contrôles d'état et désactivent le cache d'artefacts pour
les mutations du poste. Les builds conservant des sorties réutilisables gardent leur politique
propre. Les dépendances de paquets JavaScript sont préparées par la toolchain Bun native ;
les tâches d'un autre projet qui consomment le paquet racine en dépendent explicitement.

`Brewfile` et `Brewfile.optional` restent les sources des paquets non migrés. Une formule
migrée vers une tâche autonome quitte son Brewfile selon l'ADR-002. Installer un paquet
n'implique pas déployer sa configuration.

## Transition

Make conserve provisoirement les opérations optionnelles et le nettoyage existants, ainsi
que les adaptateurs qui délèguent aux tâches migrées. Le profil optionnel converge d'abord
le minimal Moon ; il ne réimplémente pas son installation.

`install.sh` vérifie les prérequis macOS et amorce Moon avant le profil minimal.
L'installation de Moon lui-même reste hors de son graphe. `tooling/upgrade` appelle
le même profil Moon après la mise à jour du dépôt.

L'installation de Semctx reste explicite via `harness:semctx`. La préparation de son paquet
ne suffit pas à intégrer automatiquement ses plugins et leurs prérequis hôtes au minimal.

## Conséquences

- Le minimal possède un point d'entrée Moon, partagé avec le smoke macOS de l'ADR-023.
- Les contrôles de développement déclarent leurs prérequis sans installer tout le poste.
- Les opérations optionnelles restent une étape distincte de la suppression finale de Make.
- L'état observé et le silence au rejeu sont évalués par les oracles exécutés, pas par la seule
  présence des tâches ou des dépendances dans le graphe.

## Alternatives écartées

- Conserver Make derrière les tâches migrées : maintient deux graphes d'installation.
- Créer un projet par commande : ajoute des répertoires sans responsabilité propre.
- Un script shell installateur central : duplique le graphe Moon.
