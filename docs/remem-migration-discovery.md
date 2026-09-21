# Migration remem : découverte et essai de mémoire partagée

État final au 21 septembre 2026 : **installation locale déployée**, décrite dans
la [procédure d'exploitation](remem.md) et la section de mise en service ci-dessous.
Les sections de découverte et d'essai conservent leurs observations antérieures.
Le partage explicite d'une
mémoire par projet via MCP fonctionne dans l'essai Codex CLI/Desktop ci-dessous,
y compris entre worktrees et après suppression du worktree d'origine. Il dépend
des instructions suivies par les agents. La version publiée `v0.6.93` ne regroupe
pas automatiquement les worktrees pour ses hooks. Le parcours Claude reste non
vérifié : l'utilisateur ne dispose pas d'abonnement Claude sur cet ordinateur.
Cette absence ne bloque pas le travail indépendant sur Codex. Cette note rapporte des observations,
pas une nouvelle ADR ni une garantie de capture automatique.

## Version et exécuteur vérifiés

- [Release v0.6.93](https://github.com/majiayu000/remem/releases/tag/v0.6.93), publiée
  le 8 septembre 2026 ; commit `dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d`.
- Binaire publié `remem-darwin-arm64.tar.gz`, version affichée `0.6.93`, schéma
  `v92`. SHA-256 vérifié contre le fichier publié `SHA256SUMS` :
  `246b5429fa2b141424ba95186b2f2317f796c0f4b27b1c14ee5d8b9a10167d81`.
- Exécuteur `codex-cli`, modèle explicite `gpt-5.6-luna`, raisonnement `low`.
  Les profils des deux hôtes pointent vers le même profil Codex dans la base
  temporaire. Trois diagnostics natifs `remem model test --live` répondent `ok`.
- Le processus observé lance réellement la CLI officielle installée :

  ```text
  codex --ask-for-approval never exec --ephemeral --ignore-user-config --ignore-rules --skip-git-repo-check --sandbox read-only --json --output-last-message <fichier-temporaire> --model gpt-5.6-luna -c model_reasoning_effort="low" -
  ```

L'[exécuteur publié](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/ai/codex_cli.rs)
laisse la CLI gérer l'authentification et fixe `REMEM_DISABLE_HOOKS=1` dans son
enfant. Le [routeur](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/ai.rs)
propage l'erreur de l'exécuteur sélectionné, sans bascule automatique vers HTTP.
Aucun fichier d'authentification ni secret du trousseau n'a été lu par le diagnostic.

## Source canonique et inventaire initial

| Composant                     | Source ou destination                                                                                                                                          | Traitement prévu si la migration devient possible                                                    |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Moteur actuel                 | `tooling/agent-memory/`, tâches Moon, binaire `~/.local/bin/agent-memory`                                                                                      | Remplacer après validation du nouveau parcours                                                       |
| Souvenirs du moteur           | `~/.local/share/agent-memory/`                                                                                                                                 | `agent-memory audit --include-terminal --format json` retourne zéro entrée ; réauditer avant bascule |
| Hooks mémoire Claude et Codex | Arnes, sous `tooling/arnes/src/hooks/`, appelé par `.moon/tasks/harness-{claude,codex}.yml`                                                                    | Remplacer les handlers `UserPromptSubmit` dédiés à `agent-memory` ; préserver les autres hooks       |
| Configuration générée         | `~/.claude/settings.json`, `~/.codex/hooks.json`                                                                                                               | Régénérer par le déploiement canonique, pas uniquement éditer les destinations                       |
| Gouvernance mémoire           | `harness/skills/memory-governance/` et règle Cursor `harness/rules/memory-governance-cursor.mdc`                                                               | Réconcilier tous les consommateurs avant retrait du moteur partagé                                   |
| Déploiement des skills        | `tooling/install-agent-skills.ts`, tâches Moon ; Make pour Cursor                                                                                              | Préserver les skills métier ; retirer uniquement les références remplacées                           |
| Mémoire native Claude         | Neuf fichiers Markdown dans cinq répertoires `~/.claude/projects/*/memory/`                                                                                    | Sauvegarder et examiner leur provenance avant toute sélection ou migration                           |
| Mémoire native Codex          | `~/.codex/memories/` vide ; `codex features list` indique `memories=false`                                                                                     | Aucun souvenir natif à importer dans cet état local                                                  |
| MCP et tâches de fond         | Aucun enregistrement remem trouvé dans les configurations des agents inspectées ; aucun LaunchAgent mémoire trouvé ; répertoire `~/.codex/automations/` absent | Ne pas confondre cet inventaire local avec un inventaire des services distants                       |
| Instructions stables          | `harness/AGENTS.md`, `SOUL.md`, `USER.md` ; liens Claude et assemblage `~/.codex/AGENTS.md`                                                                    | Préserver                                                                                            |
| Relais et suivi des tâches    | `agent-handoff`, documents de projet, notes Obsidian                                                                                                           | Préserver : hors mécanisme remplacé                                                                  |

Les [ADR-003](adr/003-deploiement-par-symlinks.md) et
[ADR-038](adr/038-frontieres-home-harness-tooling.md) définissent le déploiement.
L'[ADR-042](adr/042-memoire-durable-locale-partagee.md), encore acceptée, décrit
le moteur actuel et son identité fondée sur `git-common-dir`. Une migration
effective devra remplacer cette décision explicitement.

## Limite des hooks : identité des worktrees

Dans la version publiée, [l'identité projet](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/project_id.rs#L24-L33)
est la racine du worktree. La résolution utilise
[`git rev-parse --show-toplevel`](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/git_util.rs#L358-L363).
Le [test upstream](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/project_id.rs#L491-L501)
attend explicitement le chemin du worktree lié.

Les alias existent dans la bibliothèque, mais leur écriture est exposée par
[un exemple Rust](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/examples/project_alias_apply.rs),
pas par une commande du binaire publié ni un outil MCP. Ils demandent une
correspondance exacte préenregistrée et ne regroupent pas les nouveaux worktrees.
Forcer `--project` lors d'une recherche ponctuelle ne corrige pas l'identité
utilisée par les hooks de capture.

Reproduction avec le binaire publié, une base SQLCipher temporaire et des données
fictives :

| Répertoire interrogé | Identité affichée par `context --debug` | Résultats de `search NEBULA-742 --project … --json` |
| -------------------- | --------------------------------------- | --------------------------------------------------- |
| Checkout A           | Chemin absolu de A                      | 2                                                   |
| Worktree lié à A     | Chemin absolu du worktree               | 0                                                   |
| Dépôt indépendant B  | Chemin absolu de B                      | 0                                                   |

Les deux entrées ont été créées par les interfaces publiques `preferences add`
et MCP `save_memory`, puis retrouvées par un autre processus et exportées en
Markdown. Elles sont classées `legacy_unverified`, avec
`current_context_eligible=false` : aucun rappel automatique n'est démontré par
ces écritures manuelles. Le résultat sur B établit seulement l'isolation de
cette recherche explicitement filtrée.

Cette limite concerne le routage automatique des hooks. Elle n'interdit pas aux
agents de fournir explicitement le même `project` aux outils MCP depuis leurs
worktrees. L'utilisateur a autorisé un essai de ce second chemin. Aucun wrapper,
outil d'alias permanent ou retrait du filtre projet n'a été ajouté.

## Vérifications et limites

Environnement exercé : macOS arm64, Codex CLI `0.154.0`, Claude Code `2.1.236`.
L'application de bureau se trouve dans `ChatGPT.app`, version `26.908.70816`,
avec le Codex embarqué `0.154.0-alpha.6.2`.

La [note upstream sur Desktop](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/docs/research/codex-app-sessionstart-visibility-2026-06-29.md)
rapporte une injection `SessionStart` visible du modèle, mais pas un bloc visible
dans la conversation. Elle concerne une version antérieure et ne valide pas
les hooks de l'application installée ici. L'essai Desktop réel décrit plus bas
porte sur MCP, avec les hooks désactivés pour le projet de test.

- Recherche CLI : environ 20 ms par invocation sur cette base de deux entrées.
  Rendu de contexte sans mémoire admissible : 40–50 ms. Ce ne sont ni une mesure
  de rappel dans un agent ni une démonstration de gain de qualité.
- Trois appels LLM de diagnostic ont réussi. Le premier précède l'initialisation
  SQLCipher et son usage n'a pas été enregistré, avec avertissement explicite.
  `remem usage` rapporte pour les deux autres : 10 514 tokens d'entrée, 27 136
  de cache lu et 10 de sortie, soit 37 660 selon sa comptabilité. Le tarif du
  modèle est inconnu de remem ; le montant affiché `0.0000` ne prouve pas un coût.
- Quota Codex du compte : 84 % disponibles avant les sondes, 83 % ensuite sur
  la fenêtre hebdomadaire. Ce delta inclut le travail de la session et toute
  utilisation concurrente ; il n'est pas attribuable aux seuls diagnostics.
- `remem doctor` inspecte la base, mais retourne un échec global : hooks/MCP non
  installés, contrôle Cursor en échec et enrichissement des entrées en attente.
  Aucun état « installation saine » n'est revendiqué.
- Capture croisée Claude/Codex par hooks, redémarrage de l'application complète et
  retrait de l'ancienne mémoire : **non exécutés**. La correction et le rappel dans
  de nouveaux processus/tâches Codex via MCP sont observés dans l'essai suivant.
- Les limites de 90 s par appel LLM, les baux, les tentatives bornées et le budget
  `worker --once` sont présents dans le code publié. Le budget de 180 s est
  vérifié entre éléments de travail, pas une borne absolue sur un appel en cours.
  Aucun scénario de quota épuisé ou de crash worker n'a été exercé.

## Essai autorisé : mémoire projet explicite via MCP

Le projet fictif est
`/Users/sebastien/Documents/Codex/2026-09-21/remem-mcp-probe`.
Ses deux worktrees partagent le même `AGENTS.md`, avec un `CLAUDE.md` qui
l'importe. L'instruction impose de chercher et d'enregistrer les faits durables
avec ce chemin canonique comme `project`, sans filtre de branche pour les faits
de portée projet. Elle demande de conserver la provenance et la branche d'origine.
Les demandes métier ne rappellent pas aux agents d'utiliser la mémoire.

Le serveur MCP est le binaire publié intact, avec une base SQLCipher séparée sous
`/tmp/remem-mcp-probe-20260921/store`. Recherche locale `feature-hash`, outils
natifs `search`, `get_observations`, `save_memory` et `govern_memory` uniquement.
Les hooks et mémoires natives sont désactivés dans les sessions CLI et la
configuration locale du projet Desktop. Aucun adaptateur ni moteur n'est ajouté.

| Parcours réellement exécuté                         | Résultat observé                                                                                                     |
| --------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| Codex CLI, worktree A, décision fictive             | Recherche préalable, `save_memory`, puis lecture de contrôle ; projet canonique, origine `probe-a`, entrée `id=1`    |
| Nouvelle session CLI, worktree B                    | Retrouve `SABLE-583`, attente de 37 secondes, verrou de 36 secondes                                                  |
| Nouvelle tâche Desktop, checkout principal          | Retrouve les mêmes trois informations par MCP                                                                        |
| Codex CLI, autre projet, même base                  | Recherche avec sa propre clé projet ; répond « information inconnue »                                                |
| Correction explicite dans Desktop                   | Réutilise le `topic_key`, met à jour `id=1`, puis relit : attente de 53 secondes, verrou de 52 secondes              |
| Suppression du worktree A par `git worktree remove` | Worktree propre supprimé, mémoire et checkout principal conservés                                                    |
| Nouveau processus CLI, worktree B après suppression | Retrouve les valeurs corrigées 53/52                                                                                 |
| Nouvelle tâche Desktop après suppression            | Retrouve les valeurs corrigées 53/52                                                                                 |
| Claude → Codex et Codex → Claude                    | Non exécutés : aucun abonnement Claude disponible sur cet ordinateur, confirmé par l'utilisateur ; CLI non connectée |

Les tâches Desktop proviennent d'une tâche de préparation ne contenant que le
chemin de travail. Elles ne reprennent pas l'historique des décisions fictives ;
le dernier rappel ne reprend pas non plus la tâche ayant reçu la correction.
Les versions d'application et de CLI restent celles indiquées plus haut, avec
`gpt-5.6-luna` et raisonnement `low` pour les agents Codex de l'essai.

### Ce que cet essai ne garantit pas

- Les recherches et écritures sont déclenchées par le modèle sous instruction,
  pas par une capture systématique des hooks. L'identité canonique est déclarée
  explicitement dans cette fixture ; sa découverte générique n'est pas testée.
- Le souvenir reste `legacy_unverified`, avec
  `classification_reason=legacy_unverified_provenance_missing` et
  `current_context_eligible=false`. Il est accessible par recherche explicite,
  sans être rendu admissible à l'injection automatique de contexte.
- Les premières réponses CLI/Desktop signalent cette réserve ; la CLI finale
  qualifie même l'information d'inconnue, tandis que le dernier rappel Desktop
  énonce la valeur corrigée sans réserve. La prudence n'est donc pas uniforme.
- Supprimer le worktree n'a ni supprimé ni invalidé l'entrée. Le résultat montre
  la persistance après correction explicite, pas une détection automatique de
  fausseté. Aucun ancien incident réel ou souvenir utilisateur n'a été importé.
- Un seul scénario positif et un scénario d'isolation ont été exercés, sans test
  de concurrence entre corrections. Ce n'est pas une preuve générale de qualité,
  d'isolation ou de robustesse.

### Mesures et traces

Les journaux natifs remem donnent 18–28 ms pour les premières recherches/lectures
positives ; les recherches parallèles sans résultat dans le projet témoin montent
à 38–48 ms. Les sessions CLI complètes durent 31,49 s pour l'écriture, 47,57 s pour
le premier rappel, 36,29 s pour le témoin et 18,83 s après suppression. Les tours
Desktop durent 14,401 s pour le premier rappel, 12,615 s pour la correction et
28,248 s pour le dernier rappel. Le temps complet inclut le modèle et son contexte.

Les quatre sessions CLI rapportent respectivement 152 674, 99 108, 98 581 et
99 081 tokens d'entrée, dont 113 152, 69 632, 80 896 et 78 848 tokens en cache ;
635, 486, 386 et 501 tokens de sortie. Ce sont les compteurs de sessions entières,
pas le coût marginal de la mémoire. Aucun coût monétaire n'est déduit.
Le quota hebdomadaire du compte passe de 80 % à 79 % disponibles pendant l'essai,
sans attribution exclusive aux sondes ni à cette tâche.

Les événements CLI, résultats réduits de Desktop, configuration, journaux natifs
et export Markdown fictif sont sous `/tmp/remem-mcp-probe-20260921/`.
`observed-results.json` résume les résultats ; `store/remem.log` contient les
appels MCP et leurs durées. La consultation native ne nécessite pas l'interface
prototype :

```sh
export REMEM_DATA_DIR=/tmp/remem-mcp-probe-20260921/store
/tmp/remem-discovery-20260921/remem search SABLE-583 --project /Users/sebastien/Documents/Codex/2026-09-21/remem-mcp-probe --json
/tmp/remem-discovery-20260921/remem show 1
/tmp/remem-discovery-20260921/remem export --markdown --output /tmp/remem-mcp-probe-20260921/export --project /Users/sebastien/Documents/Codex/2026-09-21/remem-mcp-probe
```

L'utilisateur a confirmé ne pas disposer d'abonnement Claude sur cet ordinateur.
Aucune connexion ou souscription Claude n'est donc un préalable à la suite du
travail sur Codex. Les contrats et configurations Claude peuvent être contrôlés
sans session LLM, mais cela ne validerait ni son comportement réel ni le parcours
croisé. Ces résultats resteront explicitement non vérifiés. Aucun jeton n'a été
extrait, aucun abonnement ni appel API facturé de remplacement n'a été configuré.

## Artefacts des essais initiaux

Lors des essais initiaux, aucun composant ni souvenir utilisateur n'avait été
modifié. La mise en service ultérieure est distinguée ci-dessous ; elle a été
précédée d'une sauvegarde vérifiée des configurations et des neuf fichiers Claude.

Le téléchargement, la configuration de test, la base chiffrée, les captures de
diagnostic et l'export fictif sont locaux, hors Git, sous
`/tmp/remem-discovery-20260921/`. Ce chemin temporaire n'est pas une sauvegarde
durable. Aucun service de fond n'a été installé.

Pour consulter ces seuls artefacts tant qu'ils existent :

```sh
export REMEM_DATA_DIR=/tmp/remem-discovery-20260921/store
/tmp/remem-discovery-20260921/remem model current
/tmp/remem-discovery-20260921/remem search NEBULA-742 --project /private/tmp/remem-discovery-20260921/project-a --json
/tmp/remem-discovery-20260921/remem status --json
/tmp/remem-discovery-20260921/remem doctor
/tmp/remem-discovery-20260921/remem usage
/tmp/remem-discovery-20260921/remem export --markdown --output /tmp/remem-discovery-20260921/export --project /private/tmp/remem-discovery-20260921/project-a
```

Fermer le shell de diagnostic retire la variable exportée. Les artefacts fictifs
restent distincts de la base réelle sous `~/.remem/`. Le retour arrière de
l'installation est décrit dans la procédure d'exploitation, pas par suppression
des répertoires temporaires.

## Mise en service locale

L'utilisateur a demandé de passer à l'installation. Les seuls fichiers de cette
migration ont été appliqués au checkout principal avant exécution des tâches Moon,
sans fusion de branche ni réécriture Git. Ses changements non liés ont été conservés.
Les mêmes sources sont proposées dans la PR ; le checkout principal conserve ces
modifications de déploiement non commitées jusqu'à intégration de la PR.

- Le binaire publié `0.6.93` est installé sous `~/.local/bin/remem`, avec checksum
  vérifié par l'installateur upstream figé au commit.
- La configuration liée, la base SQLCipher, le MCP Codex/Claude, le skill partagé
  et les instructions Codex assemblées sont déployés. Le LaunchAgent natif
  `dev.remem.worker` exécute `worker --once` toutes les 300 secondes ; son dernier
  passage observé s'est terminé avec le code zéro.
- Arnes rapporte les MCP et hooks concernés sains. Les anciens hooks mémoire et
  liens `memory-governance` Codex/Claude sont retirés. Le moteur historique reste
  présent pour Cursor. Les fonctionnalités natives sont désactivées :
  `memories=false` pour Codex, `autoMemoryEnabled=false` pour Claude.
- Trois connaissances utiles ont été migrées par la CLI officielle Codex, puis
  relues via MCP et exportées en Markdown. Leurs sources et qualifications sont
  conservées, avec `claim_enabled=false` ; aucune proposition n'est devenue une
  règle actuelle. L'ancienne consigne Bash contredit la politique actuelle et
  l'ancien statut d'annulation n'a pas été revalidé : ils ne sont pas réinjectés.
- Les neuf fichiers d'origine restent en place, leur mécanisme natif étant
  désactivé. Le contrôle automatique d'approbation a refusé leur déplacement,
  car trois entrées migrées ne justifient pas à elles seules ce retrait plus large.
  Aucune tentative de contournement ni suppression n'a suivi.

La sauvegarde initiale est
`~/.local/state/remem-backups/20260921-162605/before-remem.tar.gz` : 33 membres
vérifiés, permissions `0600`, sans fichier d'authentification. Une archive séparée
conserve les sources avant déploiement ; les trois exports sont sous `exports/`.

### Vérification avec le harnais installé

Les sessions suivantes n'utilisent plus les instructions locales de la fixture
initiale. Elles découvrent le skill utilisateur et la configuration MCP déployés.

- La première tentative de migration a été refusée par les permissions MCP de
  Codex en mode non interactif ; aucun ID n'a été annoncé comme sauvegardé.
  Les autorisations natives ciblées `save_memory` et `get_observations` ont ensuite
  été configurées. La reprise a produit trois IDs relus avec succès. La gouvernance
  destructive reste soumise à approbation.
- Une première tâche Desktop a seulement accusé réception sans écrire. La
  découverte explicite des outils a confirmé leur présence. L'instruction de
  démarrage a été clarifiée pour imposer le chargement du skill, sans condition
  ambiguë de disponibilité, et interdire une promesse de sauvegarde sans reçu.
- Une nouvelle tâche Desktop a alors effectué `search`, `save_memory` et
  `get_observations` pour une décision fictive. Une nouvelle session CLI dans ce
  même projet a retrouvé les valeurs attendues, sans les recevoir dans le prompt.
- Une session CLI dans un nouveau worktree sans instructions locales a résolu
  `/private/tmp/remem-installed-git/.git`, enregistré la décision fictive et relu
  l'ID. Cette clé est identique depuis le checkout principal.
- Le premier rappel depuis le checkout a échoué sur une requête trop précise et
  l'agent a donné une réponse issue de notes hors projet. La recherche native avec
  un terme plus court retrouvait pourtant le souvenir. Le skill a été corrigé pour
  essayer une fois un terme distinctif dans le même projet, puis signaler l'absence
  de mémoire sans repli vers un autre corpus. Le rejeu du même prompt a effectué
  les deux recherches dans la même clé projet puis une relecture de l'ID, et
  retrouvé les valeurs attendues sans consulter un autre corpus.

Ces échecs et reprises montrent la limite du rappel guidé par instructions. Ils ne
sont pas masqués par les essais positifs. La base reste locale et les résultats
`legacy_unverified` demandent de vérifier les sources avant une utilisation décisive.
Claude reste configuré statiquement, sans test LLM faute d'abonnement sur ce poste.

Les traces de mise en service sont sous `/tmp/remem-production-verification/`.
Les tests de l'installateur, le lint et le typecheck locaux portent sur le retrait
des liens gérés et la préservation des destinations divergentes et de Cursor.
Sur macOS, les trois suites ciblées totalisent 19 tests réussis ; Linux n'a pas
été exercé localement. La régénération de l'index des skills est byte-identique.
Le relevé Codex après installation indique 72 % disponibles sur la fenêtre
hebdomadaire partagée du compte ; ce relevé n'isole pas le coût de la mémoire.
Le diagnostic global des skills signale aussi des plugins externes non liés à cette
migration ; leur configuration n'a pas été modifiée pour masquer ces écarts.
