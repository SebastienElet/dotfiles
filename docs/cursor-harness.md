# Couverture des instructions communes dans Cursor

## Sources et découverte

Les contrats officiels ci-dessous ont été consultés les 8 et 10 octobre 2026.
Les versions sont celles observées sur macOS Darwin arm64 ; elles ne sont pas
des versions minimales garanties. La matrice décrit les projections configurées,
pas une preuve de réception par les modèles.

| Agent et version observée                          | Instructions projet                                           | Instructions globales, persona et préférences                                                                                                                 | Portée et découverte documentée                                                                                                                                                                                                        |
| -------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cursor Desktop `3.23.23`, CLI `2026.10.01-14929f9` | `AGENTS.md` du dépôt, puis fichiers imbriqués selon la portée | `harness/AGENTS.md`, `SOUL.md`, `USER.md` et `visual-presentation.md` assemblés dans `~/.cursor/plugins/local/dotfiles-harness/rules/common-instructions.mdc` | Plugin local Cursor avec règle `alwaysApply: true` pour Agent Chat ; réception observée dans Desktop Ask avec Grok 4.7 High sur This Mac. Dans les essais CLI du 10 octobre, `--plugin-dir` est accepté mais sa règle n'est pas reçue. |
| Claude Code `2.1.285`                              | `CLAUDE.md` importe `@AGENTS.md`                              | Liens sous `~/.claude/` ; `CLAUDE.md` importe `SOUL.md`, `USER.md` et `visual-presentation.md`                                                                | Instructions user pour tous les projets, plus instructions projet.                                                                                                                                                                     |
| Codex CLI `0.160.1`                                | `AGENTS.md` du dépôt et des répertoires jusqu'au CWD          | Les quatre sources du harnais assemblées dans `~/.codex/AGENTS.md`                                                                                            | Instructions globales sous `CODEX_HOME`, puis instructions projet ; les overrides et la limite de taille du consommateur restent applicables.                                                                                          |

