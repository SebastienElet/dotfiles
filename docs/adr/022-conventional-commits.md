# ADR-022 — Conventional Commits imposés

- **Statut** : accepté
- **Date** : 2025-12
- **Commits** : `28f9caa`

## Contexte

Les messages de commit antérieurs à 2015 sont des phrases libres (« Add
wallpapers », « Fix tmux install »), impossibles à filtrer par nature de
changement. La convention `type(scope): sujet` s'était installée par usage
sans être écrite, et les agents ne pouvaient donc pas la respecter.

## Décision

Formaliser Conventional Commits comme règle d'agent : `type(scope): sujet` à
l'impératif, corps expliquant le pourquoi quand il n'est pas évident.

## Conséquences

- L'historique devient filtrable par type et par périmètre — c'est ce qui a
  rendu possible la reconstitution de ces ADR.
- La convention est appliquée sans automatisation : ni commitlint, ni hook de
  vérification du message.
- Les commits antérieurs restent non conformes ; la convention ne vaut que
  pour la suite.

## Alternatives écartées

- commitlint en hook : une dépendance Node dans le chemin de chaque commit.
- Aucune convention : historique non exploitable.
