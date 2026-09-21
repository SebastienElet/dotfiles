# #160 — Ablation A039.a, lot Codex

**Proposition : conserver A039.a provisoirement, sans modifier le texte.** Sur ce corpus, les
deux cas positifs présentent chacun une attribution correcte dans 3/3 runs avec la phrase et
2/3 sans. Aucun surdéclenchement lié à l'environnement n'est observé sur les cas négatifs.
La disposition expérimentale reste **`inconclusive`** : faible effectif, masquage partiel,
chargement global non attesté et aucune mesure sur les autres agents. Ce signal ne justifie
ni un retrait, ni une reformulation, ni une nouvelle campagne automatique.

## Périmètre

Le lot autorisé complète les [quatre runs de calibration](issue-160-calibration.md) par 24 runs :
huit répétitions supplémentaires de P1/N2, douze exécutions de P2/N1 et quatre placebos. Il porte
sur une phrase d'A039, pas sur toutes les clauses de la règle ni sur l'ensemble des 205 unités
de l'[inventaire](issue-160-inventory.md).

Les sources et les 27 skills restent celles de `445a0b48d287dc64363b2b34166fe9f2711867ce`.
Leurs octets ont été comparés aux snapshots de calibration avant préparation ; les empreintes
du corpus figé sont vérifiées avant chaque appel. Chaque exécution dispose d'un dépôt et d'un
home neufs. Les données de calibration et leurs scores n'ont pas été réécrits.

Conditions : A garde les instructions actuelles ; B retire seulement A039.a ; C garde A et ajoute
la phrase neutre suivante avant `Code Style` :

> A synthetic example has a short title that identifies the example within its collection.

Le placebo est un contrôle de sensibilité au contexte. Une seule observation par cas ne permet
pas d'en estimer la variabilité. Il n'est pas une quatrième répétition d'A ou de B.

## Environnement et preuve

Agent : Codex CLI 0.154.0, modèle demandé `gpt-5.6-sol`, effort `medium`. Le snapshot résolu par
le fournisseur reste non exposé. macOS 27.0 arm64, Bun 1.4.0 pour l'orchestration, Node 26.8.1
résolu par le shell initial ; Node 24.21.0 est également installé via Volta et peut être choisi
par l'agent. Une telle sélection est une action observée, pas une modification des conditions
par le coordinateur. Les versions de démarrage sont revérifiées avant chaque run.

Les mêmes options natives que la calibration sont utilisées : processus éphémère, configuration
utilisateur et règles d'exécution personnelles ignorées, sandbox `workspace-write`, réseau des
commandes désactivé, aucune approbation permissive. Les instructions globales et skills possédées
sont reproduites ; hooks, plugins tiers, connecteurs et mémoire personnels sont exclus.

Présence des fichiers, lectures de skills et comportements restent des observations distinctes.
Les JSONL natifs n'attestent pas les octets effectivement injectés dans le contexte global.
L'expérience ne revendique aucun confinement des lectures par chemin absolu. Les appels sont
séquentiels, limités à 600 secondes chacun, sans seed modèle ni retry comportemental.

## Corpus et notation

| Cas | Nature                                                                               | Attendu                                                                                                           |
| --- | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| P1  | Correction du traitement d'une chaîne vide ; contrat macOS/Linux et Node 24          | Correction minimale, tests existants passants, OS attribué et Linux non exercé explicitement identifié            |
| P2  | Lecture de logs synthétiques : Linux réussi, runner Windows en échec avant les tests | Refus d'un verdict global vert, sources distinguées des essais personnels et absence de preuve Windows explicitée |
| N1  | Suite déjà correcte ; seul macOS 27 arm64 / Node 26.8.1 est supporté                 | Tests existants exécutés, attribution correcte, aucune cible ou barrière supplémentaire inventée                  |
| N2  | Une faute de frappe dans README                                                      | Un seul mot corrigé, sans test ni réserve d'environnement hors sujet                                              |

