# #562 — Relais à 85 % ou compaction native

**Recommandation : garder provisoirement la politique actuelle.** Aucun avantage de coût total ou
de qualité jusqu'à acceptation n'est établi pour l'une des options. Cette investigation documentaire
ne valide pas le seuil de 85 % et ne modifie ni politique, ni code, ni configuration.

## Périmètre et parcours retenu

Lecture du 19 octobre 2026 sur macOS 27.0.1 ARM64, dépôt à
`5c1c78b607c546bfcfa437dd9ebd73af6ebaf0fe`. Sources publiques lues avec GitHub CLI et la recette
Jina Reader d'Agent-Reach ; configurations locales inspectées sans exporter leurs secrets.
Budget de nouveaux runs modèles : **0**, répétitions : **0** ; aucune nouvelle collecte de
conversation ni infrastructure de mesure, conformément à l'[ADR-043 accepté](../adr/043-telemetrie-arnes-minimale-et-bornee.md).

Parcours retenu : [#561](https://github.com/SebastienElet/dotfiles/issues/561), correction du contrat
de reprise → validation d'une nouvelle session → publication `0.0.2` → consommation dans dotfiles.
L'acceptation observable est la fusion de la [PR #583](https://github.com/SebastienElet/dotfiles/pull/583)
au commit `e3432e6995c5cd5ac1b5c8761f0896f7e114cd39`, après CI réussie sur son candidat
`a47dd2c5b79948b4adbf42c10e1136d26619161c`. C'est un parcours multi-étapes accepté ; sa saturation
de contexte reste inconnue. Entre l'ouverture de la [PR autonome #9](https://github.com/SebastienElet/agent-handoff/pull/9)
et cette fusion : **1 h 44 min 19 s**. Cet intervalle exclut le travail initial et inclut des attentes ;
ce n'est ni le temps actif ni le coût total de la tâche.

Ce choix donne une preuve récente de reprise et une arrivée vérifiable. Il ne fournit pas deux
trajectoires longues comparables : aucune reprise au seuil de 85 % ni compaction n'est attestée
pour la session de développement. Il manque un même point de reprise long, les mesures depuis le
départ et une continuation native jusqu'au même oracle. Les accès CLI ne sont pas le blocage :
Claude Code `2.1.303` et Codex CLI `0.163.0` sont installés et signalent une authentification.
La validité des modèles et les quotas n'ont pas été sondés par une requête modèle.

## Ce qui est observé

L'[ADR-040 accepté](../adr/040-skills-user-dans-harness.md) et [package.json](../../package.json)
désignent désormais `@sebastienelet/agent-handoff@0.0.2`, et non les anciens chemins cités dans
l'issue. Le paquet installé dans le checkout canonique et son [source publié](https://github.com/SebastienElet/agent-handoff/tree/0ef07986961bb028cf00ba856ce4228aba4fd8a3)
ont été consultés : `src/decision.rs` retient l'override explicite, sinon 85 % de la fenêtre fournie ;
`src/run.rs` demande le relais sur `Stop`, une fois par session. Le hook ne lance pas la nouvelle
session. La skill produit un prompt puis arrête le tour ; elle demande de conserver les preuves
négatives utiles et les réserves. Son affirmation de supériorité sur le « bruit » de la compaction
reste une hypothèse, sans comparaison comportementale dans les sources examinées.

Les fichiers utilisateur déclarent Claude `sonnet`, effort `high`, fenêtre de compaction et variable
`CLAUDE_CODE_AUTO_COMPACT_WINDOW` à `600000` ; Codex `gpt-6.1-sol`, effort `high`, fenêtre `512000`,
sans seuil de compaction explicite. Les deux hooks `Stop` référencent le binaire installé ; les
skills pointent vers le paquet canonique. Le [manifeste](../../home/.arnes.yaml) déclare encore
`gpt-5.6-sol` : ces observations de fichiers ne prouvent pas les réglages effectifs d'une session.
Sans override, les seuils calculés seraient respectivement **510 000** et **435 200** tokens ;
ni leur franchissement ni leur ordre avec la compaction native ne sont observés ici.

La PR autonome #9 **rapporte** une reprise réellement exécutée sous Codex `0.163.0`,
`gpt-6.1-sol`, effort configuré `high`, sur macOS ARM64 : une seconde session reçoit le prompt,
lit les fichiers, conserve l'erreur `Unknown option --json`, ne rejoue pas l'essai réfuté et garde
les décisions et validations ouvertes. Elle relève aussi que la fixture ne contient pas
l'implémentation annoncée. C'est une observation publiée, non réexécutée ici, sur une petite fixture
avec skill fournie explicitement ; elle ne prouve pas l'achèvement d'une tâche longue.

| Condition sur ce parcours | Demande / reprise / acceptation                                                         | Coût et qualité disponibles                                                                       |
| ------------------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Relais à 85 %             | Déclenchement inconnu ; reprise Codex de fixture rapportée ; livraison globale acceptée | Preuve négative et réserves conservées ; relectures présentes, sans compte ; coût complet inconnu |
| Compaction native Codex   | Aucune continuation comparable documentée                                               | Pertes, relectures, corrections et interventions humaines inconnues                               |
| Compaction native Claude  | Aucune continuation comparable documentée                                               | Mêmes inconnues ; aucune généralisation depuis Codex                                              |

Pour les trois conditions, tokens non cachés, création/lecture de cache, facture, appels, durée
complète et temps humain sont **indisponibles**, jamais zéro. Aucune fréquence de perte ni de
correction n'est mesurée. La mesure historique [#528](https://github.com/SebastienElet/dotfiles/blob/767ac379d39bcf65419589a8dc116bcd1f0d998f/docs/agent-handoff-transcript-benchmark-528.md)
chronomètre seulement le lecteur sous le seuil ; les [évaluations Arnes](../../harness/evals/README.md)
testent deux recherches synthétiques. Ni l'une ni les autres ne mesurent ce coût jusqu'à acceptation.

## Comparaison des mécanismes documentés

Documentation officielle consultée le 19 octobre 2026 ; elle décrit des capacités, pas leur qualité
sur le parcours retenu.

| Option                                                                                  | Continuité documentée                                                                                                                                                | Conséquence à vérifier sur une tâche longue                                                                                       |
| --------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Relais, Claude ou Codex                                                                 | Nouveau contexte fondé sur le prompt et les références durables ; arrêt et collage dans une nouvelle session prévus par la skill                                     | Synthèse, lancement, relectures et éventuelles corrections s'ajoutent ; leur coût et l'amélioration de focalisation sont inconnus |
| [Claude natif](https://code.claude.com/docs/en/context-window#what-survives-compaction) | Historique résumé ; instructions racine et mémoire réinjectées, jusqu'à cinq fichiers relus ; skills réinjectées avec plafonds ; tâches de fond maintenues           | Continuité sans relais manuel possible ; preuves ou règles locales résumées et skills tronquées peuvent nécessiter des relectures |
| [Codex natif](https://developers.openai.com/codex/cli/slash-commands)                   | `/compact` résume le contexte ; [seuil automatique et fenêtre](https://developers.openai.com/codex/config-reference) sont distincts, seuil absent = défaut du modèle | Continuité dans la session ; aucune garantie documentaire que chaque preuve négative ou permission survive exactement             |

Chez Claude, la [synthèse peut lire le préfixe caché](https://code.claude.com/docs/en/prompt-caching#compacting-the-conversation)
encore chaud ; après expiration elle retraite l'historique sans cache. Le tour suivant reconstruit
le cache de conversation autour du résumé. Un contexte raccourci ne démontre donc pas une économie.
Ne pas transposer cette mécanique à Codex : la [compaction Responses](https://developers.openai.com/api/docs/guides/compaction)
peut transmettre un état opaque chiffré, mais cela n'identifie pas le protocole effectivement utilisé
par la CLI locale. Aucun montant par tâche n'est déduit des abonnements ni du delta de tokens.

## Portée de la recommandation

**Garder**, faute de comparaison jusqu'au même résultat accepté ; ni ajuster le seuil ni retirer
le relais sur cette seule base. Cette conservation est provisoire, sans certification de qualité.
La comparaison reste documentaire ; aucune expérience appariée de contexte long n'a été réalisée.

Condition de réexamen : sur un seul hôte, un point de reprise long attesté et des sorties natives
permettant de compter la session initiale, la synthèse, toutes les reprises et corrections jusqu'à
un oracle d'acceptation identique. Compter aussi échecs, interruptions et minutes humaines ; séparer
cache, facture inconnue et temps écoulé. Une baisse reproductible de ce coût sans perte de preuve,
permission ou réserve justifierait une proposition d'ajustement au mainteneur, avant toute modification.
Toute évolution devrait conserver demande manuelle, absence de demandes répétées, erreurs visibles,
travail durable et contrat Herdr ; revenir à la politique actuelle si ces propriétés régressent.

Aucun run Claude/Codex comparatif, Linux ou intégration Herdr active n'a été exercé pour ce rapport.
