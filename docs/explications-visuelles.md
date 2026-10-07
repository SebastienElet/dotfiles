# Explications visuelles — livraison de #398

## Usage et périmètre

Après déploiement du harnais, invoquer `$visual-explanation <sujet>` dans Codex ou
`/visual-explanation <sujet>` dans Claude Code et Cursor. Les explications simples
restent dans la CLI ; un HTML local temporaire sert les explications denses ou
interactives. Aucun navigateur ne s'ouvre spontanément au premier plan.

Décisions utilisateur de cet entretien : Codex et Claude CLI prioritaires,
visuels CLI d'abord, liens identifiables entre agents, aucun archivage des HTML,
annotation différée. Elles précisent le [contrat de #398](https://github.com/SebastienElet/dotfiles/issues/398).

La [préférence commune](../harness/visual-presentation.md) contient le choix de
présentation. Le [skill](../harness/skills/visual-explanation/SKILL.md) construit,
inspecte et livre les visuels ; `output-discipline` conserve la concision des
rapports. Les procédures de diagnostic, planification et review gardent leur
responsabilité. Aucun workflow de review, outil de rendu, MCP, hébergement,
framework ou hook d'annotation n'a été ajouté.

Les références comparées, révisions et licences sont dans
[comparison.md](../harness/skills/visual-explanation/references/comparison.md).
`effective-html` est la référence principale pour les HTML autonomes et la
traçabilité ; `visual-explainer` contribue le propos des figures. Ses seuils,
HTML systématique, SVG imposé et commandes de review ne sont pas repris.
Plannotator reste évalué séparément, sans installation ni succès runtime allégué.

## Déploiement canonique

| Agent  | Préférence                                                                                  | Skill user                            |
| ------ | ------------------------------------------------------------------------------------------- | ------------------------------------- |
| Claude | Include `@visual-presentation.md` dans le `CLAUDE.md` global ; lien vers la source partagée | `~/.claude/skills/visual-explanation` |
| Codex  | Quatrième source de l'assembleur de `~/.codex/AGENTS.md`                                    | `~/.agents/skills/visual-explanation` |
| Cursor | Même source liée sous `.cursor/rules/visual-presentation.mdc`, avec `alwaysApply: true`     | `~/.cursor/skills/visual-explanation` |

Arnes déclare ces installations dans `home/.arnes.yaml`. Moon conserve ses tâches
existantes. Le nettoyage connaît les deux nouveaux liens ; l'empreinte des
instructions Claude suit aussi la source liée. Les ADR acceptées 025, 027, 029,
038 et 040 restent en vigueur.

Depuis ce worktree, seuls les déploiements isolés ont été exécutés, avec
`moon exec --no-actions --upstream none` et un HOME temporaire. Aucun déploiement
vers le HOME personnel ni installation globale n'a été réalisé. Depuis le
checkout canonique après intégration, les tâches de déploiement concernées sont :

```sh
moon run harness:claude-instructions harness:claude-skills \
  harness:codex-instructions harness:codex-skills \
  harness:cursor-rules harness:cursor-skills
```

Leurs dépendances ne doivent pas être exécutées comme une installation globale
depuis un worktree. Les Doctor Arnes `instructions`, `rules` et `skills` passent
séparément pour chaque combinaison prise en charge dans le HOME temporaire.
Ces résultats prouvent les projections, pas l'activation des sessions personnelles.

## Observations des agents

Exécutées le 6 octobre 2026, sur macOS 27.0.1 arm64 : Codex CLI 0.160.1,
Claude Code 2.1.291, Cursor CLI 2026.08.31-4057e58. Modèles demandés :
`gpt-5.6-sol` à effort `low`, alias `sonnet` résolu en `claude-sonnet-5-5`,
`grok-4.6` annoncé par Cursor comme `Grok 4.6 High Fast`. Aucun modèle n'a été
substitué après échec.

Les scénarios utilisent des copies des vrais fichiers du checkout : manifeste,
tâches Moon, installateur, assembleur, tests de l'assembleur et version précédente
de celui-ci obtenue avec `git show HEAD`. Les sources sont en lecture seule ; les
sessions ne déploient rien. Codex utilise un HOME/CODEX_HOME temporaire, ignore
configuration user et règles d'exécution, et reçoit `TERM=dumb`. Claude utilise
l'authentification existante avec hooks désactivés, configuration projet,
MCP vide, outils de lecture et le répertoire du skill explicitement autorisé.
Cursor conserve son authentification avec `CURSOR_CONFIG_DIR` et
`CURSOR_DATA_DIR` temporaires, mode `ask`, sandbox et confiance explicite envers
la fixture construite. Ce ne sont pas les sessions personnelles complètes.

| Scénario rejoué                                               | Codex                      | Claude                                | Cursor                                  |
| ------------------------------------------------------------- | -------------------------- | ------------------------------------- | --------------------------------------- |
| Flux réel : source, manifeste, liens vers les trois agents    | Schéma vertical et sources | Schéma et tableau des états de preuve | Schéma, destinations et états de preuve |
| Diagnostic réel : source visuelle manquante dans l'assembleur | Succès/échec en texte      | Arbre de lecture et refus             | Schéma et explication du refus          |
| Avant/après réel : trois sources deviennent quatre            | Deux petits schémas        | Tableau comparatif                    | Tableau et explication du changement    |
| Réponse simple : nom de la branche courante                   | Une commande               | Commande et alternative               | Commande et alternatives                |
| Invocation explicite : chemin succès/échec                    | Diagramme CLI              | Diagramme CLI                         | Diagramme CLI                           |
| Navigateur absent et fichier d'une autre session présent      | Repli CLI, limite nommée   | Repli CLI, limite nommée              | Repli CLI, limite nommée                |

Les 18 exécutions retenues ci-dessus terminent avec le code 0. Les deux derniers
agents restent plus bavards que Codex sur la question simple, mais ne produisent
pas d'artefact. Les réponses contiennent des sources ; aucun compte ou pourcentage
de gain de compréhension n'est déduit de cette observation.

Les premiers essais de flux manquaient de fichiers nécessaires dans la fixture :
ils ne servent pas de preuve finale. La fixture a été complétée. Des essais suivants
de Claude et Cursor confondaient encore sélection du manifeste et activation.
Le skill précise désormais que l'activation signifie chargement et application
en session. Trois nouvelles sessions, une par agent, avec invocation explicite,
ont ensuite séparé déclaration, déploiement et activation non observée. Le passage
du prompt naturel à l'invocation explicite ne permet pas d'attribuer ce résultat
au seul ajout de la phrase ; ce n'est pas une ablation ni une garantie universelle.

La découverte de Claude est visible dans ses événements `init` (`skills` et
`slash_commands`) ; les invocations explicites donnent les réponses observées,
sans trace du contenu complet injecté par le préprocesseur. Les traces Codex et
Cursor montrent la lecture du skill ; Cursor le lit aussi dans des scénarios naturels.
Un essai HTML naturel Codex lit le `SKILL.md` sans invocation explicite et crée
un fichier. Son événement `turn.completed` est reçu, mais le processus ne termine
pas dans les 180 secondes et est arrêté avec le code 143 : cet essai ne compte
pas comme exécution réussie. L'artefact peut être inspecté séparément de ce délai.

Les sorties `exec` et `--print` établissent la génération de diagrammes texte et
de comparaisons. Elles n'établissent ni le rendu Mermaid ni l'affichage d'images
dans les TUI interactives. Le chemin complet Fish → Codex TUI, la CLI interactive
Claude, Cursor graphique et les applications Codex/Claude restent non exercés.
Les documentations consultées ne remplacent pas ces essais de rendu :
[Codex CLI](https://learn.chatgpt.com/docs/codex/cli),
[terminal Claude](https://code.claude.com/docs/en/terminal-config),
[règles Cursor](https://cursor.com/docs/rules).

## HTML, données et repli

Un HTML temporaire produit dans cette session Codex représente le déploiement,
l'avant/après de l'assembleur et un graphique des installations déclarées.
Il a été ouvert dans Chrome headless 154.0.8037.98, profil temporaire, accès
HTTP/HTTPS bloqués pendant le rendu. Captures inspectées à 1440 et 390 px :
libellés lisibles, aucune superposition gênante ni débordement horizontal,
conclusion visible avant les détails. Les quatre états du filtre ont été exercés
par événement `change`, le volet de détail par Entrée, et la navigation par Tab.
Les cibles des liens locaux existent ; l'issue GitHub a été relue par API.
Aucune erreur de page ni ressource échouée n'a été observée dans ce rendu.

L'utilisateur a jugé ce premier exemple trop peu soigné. Une nouvelle version
temporaire du 2026-10-06 privilégie le schéma de déploiement, des connexions
visibles et un avant/après typographique ; les chiffres restent secondaires.
Le complément de composition dans la référence du skill décrit ces choix sans
imposer un modèle de page. Cette version a été inspectée dans le même Chrome,
à 1440 et 390 px, en clair et sombre, avec HTTP/HTTPS bloqués : aucun débordement
horizontal ni erreur de page observé, quatre états du filtre, détail par Entrée
et navigation par Tab exercés. Cette inspection technique n'établit pas
l'approbation esthétique de l'utilisateur. Les exemples précédents sont préservés.

Le graphique utilise le manifeste courant au 2026-10-06 : **34 installations
user Claude, 37 Codex, 29 Cursor**, unité « installations de skills déclarées
par agent », skills externes exclues. Il ne mesure ni activation ni utilisation.
Le fichier de données de la fixture reprend ces valeurs et leur source.

L'HTML généré par l'essai CLI interrompu est un second exemple distinct, lui aussi
inspecté à 1440 et 390 px. Les quatre filtres ont été exercés par Entrée sur les
boutons, puis Tab a déplacé le focus vers le filtre suivant. Comptes et cibles
locales concordent ; aucun débordement horizontal ni erreur de page observé.
Sa liste exhaustive des skills allonge fortement la vue étroite ; le filtre réduit
cette densité. Son inspection ne transforme pas l'arrêt du processus en succès. Les fichiers HTML,
captures et traces restent temporaires et ne sont pas versionnés. Aucun mécanisme
de suppression à la fin d'une session n'est installé, aucune durée de rétention
n'est promise, et un lien ne doit pas être supprimé avant sa consultation.

Les trois essais de repli retenus ne modifient pas le fichier sentinelle `existing.html`.
Un probe séparé a réellement tenté une création exclusive au même chemin :
`EEXIST`, avec octets de la sentinelle préservés. Cette observation ne certifie
pas toutes les futures générations des agents ; le skill prescrit une allocation
privée et une création exclusive pour chaque nouvel artefact.

## Vérification et limites

- `harness:check` : passe sur macOS arm64 avec la toolchain Rust 1.98.1 ;
  format, Clippy, compilation, tests Arnes/proof-integrity, types/lint/format
  TypeScript, Prettier et contrats d'évaluation natifs. Aucun appel LLM dans cette gate.
- `repository:typescript-test` et `repository:cspell-check` : passent, **596 tests
  réussis, 1 test Docker existant non exécuté, 0 échec**, sur 60 fichiers pour la
  suite TypeScript. Aucun skip n'a été ajouté. Les tests existants de déploiement,
  nettoyage et rejeu exercent les vraies tâches Moon dans des HOME temporaires.
- TDD : absence de la préférence dans l'assembleur et empreinte Claude inchangée
  reproduites avant correction ; tests publics verts après correction. Les omissions
  de nettoyage ont également été observées par les tests existants avant correction.
- Doctor procédural `skill-manager` : frontmatter standard, description ciblée,
  sections, références/licences, slug unique, adapters et trois liens user contrôlés.
  Index régénéré deux fois avec sortie identique. Validation standard indisponible :
  `skills-ref` n'est pas installé ; aucun validateur supplémentaire n'a été installé.
- Semctx : outils MCP découverts, préflight sur la racine absolue avec
  `initialized: false`. Aucun `.semctx/` dans ce worktree ; protocole no-op, sans
  initialisation implicite, contrat, index ou verdict Plane A/B inventé.

Le premier contrôle global résolvait Clippy vers Homebrew 1.99 et échouait sur
une assertion Rust préexistante. La toolchain déclarée 1.98.1 a ensuite été
exercée en plaçant `~/.cargo/bin` en tête de PATH, sans suppression de lint ni
modification de cette assertion. Les hooks d'édition restent consultatifs.

Linux, CI distante, installation minimale complète macOS et déploiement du HOME
personnel ne sont pas exercés. Les revues indépendantes et leurs éventuelles
limites complètent ces checks ; une gate verte n'est pas une revue ni une preuve
de gain de compréhension. Ces vérifications ont précédé le commit, le push et
la publication de la PR.
