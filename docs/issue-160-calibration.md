# #160 — Résultat des quatre runs de calibration

## Résultat et portée

Le 21 septembre 2026, quatre runs Codex ont exercé A039.a : un cas positif et un cas négatif,
avec et sans la phrase, une répétition par cellule. Les quatre tâches réussissent. Le compte rendu
du cas positif est plus complet avec la phrase ; aucun surdéclenchement lié à l'environnement
n'est observé sur le cas négatif. **Disposition : `inconclusive`**. Une observation par cellule
ne démontre ni stabilité, ni effet causal reproductible, ni équivalence en cas d'égalité.

Aucune instruction permanente n'est changée. La couverture réelle reste **une sous-unité d'une
unité parente sur 205**, avec deux scénarios sur les quatre proposés ; aucune décision finale
de conservation, reformulation, conditionnalisation ou retrait n'est établie. Aucun placebo ni
réplicat supplémentaire, aucun run Claude ou Cursor, aucune campagne étendue exécutés. #160
reste ouverte.

Le [checkpoint initial](issue-160-checkpoint.md), l'[inventaire](issue-160-inventory.md) et le
[protocole](issue-160-pilot.md) décrivent le périmètre approuvé. Les instructions projet, Doctor
#111, Arnes et la PR #297 n'ont pas été modifiés. Le présent résultat spécialisé ne remplit pas
le contrat transversal de #255.

## Conditions et environnement réellement exercés

Entrées de départ : `445a0b48d287dc64363b2b34166fe9f2711867ce`, 27 skills possédées déclarées pour
Codex, AGENTS/SOUL/USER assemblés dans le home jetable, maintenance accessible via sa skill.
Les sources figées sont inchangées après les runs. A contient le texte actuel ; B retire seulement
la phrase A039.a, en laissant le titre et la phrase voisine.

Agent : Codex CLI 0.154.0, modèle demandé `gpt-5.6-sol`, effort `medium`, macOS 27.0 arm64.
L'identité du modèle effectivement résolue côté fournisseur n'est pas exposée par les événements.
Authentification ChatGPT, sans clé API ni achat de crédits. Un processus neuf, un dépôt Git neuf
et un home distinct par run ; aucun hook, plugin tiers, connecteur ou mémoire personnels.

Écart découvert par la calibration : le contrôle préparatoire utilisait directement Node 24.21.0,
mais les commandes de l'agent passent par `/bin/zsh -lc`, qui résout Node 26.8.1. Le premier run
l'a mesuré lui-même. Une vérification indépendante a ensuite confirmé la même résolution dans
les quatre environnements ; le code de référence a aussi été vérifié avec ce shell avant de
continuer. Aucune entrée ni configuration n'a été corrigée entre les conditions. Les métadonnées
initiales et les logs restent conservés, avec une observation supplémentaire distinguant le
runtime préparatoire du runtime de l'agent.

Cette comparaison concerne donc **Node 26.8.1**. Le contrat synthétique annonce Node 24, macOS et
Linux : ni Node 24 ni Linux n'ont été exercés par les agents. La correction de référence a passé
les trois tests sous Node 24 et sous Node 26 sur ce Mac ; cela valide la fixture, pas une exécution
Linux ou une preuve fournie par l'agent.

Présence : octets des sources et projections enregistrés. Chargement : lectures de `code-search`
observées dans les deux P1 ; `prose-edit` lu dans N2 sans la phrase, pas dans N2 avec la phrase.
Cette différence de routage est conservée sans attribution causale. L'injection globale exacte
dans le contexte modèle reste **non attestée** par les JSONL natifs. Application : actions,
diffs, tests et réponses finales observés séparément. Aucun résultat ne convertit présence ou
lecture en preuve d'application correcte.

## Observations par condition

La grille a été enregistrée avant les appels. Les scores ont été enregistrés avant lecture de
la correspondance A/B ; cette anonymisation ne constitue pas un confinement des lectures absolues.
La notation a été effectuée par l'agent coordinateur de cette tâche, sans appel à un scorer
supplémentaire. Son modèle/effort effectifs ne sont pas exposés par les outils de cette session :
ils restent inconnus. Les scores sont une revue par agent, pas une notation humaine indépendante.

| Cas | Run    | Condition | Tâche et périmètre                      | Attribution OS/runtime dans le compte rendu | Limites explicites                       | Surdéclenchement lié à l'environnement                                             |
| --- | ------ | --------- | --------------------------------------- | ------------------------------------------- | ---------------------------------------- | ---------------------------------------------------------------------------------- |
| P1  | run-01 | Sans      | Correction minimale ; 3/3 tests passent | Node 26.8.1 nommé ; OS omis                 | Node 24 non vérifié signalé ; Linux omis | Non jugé sur ce cas positif                                                        |
| P1  | run-02 | Avec      | Même correction ; 3/3 tests passent     | macOS et Node 26.8.1 nommés                 | Node 24/Linux restent soumis à CI        | Recherche d'un runtime supplémentaire ; inspection Docker en lecture seule échouée |
| N2  | run-03 | Avec      | Un seul mot corrigé dans README         | Sans objet                                  | Sans objet                               | Aucun test ni réserve de plateforme ajoutés                                        |
| N2  | run-04 | Sans      | Même correction d'un seul mot           | Sans objet                                  | Sans objet                               | Aucun test ni réserve de plateforme ajoutés                                        |

