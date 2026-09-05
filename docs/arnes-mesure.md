# Mesure locale des exécutions Arnes

`arnes measure list --without-result` limite la restitution aux exécutions dont l’historique ne
contient aucun résultat structuré. `--format json` fournit la même observation sous forme
structurée.

Le rapport indique une unique référence `reported_at_ms`. Pour chaque exécution, il expose
`run_id`, `agent`, `started_at_ms`, `last_event`, `last_event_at_ms`,
`start_to_last_event_ms` et `silence_ms`. Les durées sont exprimées en millisecondes. La sortie
humaine emploie `unavailable` et la sortie JSON `null` lorsqu’un événement ou une durée ne peut pas
être jugé. Ces valeurs restent distinctes de zéro.

Les exécutions sont ordonnées par `silence_ms` décroissant ; celles dont la durée est indisponible
apparaissent ensuite. Un journal d’événements absent ou vide produit des valeurs indisponibles. Un
enregistrement présent mais invalide reste un échec de mesure visible.

Le silence mesure uniquement l’intervalle observable entre le dernier événement collecté et
l’heure du rapport. Il ne prouve ni un blocage, ni une progression, ni un temps actif, ni que le
processus est encore vivant ou arrêté. La commande n’applique aucun seuil et ne modifie,
n’interrompt ou ne relance aucune exécution.

Cette vue ciblée n’affiche ni prompt, ni contenu utilisateur, ni dépôt, ni secret, ni chemin local.

## Outcome explicite

Un run n’est jugeable qu’après une déclaration séparée liée à un oracle nommé :

```sh
arnes measure outcome RUN_ID --status pass --oracle cargo-test
arnes measure outcome RUN_ID --status fail --oracle cargo-test
arnes measure outcome RUN_ID --status unjudgeable --reason missing-oracle
```

Le même enregistrement est rejouable sans écriture. Un changement est refusé, sauf ajout explicite
de `--replace`, qui conserve l’historique. `Stop`, une réponse finale, une session terminée et un
statut natif ne créent jamais d’outcome. Arnes garantit la structure et l’historique de la
déclaration ; il n’exécute ni ne certifie l’oracle externe nommé.

## Rapport et rétention

`arnes measure report --format json` restitue globalement et par agent les runs, leur jugeabilité,
les `pass` déclarés parmi les runs jugeables, les événements, les appels d’outils, la latence
observable et les octets logiques et alloués. Une valeur sans dénominateur ou sans horodatages cohérents vaut
`null` ; la présence d’une configuration Cursor, Claude Code ou Codex ne compte pas comme une
exécution observée.

Les runs v2 expirent après soixante jours depuis leur dernier événement cohérent ou outcome. Le sweep est
opportuniste, au plus quotidien, sérialisé avec les lecteurs et écrivains et publie son intention
avant suppression puis son résultat dans `retention.json`. Une incohérence d’horodatage ou de chemin
conserve le run, marque le sweep en échec et n’empêche pas la collecte courante. Les runs v1 ne sont
jamais purgés automatiquement.
