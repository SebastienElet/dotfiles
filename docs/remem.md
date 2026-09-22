# Mémoire du harnais avec remem

Codex CLI et Desktop utilisent le même serveur MCP local remem `0.6.93`. Claude
dispose d'une cible de configuration, sans validation LLM sur ce poste faute
d'abonnement. Le skill `remem-memory` demande aux agents de rechercher avant
l'analyse et de conserver les décisions, corrections, causes et procédures utiles.
Depuis le 2026-09-22, les hooks de capture natifs complètent ce suivi d'instructions.

## Installation et sources

Depuis le checkout principal des dotfiles :

```sh
moon run harness:remem
moon run harness:remem-claude
moon run harness:claude-hooks
```

La première cible installe Codex et les hooks de capture des deux hôtes ; les deux
suivantes configurent Claude sans exiger une connexion à son modèle. Fermer puis
ouvrir une nouvelle tâche pour
charger les nouvelles instructions et le MCP. Les hooks ne prennent effet qu'au
prochain démarrage de chaque hôte.

Ne pas lancer `remem install` à la main : `harness:remem-hooks` le fait avec
`--hooks-only`, puis restaure `home/.remem/config.toml`, que cette commande
réécrit à travers le lien déployé en y ajoutant un profil `api.anthropic.com`.

- `.moon/tasks/harness-remem.yml` fixe le tag et l'installateur upstream par commit.
- `home/.remem/config.toml` est lié à `~/.remem/config.toml`. Modifier cette source
  pour changer le modèle ; `remem model use` ne remplace pas ce déploiement.
- `home/.arnes.yaml` déclare le MCP et les skills ; Arnes retire ses anciens hooks
  mémoire de Claude et Codex. `install-agent-skills.ts` retire uniquement leurs
  anciens liens `memory-governance` appartenant à ce dépôt.
- `home/.codex/remem-mcp-tools.toml` autorise les écritures mémoire et lectures de
  contrôle demandées ; les actions de gouvernance restent soumises à approbation.
  La CLI officielle remplace d'abord sa propre section MCP, puis la tâche y ajoute
  ce fragment et relit la configuration avec `codex mcp get remem`.
- `harness/AGENTS.md` appelle le skill `harness/skills/remem-memory/SKILL.md`.
  Les instructions Codex sont assemblées par la cible existante.

L'exécuteur configuré est `codex-cli`, modèle `gpt-5.6-luna`, raisonnement `low`.
L'authentification reste dans la CLI officielle. La recherche `feature-hash` est
locale et ne requiert ni clé API ni modèle génératif local. Le profil partagé est
utilisé aussi pour les opérations mémoire provenant de Claude.

## Portée et exploitation

La clé `project` est le chemin réel du `git-common-dir` : checkout et worktrees
convergent, deux clones restent distincts. Hors Git, le chemin physique du projet
sert de clé. Les agents fournissent cette clé à chaque appel MCP ; ce n'est pas une
restriction d'accès imposée par le serveur. Ils conservent la branche et la source
des faits et vérifient les affirmations importantes avant de les appliquer.

Le LaunchAgent `dev.remem.worker` exécute `remem worker --once` toutes les cinq
minutes. Le code de la release prévoit quatre éléments par passage, une limite
de 180 s vérifiée entre éléments, 90 s par appel LLM et 420 s par job. Ces limites
ont été lues dans le code, sans essai de saturation, timeout ou quota épuisé ;
aucune borne absolue de 180 s par processus n'est revendiquée. Le worker draine aussi
les transcripts Claude et Codex de lui-même, sans hook : une passe observée le
2026-09-22 a ingéré 21 634 messages depuis 947 fichiers. Cette archive brute sert
`search_raw` et `list_raw_sessions`, jamais la promotion : `captured_events` n'est
écrit que par les entrypoints de hook `observe`, `session-init` et `summarize`, et
les candidats produits restent soumis à `remem review`. Les erreurs des files et les
logs natifs restent consultables ; un échec direct de `save_memory` n'est pas une
preuve de mise en file.

```sh
remem status --json
remem model current
remem model test
remem pending --help
remem usage
launchctl print "gui/$(id -u)/dev.remem.worker"
tail -n 60 ~/.remem/remem.log
```

`remem doctor` rapporte `Hooks (claude): 0/6 registered` et `Hooks (codex): no remem
hooks` alors que les entrées sont présentes et que `remem install --target claude
--repair` répond `6/6 registered` sur le même fichier. Ce diagnostic n'est pas une
preuve d'absence : compter les commandes déployées, comme le fait le contrôle de
`harness:remem-hooks`. Remem signale par ailleurs une dérive de binaire, le MCP étant
déclaré via `/bin/sh` par `home/.arnes.yaml` quand les hooks appellent le binaire
directement. Vérifier séparément le serveur enregistré avec `codex mcp get remem`,
le modèle, la base, les files et le dernier code de sortie du LaunchAgent.

Depuis le projet à consulter :

```sh
remem_project="$(realpath "$(git rev-parse --path-format=absolute --git-common-dir)")"
remem search "décision recherchée" --project "$remem_project" --json
remem export --markdown --project "$remem_project" --output /tmp/remem-export
```

La classification `legacy_unverified` ne démontre ni vérité ni fausseté. Une
correction remplace explicitement l'ancien énoncé ; une suppression de worktree
ne déclenche pas une validation sémantique. Les tâches, instructions stables et
notes Obsidian restent dans leurs sources actuelles. Cursor conserve son moteur
historique `agent-memory`, qui n'est donc pas supprimé du dépôt.

## Sauvegarde et retour arrière

Avant la bascule locale, les configurations, leurs sources et les neuf fichiers
de souvenirs Claude ont été sauvegardés dans
`~/.local/state/remem-backups/20260921-162605/before-remem.tar.gz` (archive vérifiée,
permissions `0600`). Aucun fichier d'authentification n'est inclus.

Pour arrêter remem sans détruire ses données :

```sh
launchctl bootout "gui/$(id -u)/dev.remem.worker"
codex mcp remove remem
claude mcp remove --scope user remem
```

Retirer ensuite le fichier `~/Library/LaunchAgents/dev.remem.worker.plist` pour
éviter sa reprise à la prochaine connexion. Conserver `~/.remem/`, dont la clé
SQLCipher et la base, ensemble et hors Git. Restaurer les seuls fichiers concernés
depuis l'archive et leurs sources avant de redéployer les instructions et les hooks
historiques. Ne pas écraser les modifications faites depuis la sauvegarde.

Le [bilan de la PR](https://github.com/SebastienElet/dotfiles/pull/346) distingue les essais
réels, les limites du mode MCP et les parcours Claude non exécutés.

Trois souvenirs utiles ont été migrés, relus et exportés sous `exports/` dans le
dossier de sauvegarde. Les propositions et anciens retours d'agents gardent leur
qualification historique/non vérifiée. Les neuf fichiers Claude restent en place
avec la mémoire native désactivée : leur déplacement a été refusé par le contrôle
automatique d'approbation, car seules trois entrées ont été migrées. Les anciennes
consignes obsolètes n'ont pas été réinjectées.
