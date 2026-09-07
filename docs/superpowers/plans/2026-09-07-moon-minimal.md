# Migration Moon du profil minimal

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Exécuter tout le profil minimal par Moon et réorganiser les tâches et contrôles existants.

**Architecture:** La racine expose `install`, `check`, `test` et hérite des regroupements ciblés de `.moon/tasks/`. `home` possède les déploiements, `harness` les agents et leurs projections, `tooling` les outils et oracles ; les trois projets Rust conservent leurs particularités.

**Tech Stack:** Moon 2.5.3, Bun 1.4.0, TypeScript, Cargo, Homebrew.

**Spec:** Proposition validée dans la conversation de migration Moon le 7 septembre 2026, avec `checks.yml`, `packages.yml`, contrôles Lua/Fish racine et déploiements directement dans `home/moon.yml`.

## Contraintes

- Installation du poste macOS ; conserver les contrôles et tests Ubuntu existants.
- Préserver les destinations, refus de collisions, restauration Fisher et erreurs Git observables.
- Second passage du minimal : statut zéro, stdout/stderr vides, artefacts observés identiques.
- Aucun test de déclaration ni inventaire miroir ; valider Moon nativement.
- Aucune installation globale depuis le worktree ; fixtures locales et smoke macOS en CI.
- Indexer les nouveaux fichiers avant les contrôles qui utilisent l'index Git.
- Pas de commentaire ajouté ; conserver les variantes Rust, le verrouillage et les mutex.
- Semctx reste explicite ; optionnels et nettoyage Make encore transitoires.

## Task 1: Déploiement des configurations

**Files:** `home/moon.yml`, utilitaires `tooling/deploy-*.ts`, tests de déploiement Fish/XDG/liens et leur support.

**Interfaces:** Tâches `home:install`, `home:bat`, `home:fish`, `home:nvim`, `home:wezterm`, `home:git-delta`, `home:starship`, `home:tmux`, `home:cspell-config`, `home:hunspell-dictionaries`, `home:arnes-config`. CLI de lien réutilisée pour les projections du harness.

- [x] Adapter les tests comportementaux existants au point d'entrée Moon et observer leurs échecs avant la migration.
- [x] Porter les logiques de déploiement possédées en TypeScript ; conserver leurs chemins d'échec et le rejeu silencieux.
- [x] Définir les tâches et prérequis de configuration, sans appeler Make.
- [x] Exécuter les tests de déploiement concernés sur fixtures et faire relire le diff.

## Task 2: Graphe Moon et harness

**Files:** `moon.yml`, `.moon/workspace.yml`, `.moon/tasks/{workstation,javascript,rust,harness-claude,harness-codex}.yml`, `harness/moon.yml`, manifestes des trois outils Rust, `Makefile`.

**Interfaces:** Tâche racine `dependencies` pour l'installation native du paquet ; `rust` pour les prérequis Cargo ; CLI de lien de Task 1 ; tâches de paquets de Task 3.

- [x] Reclasser les tâches existantes avec héritage ciblé et vérifier le graphe natif.
- [x] Déployer les projections Claude/Codex depuis leurs sources canoniques, y compris le skill `issue-simplify` ajouté sur main.
- [x] Fermer les dépendances des hooks et des tâches Cargo ; conserver les variantes et comportements des builds.
- [x] Retirer les recettes minimales remplacées et adapter les consommateurs encore sous Make.
- [x] Vérifier les erreurs de build, les projections et les diagnostics Arnes.

## Task 3: Contrôles, smoke et consommateurs

**Files:** `.moon/tasks/{packages,checks,tests}.yml`, `tooling/moon.yml`, workflows, `install.sh`, `tooling/upgrade`, oracles extraits du Makefile et leurs tests.

**Interfaces:** Cibles racine et projets des Tasks 1–2 ; mêmes préconditions et observations que le smoke Make remplacé.

- [x] Mettre la préparation des exécutables et l'exécution des contrôles dans Moon.
- [x] Remplacer les invocations Make du minimal par Moon dans l'installateur, l'upgrade et le smoke.
- [x] Conserver les tests d'échec de l'amorçage et les assertions comportementales du smoke.
- [x] Adapter les workflows et leurs sélections affectées ; conserver les environnements et intégrations existants.
- [x] Exécuter lint, types et tests pertinents, puis faire relire le diff.

## Task 4: Cohérence et livraison

**Files:** ADR-001/002/003/023/038/041, README, document des exceptions, skill projet `dotfiles`.

- [x] Appliquer les révisions validées, en limitant les garanties aux preuves obtenues.
- [x] Actualiser le skill `dotfiles` selon la frontière migrée et vérifier son index.
- [x] Exécuter les contrôles du changement indexé et une revue indépendante finale.
- [ ] Terminer la CI de la PR #316 ; le premier smoke minimal macOS a réussi, les corrections de résolution des runtimes sont en validation.
