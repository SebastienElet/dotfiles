# ADR-031 — MCP en conteneurs Docker nommés

- **Statut** : accepté
- **Date** : 2026-08
- **Commits** : `425af01`, `a2c71cf`, `508feb4`, `0525f5a`

## Contexte

Le MCP Scrapling et le navigateur d'escalade CloakBrowser embarquent des
navigateurs et dépendances Python lourdes, mal adaptées à une installation
directe sur le poste. L'ancien lancement de Scrapling par
`docker run -i --rm … mcp` créait un conteneur par session : le commit `0525f5a` constate
« five were still running, the oldest for 12 hours ».

## Décision

Livrer ces capacités par images Docker et n'exécuter qu'un conteneur nommé par
service. L'enveloppe `tooling/scrapling-mcp` démarre le conteneur Scrapling à
la demande puis lance le serveur stdio de chaque session à l'intérieur.
CloakBrowser est démarré à la demande comme navigateur CDP. Une escalade par
paliers est inscrite dans les instructions globales : fetch intégré, puis
Scrapling `fetch`, puis `stealthy_fetch`, puis CloakBrowser, avec
`--idle-timeout` pour l'arrêt automatique.

## Conséquences

- Un conteneur oublié coûte au plus un conteneur, non un par session.
- Les dépendances lourdes restent isolées du poste.
- Les cibles Docker doivent tolérer l'absence de daemon avec un résultat
  `skipped` explicite, décidé par la politique de l'appelant, et ne rapporter
  `verified` qu'après leur oracle Docker
  ([ADR-018](018-orbstack-runtime-conteneurs.md),
  [ADR-023](023-ci-lint-et-installation.md)).

## Alternatives écartées

- `docker run --rm` par session : fuite de conteneurs constatée.
- Installation native des serveurs : dépendances navigateur ingérables sur le
  poste.
