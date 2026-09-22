# ADR-042 — Mémoire durable locale partagée

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-09-22

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

Le 2026-09-22, le mode guidé par les instructions seules n'avait produit aucune
promotion : `captured_events` restait à zéro alors que l'archive brute comptait
21 634 messages de transcripts, produits par des appels manuels à `ingest-sessions`
et non par le worker. La table `captured_events` n'est écrite que par
les entrypoints de hook `observe`, `session-init` et `summarize` ; `ingest-sessions`
n'alimente que `raw_messages`, qui sert `search_raw`. L'utilisateur décide donc
d'installer les hooks de capture, en connaissance de leurs deux limites vérifiées :
la clé `project` des hooks est le chemin physique du cwd, et la promotion des
candidats reste soumise à `remem review`.

## Décision

- Remem intact possède la base SQLCipher locale sous `~/.remem/`, les index,
  la recherche, les écritures et les files. `home/.remem/config.toml` est la source
  canonique de sa configuration ; Moon porte son installation et son exploitation.
- Les opérations LLM de remem utilisent `codex-cli`, `gpt-5.6-luna` et un effort
  `low` pour les deux hôtes. La CLI officielle gère l'authentification. Les
  embeddings utilisent le moteur local `feature-hash`.
- Le skill `remem-memory` demande le chemin réel absolu de la racine de l'arbre de
  travail, `git rev-parse --show-toplevel`, comme clé de chaque lecture et écriture.
  Hors Git, il utilise le chemin physique du projet. Deux clones restent distincts.
  La portée repose sur l'agent ; aucune garantie d'isolation d'accès entre projets
  n'est revendiquée ici.
- Une instruction globale minimale demande le rappel avant analyse et la
  conservation des connaissances utiles avant livraison. Arnes retire ses hooks
  mémoire Codex/Claude et les tâches de setup désactivent leurs mémoires natives.
- Les hooks de capture natifs sont installés par `harness:remem-hooks`, qui appelle
  `remem install --hooks-only` pour les deux hôtes : six entrées Claude, trois
  entrées Codex, dont la capture reste `drain-only`. La tâche restaure ensuite
  `home/.remem/config.toml`, que `remem install` réécrit à travers le lien déployé
  en y ajoutant un profil `api.anthropic.com` contraire à la contrainte d'absence
  d'API payante. Arnes préserve ces entrées : ses suppressions ne visent que ses
  propres commandes.
- Le skill est aligné sur la clé des hooks, mesurée le 2026-09-22 : la racine de
  l'arbre de travail, y compris depuis un sous-répertoire. Hooks et écritures MCP
  partagent donc un espace de noms unique par arbre de travail. La convergence entre
  un worktree et son checkout est abandonnée : elle n'existe en amont que par les
  alias de projet, dont le binaire publié n'expose aucune commande. Aucune
  rapatriation après coup n'est possible, `reroute --target-project` ne changeant ni
  `memories.project` ni la portée de recherche ; ce qui doit survivre à un worktree
  est réécrit sous la clé du checkout principal.
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

Les [essais de la PR](https://github.com/SebastienElet/dotfiles/pull/346) observent l'écriture, le rappel
CLI/Desktop, une correction et la suppression d'un worktree. Ils ne prouvent ni
fiabilité universelle du suivi des instructions, ni vérité des faits, ni
comportement Claude réel, ni résistance aux corrections concurrentes.
La classification `legacy_unverified` reste une limite du chemin MCP testé.

Les hooks installés le 2026-09-22 ont été vérifiés par lecture des destinations et
par exécution de `arnes setup hooks` sur les deux hôtes, qui a conservé les six et
trois entrées. La chaîne de capture est active dans l'heure qui a suivi :
`captured_events=64 -> observations=25 -> candidates=20 -> promoted=0`, avec
`pending_review=20`, et le hook `session-init` injecte un index de candidats au
prompt. La promotion reste donc à démontrer : elle attend `remem review`, comme
l'annonçait l'issue amont #475. Les candidats observés portent une clé de worktree,
ce qui confirme le cloisonnement décrit plus haut.

`remem doctor` rapporte par ailleurs `Hooks (claude): 0/6 registered` là où
`remem install --repair` rapporte `6/6` sur le même fichier, et `Hooks (codex): no
remem hooks` alors que trois entrées sont déployées. Cette divergence, non résolue
sur les deux hôtes, coexiste avec une dérive de binaire signalée par remem, le MCP
étant déclaré via `/bin/sh` par `home/.arnes.yaml`.

La [procédure d'exploitation](../remem.md) décrit sources, diagnostics, sauvegardes
et retour arrière. Le retrait des souvenirs d'origine est subordonné à leur
sauvegarde et à la vérification de leur migration. Les liens et hooks de
l'ancien mécanisme sont réconciliés séparément par le déploiement existant.

## Alternatives écartées

- N'installer aucun hook de capture remem : clause en vigueur jusqu'au 2026-09-22,
  abandonnée sur mesure. Sans eux, `captured_events` restait à zéro et aucune
  promotion n'était possible, `ingest-sessions` n'alimentant que `raw_messages`.
- Conserver le moteur maison pour Codex/Claude : contraire au remplacement demandé.
- Réécrire les projets des hooks dans un wrapper : adaptateur mémoire interdit.
- Extraire des jetons, facturer une API ou exploiter un modèle génératif local :
  contraire aux contraintes de l'utilisateur.
- Présenter le rappel ou les index comme une revalidation automatique : les essais
  et le code publié ne l'établissent pas.