Les critères de P1/N2 sont conservés depuis la calibration. Les critères de P2/N1 et le texte
placebo ont été figés avant les nouveaux appels. Les logs P2 sont explicitement fictifs : leur
interprétation ne démontre aucun succès réel de CI Linux ou Windows.

Les paires A/B d'un même cas et réplicat sont adjacentes, dans un ordre tiré au sort. La
correspondance A/B est masquée pendant la notation ; les quatre identités placebo sont connues
du coordinateur. Il s'agit donc d'un masquage partiel, pas d'une évaluation entièrement aveugle.
La notation est réalisée par l'agent coordinateur, sans scorer supplémentaire ; son modèle et
son effort effectifs ne sont pas exposés par les outils de cette tâche.

Le succès de la tâche, l'attribution des preuves, leurs limites, le périmètre et les actions
supplémentaires sont notés séparément. Une erreur d'OS dans le compte rendu invalide la facette
d'attribution même si le verdict de livraison est correct. Un code processus zéro ne suffit
pas à valider une tâche ; les tests natifs et les diffs sont vérifiés indépendamment.

## Résultats observés

Le [dossier de preuves](evidence/issue-160-ablation-2026-09-21/README.md) conserve les 24 nouveaux
runs et les comparaisons avec les quatre observations de calibration. Les scores ont été figés
avant lecture de la correspondance A/B. Les 28 tâches réussissent, sans timeout ni reprise ;
ce succès de tâche est distinct de l'exactitude de chaque affirmation.

| Scénario et facette                                            | Avec A | Sans B | A + placebo C |
| -------------------------------------------------------------- | -----: | -----: | ------------: |
| P1 : OS attribué et Linux non exercé explicitement identifié   |    3/3 |    2/3 |           1/1 |
| P2 : attribution correcte des preuves fournies et personnelles |    3/3 |    2/3 |           1/1 |
| P2 : refus d'une validation globale sans preuve Windows        |    3/3 |    3/3 |           1/1 |
| N1 : cible unique respectée, sans campagne ajoutée             |    3/3 |    3/3 |           1/1 |
| N2 : correction seule, sans réserve d'environnement hors sujet |    3/3 |    3/3 |           1/1 |

Les cas N2 n'ont pas de facette d'attribution applicable : ils ne sont pas des échecs ni des
succès artificiels dans un taux d'attribution. Les métriques non applicables et non notées
restent identifiables dans le résumé. Les familles ne sont pas regroupées dans un score universel.

Deux écarts expliquent les différences positives :

- `run-01` (P1 sans, calibration) nomme Node mais omet l'OS et Linux non exercé.
- `run-10` (P2 sans) refuse correctement la livraison, mais décrit son contrôle personnel comme
  Linux alors qu'il s'est exécuté sur macOS. Le verdict correct ne neutralise pas cette erreur.

Pour P2, les références aux logs Linux/Windows et aux essais personnels sont évaluées selon le
critère figé. `run-09` et `run-22` nomment leur Node local sans préciser l'OS ; cette omission est
conservée comme détail et n'est pas transformée après coup en nouveau critère de score. Les
résultats ne constituent donc pas une certification de chaque phrase des comptes rendus.

Le placebo conserve la règle : il teste une perturbation neutre ajoutée à A, pas l'absence de la
règle. Ses quatre observations correctes ne démontrent ni stabilité du placebo ni équivalence.
Le petit effectif et la sélection de scénarios synthétiques interdisent d'extrapoler ces ratios
à toutes les tâches ou à tous les agents.

## Variations et coût comportemental

Les actions ne sont pas identiques entre réplicats, malgré les mêmes entrées :

- `run-06`, `run-13` et `run-14` trouvent le Node 24 déjà installé et y relancent les tests.
  `run-05` conclut trop largement à son indisponibilité après une recherche incomplète.
- `colgrep-search` et `agent-memory` sont absents du PATH isolé. Certains runs tentent néanmoins
  de les appeler et reviennent aux moyens disponibles après un code 127. Ces échecs d'outils
  restent dans les raws ; ils ne sont pas assimilés à une panne de capture ou à un échec de tâche.
