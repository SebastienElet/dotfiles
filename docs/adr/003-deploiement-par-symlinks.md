# ADR-003 — Déploiement depuis un état propre

- **Statut** : accepté
- **Date** : 2026-08
- **Issue** : [#152](https://github.com/SebastienElet/dotfiles/issues/152)

## Contexte

Le `Makefile` installe les artefacts versionnés sous `home/`, `harness/` et `tooling/`. La
revalidation des destinations avait transformé des recettes d'installation en logique de
diagnostic, de préservation et de migration. Cette responsabilité appartient aux oracles du
harnais, pas à l'installateur.

## Décision

Une cible de déploiement Make part d'un état propre : sa destination possédée est absente. Sa
recette applique directement l'opération d'installation requise, sans inspecter le type ou le
contenu d'un état préexistant et sans branche de migration, de réconciliation ou de réparation.

Make laisse une cible déjà présente à la résolution normale de son graphe ; ce constat ne certifie
ni son contenu ni sa provenance. Les fichiers assemblés suivent le même contrat. `~/.codex/AGENTS.md`
reste assemblé parce que Codex ignore les directives `@import`.

La remise en état explicite consiste à exécuter `make clean`, puis le profil ou la cible
d'installation voulu, par exemple `make minimal`. Le nettoyage supprime les destinations exactes
possédées par le dépôt sans interpréter leur état courant et sans suivre les liens symboliques. Les
oracles dédiés du harnais diagnostiquent la conformité ; les recettes d'installation ne la prouvent
pas.

## Conséquences

- Le chemin supporté est une installation propre ou une reconstruction explicite.
- Un profil peut rester silencieux au second passage parce que Make juge ses cibles présentes, sans
  que ce silence garantisse leur conformité.
- Le comportement d'une recette invoquée directement sur une destination préexistante n'appartient
  pas au contrat.
- Une remise en état peut supprimer une modification locale placée à une destination possédée.

## Alternatives écartées

- Revalider chaque destination pendant l'installation : mélange installation et diagnostic.
- Préserver et classifier chaque état divergent : transforme Make en outil de maintenance.
- Migrer automatiquement une ancienne disposition : pérennise dans l'installateur un état
  historique qui doit disparaître par reconstruction.
