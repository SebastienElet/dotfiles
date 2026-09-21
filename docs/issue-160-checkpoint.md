# #160 — Inventaire et premier lot proposé

Checkpoint initial au 21 septembre 2026, avant autorisation du pilote : découverte documentaire et inspection locale uniquement. Aucun nouveau
run comportemental, aucune instruction permanente modifiée. Ce document propose une expérience ;
il ne décide pas encore de retirer une règle.

Depuis ce checkpoint, les quatre runs initiaux ont été autorisés et exécutés ; voir le
[rapport de calibration](issue-160-calibration.md). Les constats « aucun nouveau run » et la
couverture nulle ci-dessous décrivent le point de départ, pas le résultat de la calibration.
L'extension de 24 runs autorisée ensuite est décrite dans le [rapport du lot](issue-160-ablation.md).

## Périmètre et autorité

Révision inspectée : `445a0b48d287dc64363b2b34166fe9f2711867ce`. Les instructions racine et
`docs/AGENTS.md` ont été relues. Les ADR-035, ADR-036, ADR-037 et ADR-043 sont acceptés dans cette
révision. L'ADR-036 impose une attribution marginale : son expérience par bloc ne mesure pas
chaque phrase. L'ADR-037 reconnaît explicitement son exception historique à cette admission ;
ce n'est pas une contradiction nouvelle à réparer dans ce lot.