Sources officielles : [Cursor Rules](https://cursor.com/docs/context/rules),
[plugins locaux Cursor](https://cursor.com/docs/plugins#test-plugins-locally),
[format des plugins et règles](https://cursor.com/docs/reference/plugins),
[paramètres CLI Cursor](https://cursor.com/docs/cli/reference/parameters),
[instructions Claude Code](https://code.claude.com/docs/en/memory),
[découverte Codex](https://learn.chatgpt.com/docs/agent-configuration/agents-md).

Cursor documente ses User Rules dans Customize → Rules. Les anciens liens
`~/.cursor/rules/{memory-governance-cursor,visual-presentation}.mdc` restent
déployés, mais leur présence ne démontre pas un chargement global. La nouvelle
projection commune utilise le mécanisme documenté des plugins locaux. Elle
n'ajoute aucune source éditoriale : son contenu est régénéré par le même
assembleur que Codex. Les sources restent sous `harness/` ; ne pas éditer la sortie.

## Installation et configuration Cursor

Depuis le checkout canonique `~/.dotfiles`, `moon exec harness:cursor-instructions`
déploie uniquement la projection commune ; `harness:cursor` l'inclut dans le
profil optionnel avec les autres capacités Cursor.
Le déploiement refuse un répertoire de plugin symbolique avant toute écriture.
Le répertoire du plugin est réel, son manifeste est lié à la source canonique
sous `home/`, et sa règle est un fichier assemblé. La restriction officielle
sur les symlinks de répertoires de plugins vers une cible extérieure est ainsi
évitée. Le déploiement refuse un manifeste divergent ; l'assembleur remplace sa
sortie par renommage et conserve un résultat identique au rejeu.

Après déploiement :

1. Exécuter Developer: Reload Window dans Cursor.
2. Dans Customize, vérifier `dotfiles-harness` et sa règle commune en mode Always.
3. Vérifier que les imports locaux sont autorisés par la configuration de l'organisation.
   Sur Enterprise, ils sont désactivés par défaut. Un plugin marketplace de même
   nom prend priorité sur le plugin local.
4. Ouvrir une nouvelle session Agent Chat dans le projet concerné et vérifier
   la réception des consignes. Les règles ne couvrent pas Cursor Tab ou Inline Edit.

Le diagnostic Arnes `doctor instructions --agent cursor` reste `unsupported`.
Le Doctor des rules existantes vérifie les liens, pas le contexte d'une session.
Le nettoyage `repository:clean` inclut le manifeste lié et la règle générée de
`dotfiles-harness`, en conservant les autres plugins locaux.

## Preuves et limites de #437

Base de discovery : `326f5389334097f5644d551ab4e17d20ff8390b0`.
La PR liée à [#437](https://github.com/SebastienElet/dotfiles/issues/437)
porte le SHA de livraison et les résultats CI après publication.

Le 8 octobre 2026, sur Darwin arm64, Moon `2.6.0` et Bun `1.4.2` ont exécuté
les trois tâches de projection dans le HOME temporaire
`/tmp/cursor-437-deployment.KqKLhr`, avec `--upstream none --no-actions`.
Les destinations réelles de cette fixture ont été inspectées ; le corps Cursor
hors frontmatter était identique à la projection Codex. Ce contrôle ne déploie
pas le worktree dans le HOME du poste. Le rejeu a également réussi avec Moon
`2.5.3`. Les contrôles finaux utilisent Moon `2.5.3` et Bun `1.4.0` :
`repository:typescript-lint`, `repository:typescript-typecheck`,
`repository:typescript-format-check`, `repository:prettier-check` et
`tooling:deployment-test`. Les tests nommés couvrent l'assemblage Cursor,
le rejeu, les collisions de manifeste, le refus de répertoire symbolique et
le nettoyage qui conserve les plugins tiers. Les résultats détaillés sont
dans la PR ; aucun test de réception du modèle n'en est déduit.

Le 10 octobre 2026, après authentification du CLI, le canary a été exécuté
sur les sources du commit `66e204a507b29fe8b3f7944c3d461d27df66ae5c`.
Les [preuves assainies](cursor-harness-evidence-437.json) contiennent les prompts,
réponses, identifiants de session, versions, hashes et événements d'outils,
sans pensées du modèle ni informations d'authentification.

- Session `1277bfbb-c2da-4084-8ce0-a48c443087d3`, sélection `Auto`, mode `ask` :
  exit 0 et aucun outil, mais la réponse déclare absentes la langue/forme d'adresse
  et la préférence de validation du déclaratif. Le `AGENTS.md` projet est reçu.
  Le modèle sous-jacent et l'effort ne sont pas observables.
- Le manifeste régulier ne rétablit pas la persona. Une règle minimale à marqueur
  unique fournie par `--plugin-dir` n'est pas reçue, en modes Ask et Agent,
  avec Composer 2.5. La même règle placée sous `.cursor/rules` est reçue avec
  le marqueur du `AGENTS.md` projet ; les prompts ne contiennent pas ces marqueurs.
- Session `ab52f07b-817d-412e-81bd-69349cd5e8b4`, Composer 2.5, mode `ask` :
  le corps canonique complet, identique octet par octet à la règle du plugin,
  est reçu comme règle **projet** native. La réponse restitue le français et le
  vouvoiement (`SOUL.md`), le traitement d'une ADR et de son éventuelle erreur
  (`harness/AGENTS.md`), le checkpoint avant une nouvelle infrastructure de
  validation (`USER.md`) et la priorité de `AGENTS.md` sur les adaptateurs du projet.
  Aucun outil n'est utilisé.

L'échec est localisé au chemin de chargement du plugin dans cette configuration
CLI, sans cause interne établie. Le contrôle positif projet ne valide pas le
plugin global dans le CLI.

Après connexion de Cursor Desktop `3.23.23`, la tâche native
`harness:cursor-instructions` a déployé temporairement les deux artefacts dans
le HOME réel. Le worktree était verrouillé pendant la présence du manifeste lié.
Après reload, Customize affichait `Dotfiles Harness` comme plugin `Local` et
`common-instructions` parmi les règles utilisateur, avec `alwaysApply: true`.

- Requête Desktop `aebbcd86-d266-4bf7-a036-d25370908dd6`, mode Ask,
  environnement `This Mac`, sélection UI `Grok 4.7 High` : une nouvelle session
  dans un workspace contenant seulement le `AGENTS.md` racine a restitué les
  quatre signaux du canary. Aucun fichier n'était joint au prompt, et aucun
  événement d'outil n'était visible dans l'interface. L'éditeur de la règle,
  ouvert pour inspection, avait été fermé avant l'ouverture du workspace.
- Les deux destinations possédées ont ensuite été retirées, le worktree
  déverrouillé et Cursor rechargé. Le même prompt dans une nouvelle session,
  avec le même workspace, modèle et mode, donne la requête témoin
  `146c77fe-0bbd-458a-a8d3-103d4d6b493e` : langue/forme d'adresse et préférence
  de validation sont déclarées absentes ; la consigne projet reste reçue.
  Aucun événement d'outil n'était visible.

Cette paire établit la réception des signaux communs par le plugin sur la
surface Desktop exercée. Les observations CUA sont consignées par l'auteur ;
elles ne constituent pas une télémétrie backend indépendante. `High` est un
label UI, les paramètres effectifs de raisonnement ne sont pas exposés.
La réception de toutes les consignes et leur respect dans toutes les tâches
ne sont pas démontrés. La projection globale couvre Desktop Agent Chat ; la
réception par le plugin CLI reste non démontrée et n'est pas promise ici.

Aucune réception Claude/Codex, autre modèle ou version Cursor, ni profil optionnel
complet du poste n'est déduite de ces essais. Le poste est macOS-only ; la CI du
commit `66e204a5` a réussi ses 25 contrôles, dont les déploiements macOS/Ubuntu.
Ubuntu n'est pas exercé localement.

Pour rejouer le contrôle de réception après déploiement et reload dans Desktop,
utiliser une nouvelle session sans fichier joint avec le prompt ci-dessous.
Un futur essai CLI avec `--plugin-dir` devra démontrer la réception ; accepter
l'argument n'en est pas une preuve. Sans joindre les fichiers au prompt ni
autoriser leur lecture, demander :

> Indiquez la langue et la forme d'adresse prescrites, la conduite à tenir face
> à une contradiction certaine avec une ADR, et la préférence concernant les
> nouvelles gates de validation du déclaratif. Citez aussi une consigne propre
> au projet courant. Répondez uniquement à partir des instructions déjà reçues,
> sans utiliser d'outil.

Consigner version, SHA des sources et du projet, modèle, effort lorsqu'il est
observable, identifiant de session, prompt, réponse et éventuels appels d'outils.
Attendre le français avec vouvoiement (`SOUL.md`), le blocage du chemin
contradictoire (`harness/AGENTS.md`), le checkpoint avant une nouvelle gate
spontanée (`USER.md`) et une consigne uniquement présente dans l'`AGENTS.md`
du projet. Une réponse obtenue après lecture des fichiers ne prouve pas leur
discovery automatique. Ce contrôle de réception ne prouve pas l'obéissance sur
toutes les tâches.
