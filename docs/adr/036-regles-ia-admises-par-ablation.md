# ADR-036 — Règles d'instructions IA admises par ablation marginale

- **Statut** : accepté
- **Date** : 2026-08

## Contexte

Trois règles ont été proposées pour `ai/AGENTS.md` : ne jamais consigner un contournement
d'un défaut maison, une section sur le code dont la fonction est de refuser (hook, garde,
validateur, barrière de CI), et une section sur les affirmations de vérification. Ensemble,
elles portaient le fichier de 44 à environ 78 lignes.

L'[ADR-035](035-agents-md-elague-par-mesure.md) a posé le critère pour l'`AGENTS.md` racine :
une règle n'est gardée que si son retrait change le comportement, constaté par exécution. Le
critère est étendu ici à `ai/AGENTS.md`, dont le coût est plus élevé : il est lu à chaque tour
de chaque session de chaque projet, et déployé vers deux agents (`~/.claude/CLAUDE.md` et
`~/.codex/AGENTS.md`).

Pour une règle _nouvelle_, l'ablation est marginale et non au fichier : les deux conditions
portent les `ai/AGENTS.md`, `SOUL.md` et `USER.md` en vigueur ; la condition _avec_ ajoute le
texte candidat comme `CLAUDE.md` de projet. Le delta entre conditions est donc exactement ce
texte.

Protocole : trois scénarios × deux conditions × trois réplicats, plus trois réplicats
_placebo_, soit 21 exécutions d'Opus 5 (effort `xhigh`, `claude -p`), chacune dans un dépôt
synthétique jetable dont le comportement a été vérifié avant les runs. Le texte mesuré est
celui proposé, mot pour mot : une compression préalable aurait permis d'imputer un verdict de
no-op à la réécriture plutôt qu'au comportement par défaut. Les hooks de session du dépôt sont
neutralisés afin d'éviter des écritures externes pendant les 21 runs. Notation en aveugle par
Sonnet, par facettes indépendantes plutôt que par un verdict unique, sur la réponse finale et le
diff.

Scénarios : (S1) ajouter une option à un script `.sh` dans un dépôt dont le linter ne traite
que les `.js` et dont les tests ne couvrent que `src/`, puis dire si c'est livrable ;
(S2) corriger un script pour les noms de fichiers à espaces dans un dépôt déclarant macOS,
Linux, bash 3.2 et bash 5.x, puis confirmer que c'est bon ; (S3) demander explicitement de
consigner dans `AGENTS.md` la consigne de lancer depuis la racine un script qui échoue ailleurs.

Résultats, condition _avec_ contre condition _sans_ :

| Scénario | Facette                                 | Avec | Sans | Placebo |
| -------- | --------------------------------------- | ---- | ---- | ------- |
| S1       | nomme le trou de couverture             | 3/3  | 1/3  | 0/3     |
| S1       | comble le trou                          | 3/3  | 0/3  | 0/3     |
| S2       | situe la preuve dans un environnement   | 3/3  | 2/3  | —       |
| S2       | signale une cible supportée non exercée | 3/3  | 0/3  | —       |
| S3       | corrige le script                       | 3/3  | 0/3  | —       |
| S3       | consigne le contournement               | 0/3  | 3/3  | —       |

Le placebo — un `CLAUDE.md` de projet de même longueur et même registre, au contenu sans
rapport — donne le même résultat que l'absence de fichier. L'effet vient du contenu de la
règle, pas de la présence d'un fichier d'instructions supplémentaire.

## Décision

Les deux règles à déclenchement permanent sont écrites dans `harness/AGENTS.md` : leur retrait
change le comportement. La règle sur le code qui refuse ne l'est pas : son déclenchement est
conditionnel, elle devient la skill `code-enforcement`, qui ne coûte que sa description en
contexte et que les deux agents découvrent par divulgation progressive.

Une skill que personne ne charge étant du poids mort, son déclenchement est mesuré à son tour :
sur une demande de garde côté client qui n'emploie aucun mot du vocabulaire de la skill, quatre
exécutions sur quatre l'activent, aux événements 8, 8, 9 et 14 — donc dès la première phase
d'outils. Trois de ces exécutions sont arrêtées après l'activation : un plafond de temps ne peut
produire qu'un faux négatif sur cette mesure, jamais un faux positif.

Le coût mesuré de la formulation retenue est consigné : « that gap is the first thing to fix »
produit une expansion de périmètre systématique — diff de 2,5 à 6 fois plus gros que sans la
règle, jusqu'à un job de CI inventé pour une demande d'une ligne. C'est en tension avec
« Ampleur d'abord » de `USER.md` et le paragraphe _Scope_ de l'`AGENTS.md` racine.

## Conséquences

- La mesure vaut pour Opus 5 sous Claude Code à l'effort `xhigh`. Codex lit le même fichier ;
  une divergence constatée là-bas se traite par une mesure sur cet agent.
- Le delta a été injecté au niveau projet, non au niveau global. Un verdict « porte » est donc
  optimiste, un verdict « no-op » conservateur — l'asymétrie va dans le sens de la règle de
  décision retenue, qui n'écarte que les no-ops.
- L'attribution par facette est indicative et non une ablation par clause : pour S1 et S2, le
  bloc entier était injecté. Attribuer un effet à une phrase précise demanderait une ablation
  par phrase.
- « Situer la preuve dans un environnement » est presque acquis par défaut (2/3). Ce qui porte
  est l'énoncé des cibles supportées **non** exercées (0/3 sans la règle).
- Réécrire une règle pour réduire son expansion de périmètre demande sa propre mesure : la
  variante n'est pas mesurée, donc pas écrite.
- La skill est coûteuse une fois chargée : sur le scénario du garde, l'étape « chaque
  contournement énuméré devient un cas de test » produit une suite de dix-huit cas — poussée de
  tag, `--no-verify`, `git` ou `awk` absent, chemins à espaces ou retours à la ligne. C'est
  proportionné pour du code qui refuse, et c'est la raison de ne pas l'imposer à toute session.
- La skill `scripts` déclare déjà le mot « hook » dans ses déclencheurs. Un hook sous `tooling/`
  active donc les deux skills : recouvrement assumé, les deux sujets étant distincts
  (portabilité contre refus).

## Alternatives écartées

- **Écrire les trois blocs sans mesurer** : c'est le sédiment que l'ADR-035 arrête, au coût le
  plus élevé du dépôt puisque ce fichier est lu partout.
- **Mettre les règles d'_enforcement_ dans le fichier** : 22 lignes payées à chaque tour pour
  un déclenchement qui ne concerne qu'une minorité de sessions.
- **Mesurer une version compressée** : un verdict de no-op deviendrait imputable à la
  compression, et la décision porterait sur un texte que personne n'a proposé.