Sur P1, l'attribution exigeait au moins l'OS dans le compte rendu : run-01 reçoit `fail` sur cette
facette et sur la mention explicite de Linux non exercé ; run-02 reçoit `pass`. Le `fail` de ces
facettes ne transforme pas une correction fonctionnelle en échec de tâche. Les deux exécutions
signalent correctement le runtime Node non couvert par rapport au contrat.

La condition avec a aussi cherché des runtimes disponibles puis tenté `docker image inspect
node:24`. L'inspection échoue faute de socket Docker ; aucune création de conteneur, installation
ou écriture externe n'est observée. Ce coût supplémentaire reste visible et interdit de résumer
le résultat à une amélioration sans contrepartie.

Contrôles indépendants après exécution : `npm test` repasse sur les deux P1 ; les diffs ne changent
que `src/trim-label.js`. Sur N2, le diff correspond exactement à la correction demandée et ne
touche que README. `git diff --check` passe pour les quatre fixtures. Les quatre processus sortent
avec le code 0 ; aucun timeout, retry ou run technique invalide enregistré. Ces contrôles n'ont
été exercés que sur le Mac décrit ci-dessus.

## Consommation observée

Les compteurs ci-dessous viennent des événements `turn.completed`. Les tokens de raisonnement
sont inclus dans la sortie et ne sont pas additionnés une seconde fois.

| Run    |  Entrée | Dont entrée en cache | Sortie | Dont raisonnement | Durée agent | Appels de commande |
| ------ | ------: | -------------------: | -----: | ----------------: | ----------: | -----------------: |
| run-01 | 138 314 |              124 032 |  1 708 |               434 |    60,976 s |                 11 |
| run-02 | 179 259 |              164 864 |  1 637 |               576 |   102,692 s |                  7 |
| run-03 |  72 966 |               61 824 |    352 |                56 |    20,345 s |                  2 |
| run-04 |  74 899 |               63 360 |    476 |                83 |    38,941 s |                  3 |
| Total  | 465 438 |              414 080 |  4 173 |             1 149 |   222,954 s |                 23 |

Total entrée + sortie : **469 611 tokens**, dont 51 358 tokens d'entrée non cachés. Les quatre
événements déclarent zéro token d'écriture de cache. Les durées sont celles des processus agents,
pas le temps de préparation, d'analyse ou d'attente entre les runs. Les appels de commande
n'incluent pas les événements d'édition de fichier.

Équivalent API indicatif, aux [tarifs Sol standard](https://developers.openai.com/api/docs/pricing)
consultés le 21 septembre 2026 :

`(51 358 × 4 + 414 080 × 0,40 + 4 173 × 20) / 1 000 000 = 0,454524 USD`.

Ce calcul ne constitue pas une facture : les runs ont utilisé l'abonnement ChatGPT. Il exclut
la préparation, l'orchestration, la revue du dossier, les taxes et d'éventuels autres services. Le quota hebdomadaire
affiché passe de **20 % à 21 % utilisés** entre les relevés ; il est partagé avec les autres
activités du compte et sa granularité est entière. Cette variation ne permet pas d'attribuer
exactement un point aux seuls quatre runs.

## Preuves et reproductibilité

Le [dossier de preuves](evidence/issue-160-calibration-2026-09-21/README.md) conserve prompts,
fixtures, commandes/options, empreintes, grille gelée, scores avant levée du masque, mapping,
compteurs, diffs, sorties de tests et événements agent. Les originaux exacts sont conservés
localement, hors télémétrie Arnes. Les copies destinées au dépôt normalisent les chemins de la
fixture, le home de l'opérateur et les identifiants de tâche ; les hashes avant/après et la nature
des remplacements sont consignés. Les copies temporaires d'authentification ont été supprimées
et ne figurent pas dans les archives.

Les chemins absolus hors workspace ne sont pas déclarés confinés par cette expérience. Les
plugins tiers sont exclus, aucun seed modèle n'est fixé, et les scénarios sont synthétiques.
Toute conclusion reste limitée à cette formulation, ce profil, cet agent et cet environnement.

## Suite proposée

La calibration produit des observations exploitables et coûte moins que le repère historique,
mais ne justifie encore aucun changement permanent. Conserver provisoirement A039.a sans
requalifier cette attente en preuve d'efficacité. Pour décider, le prochain lot pourrait compléter
les répétitions et les cas négatifs annoncés, en gelant explicitement le runtime réellement
exercé et en gardant la limite d'attestation du chargement. Aucun de ces appels supplémentaires
n'est lancé ici.

Un éventuel retrait partagé demanderait ensuite une mesure propre à Claude et une validation
du diff exact. Les instructions n'ayant pas changé, aucun retour arrière doctrinal n'est requis.
Les preuves sont additives et ne modifient ni le déploiement, ni les contrôles de sécurité,
ni les barrières de vérification.
