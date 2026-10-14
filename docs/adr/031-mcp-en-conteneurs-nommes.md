# ADR-031 — MCP en conteneurs Docker nommés

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-10-14, [décision de parcours](https://github.com/SebastienElet/dotfiles/issues/502#issuecomment-6168820713)
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
CloakBrowser est démarré à la demande comme navigateur CDP. Pour les pages
publiques non sensibles, le parcours devient : fetch intégré, puis recette
Agent-Reach/Jina Reader, puis navigateur local, puis navigateur local furtif
en cas de blocage anti-bot identifié, puis CloakBrowser. Les instructions
globales fixent l'admissibilité des données, les budgets et les échecs.
Jina fournit du texte, sans équivalence générale avec un navigateur.

Scrapling reste déployé pour les recours locaux (`fetch`, `stealthy_fetch` et
CDP vers CloakBrowser), avec `--idle-timeout` pour l'arrêt automatique. Les
lectures privées ou authentifiées restent locales, sur autorisation explicite
du compte/profil pour les sessions ; aucun secret ni session n'est transmis
au Reader hébergé. Aucun abandon de capacité n'est accepté. Le retrait de
Scrapling exige une preuve distincte de chaque besoin conservé sans Scrapling,
y compris JavaScript, anti-bot, sessions, privé/local et CloakBrowser/CDP.

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