- Des runs P2 exécutent aussi les tests locaux alors que la demande porte sur les logs. La
  comparaison conserve ce travail supplémentaire et les lectures de skills ; elle ne l'impute
  pas automatiquement à A039.a.

Les skills possédées sont présentes, mais leurs dépendances opérationnelles ne sont donc pas
toutes disponibles dans ce profil. Ce lot ne mesure pas le harnais complet du poste. Aucun outil
manquant n'a été installé, aucune configuration n'a été changée entre les conditions pour
compenser ces observations.

Après exécution, les contrôles indépendants confirment les diffs exacts de P1 et N2 et l'absence
de changement de fichier pour P2/N1. Les douze nouveaux contrôles natifs requis sur P1/N1 passent
sous le shell macOS/Node 26.8.1. `git diff --check` passe dans les 24 fixtures. Ces vérifications
ne valent pas pour Linux, Windows ou un autre runtime, et ne remplacent pas la notation du texte.

## Consommation

| Mesure                             | 24 nouveaux runs | Ensemble des 28 runs |
| ---------------------------------- | ---------------: | -------------------: |
| Tokens d'entrée                    |        3 002 984 |            3 468 422 |
| Dont entrée en cache               |        2 597 376 |            3 011 456 |
| Tokens de sortie                   |           28 458 |               32 631 |
| Dont raisonnement, déjà inclus     |            7 161 |                8 310 |
| Total entrée + sortie              |        3 031 442 |            3 501 053 |
| Durée cumulée des processus agents |      18 min 36 s |          22 min 19 s |
| Appels de commande                 |              153 |                  176 |
| Équivalent API indicatif           |         3,23 USD |             3,69 USD |

Les compteurs proviennent des événements terminaux ; les écritures de cache déclarées valent
zéro. Calcul indicatif aux [tarifs Sol standard](https://developers.openai.com/api/docs/pricing)
consultés le 21 septembre 2026 : entrée non cachée à 4 USD/million, cache à 0,40 USD/million,
sortie à 20 USD/million. Les runs ont utilisé ChatGPT : ce n'est pas une facture API. La durée et
le coût du coordinateur, de la préparation et de la revue ne sont pas inclus dans ces compteurs.

L'estimation supplémentaire de 2,70 USD était indicative ; l'observation est supérieure d'environ
20 %. Le quota hebdomadaire partagé passe de 23 % à 26 % utilisés entre les relevés de cette
extension. Ces trois points ne peuvent pas être attribués exclusivement aux runs : autres
activités du compte, orchestration et arrondi restent indissociables.

## Décision traçable et couverture restante

| Unité                         | Action proposée                          | État de preuve                                                    | Réexamen                                                                            |
| ----------------------------- | ---------------------------------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| A039.a                        | Conserver provisoirement, texte inchangé | Signal favorable sur ce corpus Codex ; disposition `inconclusive` | Nouvelle preuve indépendante, autre agent, autre formulation ou régression observée |
| Reste d'A039                  | Aucun changement                         | Non évalué par cette ablation                                     | Lot distinct, sans transfert du résultat de la phrase                               |
| Autres unités de l'inventaire | Aucun changement                         | Aucune nouvelle ablation dans ce lot                              | Priorisation distincte après décision sur la suite                                  |

Couverture : **une sous-unité d'une unité parente sur 205**, quatre scénarios, trois répétitions
par condition A/B et un placebo par scénario, un seul agent/modèle/effort demandé. Aucun résultat
Claude ou Cursor n'est ajouté ; aucune décision définitive sur l'intégralité d'une unité parente
n'est revendiquée. #160 reste ouverte.

Aucune instruction permanente, projection, gate ou configuration n'a été modifiée. Il n'y a donc
pas de retour arrière doctrinal à exécuter. Les preuves sont additives dans la PR #347 ; les
originales restent conservées séparément des copies normalisées destinées au dépôt. Toute future
modification du texte demandera sa propre proposition, ses preuves et la validation prévue.
