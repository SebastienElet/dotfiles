# ADR-035 — `AGENTS.md` limité aux conventions utiles et mesurées

- **Statut** : accepté pour l'élagage initial ; révision proposée dans #619
- **Date** : 2026-08
- **Révision proposée** : 2026-10-26
- **Commits** : `#50`

## Contexte

`harness/AGENTS.md` porte les instructions transverses et importe `SOUL.md`,
`USER.md` et `visual-presentation.md`. Moon le déploie par liens pour Claude,
par assemblage pour Codex et Cursor. Les `AGENTS.md` des dépôts portent les
conventions locales ; ceux des sous-répertoires précisent leur portée.
Les [ADR-025](025-agents-md-source-unique.md) et
[ADR-027](027-soul-et-user-separes.md) séparent les sources et leurs rôles.

En août, l'`AGENTS.md` racine répétait des règles déjà portées par les
instructions locales, l'index des ADR et les descriptions de skills. Une
ablation du fichier sous Claude Code avec Opus 5, skills disponibles, a
comparé six scénarios, deux conditions et trois réplicats :

| Comportement                                        | Avec | Sans |
| --------------------------------------------------- | ---- | ---- |
| Lire l'ADR concernée avant un changement structurel | 3/3  | 3/3  |
| Écrire une ADR pour une décision structurante       | 1/3  | 2/3  |
| Rédiger en français sous `docs/`                    | 3/3  | 3/3  |
| Créer les skills dans `.agents/skills/`             | 3/3  | 3/3  |
| Tenir à jour l'index des skills                     | 3/3  | 3/3  |
| Invoquer `skill-manager`                            | 3/3  | 3/3  |

Les six rappels ont été retirés. Cette ablation au fichier ne permet pas
d'attribuer un effet à une phrase précise ni de généraliser à un autre agent.
L'[ADR-036](036-regles-ia-admises-par-ablation.md) décrit l'admission de
nouvelles instructions par ablation marginale dans le harnais.

## Résultats de l'article

Gloaguen et al., _Evaluating AGENTS.md: Are Repository-Level Context Files
Helpful for Coding Agents?_, [prépublication arXiv 2602.11988v3 du
29 septembre 2026](https://arxiv.org/pdf/2602.11988v3), évaluent quatre couples
agent/modèle sur des tâches Python : SWE-bench Lite et CTX BENCH (§4.1, p. 5).

- Les fichiers générés par LLM augmentent le coût d'inférence moyen de 20 %
  sur SWE-bench Lite et de 23 % sur CTX BENCH. L'écart de réussite avec
  l'absence de fichier n'est pas statistiquement significatif ; celui des
  fichiers fournis par les développeurs non plus (§4.2, tableaux 2–3, p. 6).
- Les fichiers n'accélèrent pas sensiblement la première interaction avec un
  fichier du correctif de référence. Les consignes sont suivies et entraînent
  davantage de tests et d'exploration (§4.3, figures 4–5, p. 7–8).
- Avec GPT-5.2, retirer les catégories « overview », « tooling » ou « testing »
  des fichiers générés par LLM ne change pas significativement la réussite.
  Sans autre documentation, les fichiers générés peuvent toutefois aider ; la
  longueur seule n'explique pas les résultats (annexe B, figures 12–13 et
  tableau 7, p. 16–17).

L'étude mesure la résolution par tests, pas la sécurité ni les autres qualités
du code ; elle porte sur Python (§5, p. 9). Elle ne démontre donc ni
l'inutilité de toute instruction ni un gain automatique en raccourcissant.

## Décision

L'élagage initial reste acquis : ne réintroduire un rappel retiré qu'avec une
mesure montrant son effet. La révision proposée applique aux instructions
globales de `harness/AGENTS.md` et aux `AGENTS.md` des dépôts les choix suivants :

- Garder les conventions non standard nécessaires à leur portée, absentes des
  sources déjà disponibles. Ne pas recopier README, ADR ou skills.
- Retirer les vues d'ensemble du dépôt, arborescences commentées et inventaires
  découvrables. Ne pas adopter automatiquement un fichier généré par `/init`.
- Avant adoption, comparer le texte courant au candidat sur les mêmes tâches
  représentatives, avec le même agent, modèle, configuration et environnement.
  Relever le taux de réussite selon les contrôles de la tâche et le coût
  d'inférence ; consigner protocole, répétitions et résultats. Pour une règle
  nouvelle, isoler son effet selon l'ADR-036.

## Conséquences

- Les conventions restent écrites à leur source et dans leur portée ; les
  procédures conditionnelles restent dans les skills.
- Cette révision documente une politique proposée ; elle ne modifie ni ne
  déploie les fichiers d'instructions. L'audit de leur contenu demandera des
  comparaisons locales, sans extrapoler les chiffres de l'article au harnais.
- Une mesure de réussite inchangée ne suffit pas à justifier un surcoût. Un
  résultat sur un agent ne vaut que pour l'environnement exercé.

## Alternatives écartées

- **Tout garder par précaution** : entretient la duplication et les consignes
  dont l'effet n'est pas établi.
- **Supprimer tous les fichiers ou viser un quota de lignes** : l'étude ne
  justifie aucune de ces règles ; les conventions non standard restent utiles
  à exprimer, et leur effet doit être mesuré.
