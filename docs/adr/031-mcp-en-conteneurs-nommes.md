# ADR-031 — MCP en conteneurs Docker nommés

- **Statut** : accepté
- **Date** : 2026-08
- **Commits** : `425af01`, `a2c71cf`, `508feb4`, `0525f5a`

## Contexte

Les serveurs MCP de récupération web (scrapling, firecrawl, CloakBrowser)
embarquent navigateurs et dépendances Python lourdes, mal adaptées à une
installation directe sur le poste. Enregistrés en `docker run -i --rm … mcp`,
ils créaient un conteneur par session : le commit `0525f5a` constate
« five were still running, the oldest for 12 hours ».

## Décision

Livrer ces MCP par images Docker et n'exécuter qu'un conteneur nommé par
service : un script d'enveloppe démarre le conteneur à la demande
(`docker start` ou `docker run`) puis lance le serveur stdio de chaque session
à l'intérieur. L'enveloppe Scrapling vit sous `tooling/scrapling-mcp` et la
composition Firecrawl sous `harness/firecrawl/compose.yml`. Une escalade par
paliers est inscrite dans les instructions globales : fetch intégré, puis
`stealthy_fetch`, puis CloakBrowser, avec `--idle-timeout` pour l'arrêt
automatique.

## Conséquences

- Un conteneur oublié coûte au plus un conteneur, non un par session.
- Les dépendances lourdes restent isolées du poste.
- Les cibles Docker doivent tolérer l'absence de daemon
  ([ADR-018](018-orbstack-runtime-conteneurs.md),
  [ADR-023](023-ci-lint-et-installation.md)).

## Alternatives écartées

- `docker run --rm` par session : fuite de conteneurs constatée.
- Installation native des serveurs : dépendances navigateur ingérables sur le
  poste.
