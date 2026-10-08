# Couverture des instructions communes dans Cursor

## Sources et découverte

Les contrats officiels ci-dessous ont été consultés le 8 octobre 2026.
Les versions sont celles observées sur macOS Darwin arm64 ; elles ne sont pas
des versions minimales garanties. La matrice décrit les projections configurées,
pas une preuve de réception par les modèles.

| Agent et version observée                          | Instructions projet                                           | Instructions globales, persona et préférences                                                                                                                 | Portée et découverte documentée                                                                                                                                                |
| -------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Cursor Desktop `3.23.23`, CLI `2026.10.01-14929f9` | `AGENTS.md` du dépôt, puis fichiers imbriqués selon la portée | `harness/AGENTS.md`, `SOUL.md`, `USER.md` et `visual-presentation.md` assemblés dans `~/.cursor/plugins/local/dotfiles-harness/rules/common-instructions.mdc` | Plugin local Cursor avec règle `alwaysApply: true` pour Agent Chat ; activation à vérifier après reload. Le CLI peut sélectionner explicitement le plugin avec `--plugin-dir`. |
| Claude Code `2.1.285`                              | `CLAUDE.md` importe `@AGENTS.md`                              | Liens sous `~/.claude/` ; `CLAUDE.md` importe `SOUL.md`, `USER.md` et `visual-presentation.md`                                                                | Instructions user pour tous les projets, plus instructions projet.                                                                                                             |
| Codex CLI `0.160.1`                                | `AGENTS.md` du dépôt et des répertoires jusqu'au CWD          | Les quatre sources du harnais assemblées dans `~/.codex/AGENTS.md`                                                                                            | Instructions globales sous `CODEX_HOME`, puis instructions projet ; les overrides et la limite de taille du consommateur restent applicables.                                  |

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

La réception réelle reste à démontrer : le CLI Cursor a répondu `Not logged in`
et l'accès UI a expiré. Aucun modèle ou effort n'a été exercé. Aucune preuve de
réception Cursor, Claude ou Codex, ni de déploiement global sur ce poste, n'est
déduite du succès des projections. L'installation du poste est macOS-only ;
le workflow `test-deployment` prévoit macOS et Ubuntu. Ubuntu et d'autres
versions Cursor ne sont pas exercés localement.

Pour compléter la preuve, après authentification, utiliser le plugin déployé
dans une nouvelle session CLI en mode `ask` avec
`--plugin-dir "$HOME/.cursor/plugins/local/dotfiles-harness"`, ou Agent Chat
après reload. Sans joindre les fichiers au prompt ni autoriser leur lecture, demander :

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
toutes les tâches. #437 reste incomplet tant que cette observation manque.
