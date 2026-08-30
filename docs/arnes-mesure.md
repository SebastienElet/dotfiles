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
