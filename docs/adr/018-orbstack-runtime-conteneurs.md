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
- Les cibles Docker doivent tolérer l'absence de daemon — en CI, ou juste
  après une installation neuve — d'où le garde `DOCKER_OR_SKIP`
  ([ADR-023](023-ci-lint-et-installation.md)).
- Dépendance à un produit tiers propriétaire.

## Alternatives écartées

- Docker Desktop : lourd, licence contraignante.
- Colima ou Lima : intégration macOS moins aboutie.
