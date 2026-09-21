# ADR-042 — Mémoire durable locale partagée

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-09-21

## Contexte

L'utilisateur demande remem publié pour Codex et Claude Code, sans API payante,
modèle génératif local, modification de remem ou adaptateur mémoire maison.
Après les essais MCP, il autorise une mémoire commune au projet et à ses
worktrees, guidée par les instructions des agents. L'absence d'abonnement Claude
sur ce poste limite les vérifications, sans bloquer l'installation Codex.

Remem `0.6.93` sépare les worktrees dans ses hooks. Le paramètre MCP `project`
permet de choisir explicitement une identité commune. Ce chemin remplace le
retrieval synchrone de `agent-memory` pour Codex et Claude ; ses garanties
d'exécution à chaque prompt et de revalidation des sources ne sont pas conservées.

## Décision

- Remem intact possède la base SQLCipher locale sous `~/.remem/`, les index,
  la recherche, les écritures et les files. `home/.remem/config.toml` est la source
  canonique de sa configuration ; Moon porte son installation et son exploitation.
- Les opérations LLM de remem utilisent `codex-cli`, `gpt-5.6-luna` et un effort
  `low` pour les deux hôtes. La CLI officielle gère l'authentification. Les
  embeddings utilisent le moteur local `feature-hash`.
- Le skill `remem-memory` demande le chemin réel absolu du `git-common-dir`
  comme clé de chaque lecture et écriture. Hors Git, il utilise le chemin physique
  du projet. Deux clones restent distincts. La portée repose sur l'agent ; le
  aucune garantie d'isolation d'accès entre projets n'est revendiquée ici.
- Une instruction globale minimale demande le rappel avant analyse et la
  conservation des connaissances utiles avant livraison. Aucun hook de capture
  remem n'est installé. Arnes retire ses hooks mémoire Codex/Claude et les tâches
  de setup désactivent leurs mémoires natives.
- Le LaunchAgent exécute le worker natif `--once` toutes les cinq minutes.
  Cette intégration ne redéfinit pas ses budgets, baux ou tentatives. Les agents
  doivent signaler un échec direct de `save_memory` sans prétendre une mise en file.
- Les agents doivent conserver provenance, portée, branche et incertitude.
  Ni indexation ni suppression d'un worktree ne démontrent la vérité d'un fait.
  Une correction est explicite ; les sources actuelles restent prioritaires.

Cursor reste hors migration : `memory-governance`, sa règle et `agent-memory`
conservent leur contrat YAML historique. Les instructions stables, préférences,
skills métier, documents de projet, notes Obsidian et `agent-handoff` restent
indépendants de remem.

## Conséquences

Les [essais](../remem-migration-discovery.md) observent l'écriture, le rappel
CLI/Desktop, une correction et la suppression d'un worktree. Ils ne prouvent ni
fiabilité universelle du suivi des instructions, ni vérité des faits, ni
comportement Claude réel, ni résistance aux corrections concurrentes.
La classification `legacy_unverified` reste une limite du chemin MCP testé.

La [procédure d'exploitation](../remem.md) décrit sources, diagnostics, sauvegardes
et retour arrière. Le retrait des souvenirs d'origine est subordonné à leur
sauvegarde et à la vérification de leur migration. Les liens et hooks de
l'ancien mécanisme sont réconciliés séparément par le déploiement existant.

## Alternatives écartées

- Conserver le moteur maison pour Codex/Claude : contraire au remplacement demandé.
- Réécrire les projets des hooks dans un wrapper : adaptateur mémoire interdit.
- Extraire des jetons, facturer une API ou exploiter un modèle génératif local :
  contraire aux contraintes de l'utilisateur.
- Présenter le rappel ou les index comme une revalidation automatique : les essais
  et le code publié ne l'établissent pas.
