# ADR-043 — Télémétrie Arnes minimale et bornée

- **Statut** : accepté
- **Date** : 2026-09

## Contexte

La collecte Arnes initiale conserve un payload expurgé par événement. En dix-huit jours, les artifacts
de hooks ont atteint environ 1,5 Go alloués, alors qu’aucun outcome n’a été enregistré. La présence
d’un hook, d’un événement terminal ou d’une réponse finale ne juge pas le résultat de la tâche.

## Décision

La collecte v2 ne persiste que l’identité non sensible du run, le contexte de comparaison observable
et un flux compact d’événements horodatés. Prompts, réponses, transcripts, chemins locaux, identifiants
de session, payloads et artifacts de hooks sont exclus.

Un outcome est une déclaration structurée séparée, associée à un oracle nommé. `Stop`, une réponse
finale, une fin de session ou le statut natif d’un agent restent des observations et ne créent jamais
un outcome.

Les runs v2 expirent soixante jours après leur dernier événement ou outcome observable. La
maintenance est opportuniste, au plus quotidienne, verrouillée et observable. Les runs v1 sont
préservés tant qu’une migration explicite n’a pas construit et validé un store voisin.

Les agrégats restent séparés par agent et signalent les valeurs indisponibles. Ils couvrent au minimum
jugeabilité, `pass` déclarés, événements, appels d’outils, latence observable et volume du store.

## Conséquences

- Le volume persisté par événement ne dépend plus de la taille du payload remis par l’agent.
- Le store ne contient plus de contenu de conversation nouvellement collecté.
- Les données v2 ont une durée de vie explicite ; les données historiques ne sont pas supprimées par
  l’installation.
- Une configuration de hook prouve seulement une configuration ; seule une capture stockée prouve
  une activité observée par Arnes, pas l’exécution réussie des autres hooks.
- La validité des comparaisons reste limitée aux agents, modèles, systèmes et versions réellement
  observés.

## Alternatives écartées

- Déduire la réussite de `Stop`, d’une réponse ou d’un statut natif : confond activité et outcome.
- Conserver les payloads après expurgation : surface privée et coût sans utilisation d’évaluation
  démontrée.
- Purger automatiquement les runs v1 : perte irréversible avant validation de migration.
- Installer un framework d’évaluation : aucun besoin du périmètre ne l’exige.
