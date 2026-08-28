# ADR-018 — OrbStack comme runtime de conteneurs

- **Statut** : accepté
- **Date** : 2024-02
- **Commits** : `4610246`

## Contexte

boot2docker puis docker-machine puis Docker Desktop se sont succédé. Sur macOS
Apple Silicon, Docker Desktop consomme des ressources notables au repos, démarre
lentement et impose des conditions de licence en entreprise.

## Décision

Installer OrbStack à la place de Docker Desktop. L'intégration shell est
conditionnée à la présence effective d'OrbStack, afin que la configuration
Fish reste utilisable sans lui.

## Conséquences

- Démarrage rapide et empreinte réduite au repos ; CLI `docker` inchangée.
- Les installations optionnelles dépendant de Docker échouent si le daemon est absent. Un appelant
  peut demander explicitement `DOCKER_UNAVAILABLE_POLICY=allow-skip`, qui rapporte un résultat
  `skipped` distinct d'une installation vérifiée.
- Dépendance à un produit tiers propriétaire.

## Alternatives écartées

- Docker Desktop : lourd, licence contraignante.
- Colima ou Lima : intégration macOS moins aboutie.
