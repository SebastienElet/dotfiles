# ADR-033 — Ne pas utiliser RTK pour réduire les tokens des outils

- **Statut** : accepté
- **Date** : 2026-08
- **Commits** : `833a209` (adoption), `51e16d7`, `1ecef61` (retrait)

## Contexte

RTK avait été installé en 2026-03 : formule Homebrew, hook `PreToolUse`
réécrivant les commandes de l'agent vers les équivalents `rtk`, une quarantaine
d'entrées de permissions et un `~/.claude/RTK.md`. La promesse était une
réduction du coût en tokens des sorties d'outils.

Deux mesures l'ont infirmée, citées dans `1ecef61` :

- Mesure JetBrains sur 86 tâches appariées de SkillsBench : **+7,6 % de coût**
  à faible effort de raisonnement, **+0,1 %** à effort élevé — jamais un gain.
- La métrique interne `rtk gain` note son propre contrefactuel : localement,
  690,5 M des 781,5 M de tokens « économisés » provenaient de `rtk grep` à
  24,5 % de réduction moyenne, c'est-à-dire de sortie brute que l'agent
  n'aurait de toute façon jamais lue en entier.

Par ailleurs, `~/.claude/RTK.md` n'était référencé par rien.

## Décision

Retirer RTK : cible `Makefile`, hook de réécriture `PreToolUse`, entrées de
permissions et fichier d'instructions. Les agents utilisent directement `rg`,
`fd` et les outils natifs ([ADR-015](015-cli-modernes.md)).

## Conséquences

- Un hook de moins dans le chemin de chaque appel d'outil, donc un point de
  défaillance et une source d'écart de comportement en moins.
- La configuration de permissions redevient lisible.
- Une réadoption exigera une mesure appariée sur ce poste, non l'annonce d'un
  gain.

## Alternatives écartées

- Conserver RTK en n'activant que `rtk grep` : c'est précisément la commande
  dont le gain mesuré est un artefact de comptage.
- Garder l'outil sans le hook : le coût d'installation subsiste sans usage.