Le périmètre reprend [#160](https://github.com/SebastienElet/dotfiles/issues/160) : les trois fichiers
globaux et la source exposée par la règle de maintenance. Les instructions propres aux projets,
Doctor #111, les plugins tiers et l'implémentation d'Arnes restent hors périmètre. Leurs effets
éventuels sont des facteurs de contexte à consigner, pas une autorisation de les modifier.

## Sources et chargement

| Source canonique                                              | Claude Code                                     | Codex                                                  | Cursor                                                                                |
| ------------------------------------------------------------- | ----------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------- |
| `harness/AGENTS.md`                                           | Lien vers `~/.claude/CLAUDE.md`                 | Assemblage dans `~/.codex/AGENTS.md`                   | Pas de projection globale déclarée pour cette source                                  |
| `harness/SOUL.md`                                             | Lien `~/.claude/SOUL.md`, import `@SOUL.md`     | Contenu concaténé après AGENTS                         | Pas de projection globale déclarée                                                    |
| `harness/USER.md`                                             | Lien `~/.claude/USER.md`, import `@USER.md`     | Contenu concaténé après SOUL                           | Pas de projection globale déclarée                                                    |
| `harness/skills/agent-instructions/references/maintenance.md` | Règle utilisateur Markdown, sans filtre `paths` | Référence lue après activation de `agent-instructions` | Installation native non déclarée ; découverte possible par compatibilité, non exercée |

`harness/rules/agent-instructions.md` est un lien relatif vers cette dernière source, pas une
cinquième doctrine. `home/.arnes.yaml` déclare la skill `agent-instructions` pour Codex seulement.
Les consommateurs sont établis par ce manifeste, `.moon/tasks/harness-claude.yml`,
`.moon/tasks/harness-codex.yml`, `tooling/install-agent-skills.ts` et
`tooling/assemble-agent-instructions.ts`. L'assembleur supprime les lignes d'import et concatène
AGENTS, SOUL, USER ; le seul champ `source` du manifeste ne décrit pas tout cet assemblage.

Inspection sur macOS 27.0, Darwin arm64 : les liens Claude et celui de la skill Codex se résolvent
vers le checkout principal. Les quatre contenus y sont identiques à ceux du worktree. Le fichier
Codex déployé est identique à l'assemblage attendu. Les chemins natifs de cette skill sont absents
dans `~/.claude/skills` et `~/.cursor/skills`, conformément aux installations déclarées. Aucune
réparation ou installation n'a été exécutée.

Ces constats prouvent la **présence** et les octets déployés. La doctrine globale est visible dans
le contexte fourni à cette tâche et la référence de maintenance a été lue pendant la découverte ;
cela ne mesure ni son déclenchement spontané ni son effet causal. Le **chargement** et
l'**application** par de nouveaux processus Claude/Codex/Cursor restent à mesurer séparément.

Documentation officielle consultée le 21 septembre 2026 :

- [Codex AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md) décrit une chaîne
  construite au démarrage ; [skills](https://learn.chatgpt.com/docs/build-skills) décrit le chargement
  progressif. CLI observée : `codex-cli 0.154.0`. L'aide locale confirme notamment `--ephemeral`,
  `--ignore-user-config` et `--ignore-rules` ; ce dernier concerne les règles d'exécution, pas une
  preuve de suppression des instructions Markdown.
- [Claude memory](https://code.claude.com/docs/en/memory) documente imports et règles utilisateur.
  CLI observée : `2.1.236`. La documentation courante n'atteste pas l'exécution locale.
- [Cursor skills](https://cursor.com/docs/skills) documente aussi `~/.agents/skills` : l'absence de
  lien natif Cursor ne prouve donc pas l'indisponibilité. Aucune version ni activation Cursor
  vérifiée ici. Ne pas lui transférer une conclusion Codex.

Empreintes SHA-256 des octets inventoriés :

| Source      | Lignes | SHA-256                                                            |
| ----------- | -----: | ------------------------------------------------------------------ |
| AGENTS      |     94 | `4518607f35b5b0c48e49d44f7e495c75781dd4d2f0bc97bd35e9d87395650610` |
| SOUL        |     32 | `4a94270116070a72d50286f713de0f76b36f800cb1b3455cd2872e765959ae61` |
| USER        |    223 | `e9146bc21cedd4a0e1f3afa5a3ab72e1f85c7d06d8f1f5c07ad3a362fe3afdd7` |
| Maintenance |     13 | `f7e43dfe8853a2bc70e2641493e72d81d83ec973d3b78489b87e4eaea5ec4196` |

## Preuves retrouvées

| Référence                                                                                                                                                | Observation relue                                                                                                                                                  | Ce qu'elle ne prouve pas                                                                                                    |
| -------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| E035 — [ADR-035](adr/035-agents-md-elague-par-mesure.md)                                                                                                 | 36 runs Claude/Opus 5, six scénarios, deux conditions, trois répétitions ; ablation du fichier projet                                                              | Effet individuel des règles globales actuelles ; données brutes non retrouvées dans les emplacements inspectés              |
| E036 — [ADR-036](adr/036-regles-ia-admises-par-ablation.md)                                                                                              | 21 runs Claude/Opus 5 `xhigh`, trois scénarios, trois répétitions et placebos ; juge Sonnet ; effet du bloc vérification et du refus de consigner un contournement | Attribution par clause, autre agent, autre formulation ; environnement détaillé et traces brutes non fournis dans l'ADR     |
| E036-activation                                                                                                                                          | Quatre activations de `code-enforcement`, dont trois runs arrêtés après activation                                                                                 | Fin de tâche, non-activation sur cas négatifs ou bénéfice global                                                            |
| E037 — [ADR-037](adr/037-commentaire-absent-par-defaut.md)                                                                                               | Violations historiques et adoption explicitement non mesurée des clauses commentaires                                                                              | Efficacité démontrée de la rédaction actuelle                                                                               |
| E078 — [rapport historique](https://github.com/SebastienElet/dotfiles/blob/a169253da6d7a56d04573bf0af9a9986ccd44aba/docs/comment-rule-codex-ablation.md) | Relu via Git ; producteur Sol `medium`, scorer Terra `medium`, Codex 0.152.0 ; 21 runs ; C1 et C2 `inconclusive`, confirmé dans `decision.json`                    | No-op, transfert Claude/Cursor, validation du harnais actuel ; commit non ancêtre de HEAD, rapport absent de l'arbre actuel |
| E-clarification — [rapport](requirements-clarification-validation.md)                                                                                    | Rappel permanent abandonné : injection système et chemin réel ont des résultats différents ; suractivation Cursor et dégradation de cas négatifs Claude consignées | Ablation des clauses actuelles de USER ; modèle/effort absents de ce rapport, raws non relus                                |
| E-Arnes — [protocole actuel](../harness/evals/README.md)                                                                                                 | Trois cas code-search, oracles versionnés, conservation des INVALID et comparaison contrôlée ; aucun JSON dans `harness/evals/evidence`                            | Campagne live exécutée, couverture USER/SOUL, activation interne de skill, efficacité du harnais entier                     |
| E-déploiement                                                                                                                                            | Sources, liens et assemblage comparés sur ce Mac ; tests existants d'assemblage et de déploiement localisés                                                        | Comportement agent ; tests existants non réexécutés pendant cet inventaire                                                  |

E036 rapporte aussi une expansion de périmètre de 2,5 à 6 fois avec l'ancienne formulation
« that gap is the first thing to fix ». Cette phrase est absente du fichier actuel : elle ne
doit pas devenir le candidat implicite du nouveau lot.

E078 conserve les résultats défavorables : plancher sans commentaire inadmissible, dommages de
longueur, placebos non uniformément plats, erreur du scorer sur sept comptes de fichiers S3,
incident de démarrage du scorer et confinement des lectures absolues `not_enforced`. Le rapport et
la décision calculée ont été relus ; les traces brutes locales ignorées par Git n'ont pas été
retrouvées ni requalifiées. Le retrait décrit dans cette branche historique n'est pas l'état
du snapshot actuel.

Les autres évaluations de skills retrouvées sont des corpus spécialisés, pas des ablations des
quatre sources. Leur présence ne remplit aucune cellule comportementale de cet inventaire.

## Articulation proposée avec #255 et #297

[#255](https://github.com/SebastienElet/dotfiles/issues/255) reste la capacité transversale de
comparaison. #160 en consomme les contrôles disponibles et produit les décisions par règle.
Pas de second moteur, schéma universel, collecte de sessions personnelles ou migration v1 dans #160.
L'ADR-043 continue d'exclure contenus et transcripts de la télémétrie : les preuves synthétiques
volontairement produites par le pilote seront séparées de ce store.

Le runner actuel installe par défaut seulement `Context Management` et `code-search`. Son option
`--variant-file` remplace le texte installé, mais ne fournit ni les modes globaux de chaque agent
ni un oracle de restitution de vérification. Ses rapports retiennent des observations normalisées,
pas le stdout agent brut. Forcer ces nouveaux cas dans son contrat donnerait une fausse comparabilité.
Le pilote réutilise donc l'exécution native CLI, les fixtures jetables et la revue des preuves,
comme les expériences spécialisées existantes, sans modifier Arnes.

[PR #297](https://github.com/SebastienElet/dotfiles/pull/297), relue au SHA
`d6efa7105889ba851bac25db1e76c2e23d1304e4`, est ouverte, en brouillon ; l'API la déclare non
fusionnable à cette lecture. Elle propose `harness/invariants/registry.json`, le workflow de
promotion de `harness-reflection`, ses validateurs et fixtures. Ce sont des propositions, pas
l'autorité en vigueur. Ses 91 chemins modifiés n'incluent aucune des quatre sources inventoriées.

Le chevauchement est sémantique : identité, preuve, approbation, promotion et retrait d'une règle.
Les identifiants de l'inventaire désignent des unités d'analyse, pas de nouveaux invariants actifs.
Si #297 aboutit, relier une décision approuvée à son registre lorsque son contrat s'applique ; ne
pas y enregistrer automatiquement toutes les préférences ni copier son état de cycle de vie.
Recontrôler son SHA et les chemins avant toute PR de modification ; si elle touche entre-temps
la règle ou le protocole, refaire la comparaison avant le gel des entrées. Aucune modification ni
publication sur #297 n'a été effectuée.

## Décision proposée au checkpoint

Consulter l'[inventaire](issue-160-inventory.md) et le [protocole du pilote](issue-160-pilot.md).
Couverture textuelle : 205 unités parentes identifiées, dont 44 AGENTS, 15 SOUL, 133 USER et
13 maintenance. Les regroupements et exclusions sont explicités dans l'inventaire ; ce comptage
n'est pas un nombre de clauses atomiques définitivement établi. Couverture comportementale
nouvelle : **0/205** ; décisions finales fondées sur cette campagne : **0/205**. Le pilote ne vise
qu'une phrase d'A039, sans pouvoir valider à lui seul toute cette unité.

Conserver provisoirement les textes en place tant que leur preuve est manquante ou non concluante ;
cela décrit l'état d'attente, pas une décision finale « conserver » fondée sur une efficacité prouvée.

Le premier lot mesure une seule clause de restitution de l'environnement sous Codex. Une seconde
formulation, Claude, Cursor, une mise en skill et les autres règles exigent leurs propres lots.
Le pilote peut conclure qu'aucune modification n'est justifiée. Une réduction de tokens seule
ne permet aucune promotion.

La validation demandée porte sur ce pilote et son coût, puis une nouvelle présentation donnera les
modifications exactes et leurs résultats avant de toucher aux instructions permanentes. Une PR
ultérieure ne contiendra que les changements justifiés et approuvés, leurs preuves et leur retour
arrière ; elle ne fusionnera rien et ne clôturera pas #160 sur la réussite de ce premier lot.
