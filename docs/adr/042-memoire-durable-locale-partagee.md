# ADR-042 — Mémoire durable locale partagée

- **Statut** : accepté
- **Date** : 2026-08

## Contexte

La spécification approuvée de la PR 249 décrit une mémoire durable, locale et partagée entre
Codex, Claude Code et Cursor. Les décisions structurelles qui la rendent sûre et vérifiable doivent
cependant devenir une autorité en vigueur dans `docs/adr/` avant l'implémentation.

Les ADR-025, ADR-036, ADR-038, ADR-040 et ADR-041 imposent respectivement une source unique de
règles, des instructions admises par mesure, des frontières explicites, des skills user sous
`harness/` et Arnes pour l'automatisation qui porte un état durable. Les contrats officiels de hooks
vérifiés le 2026-08-28 donnent une surface synchrone `UserPromptSubmit` avec `additionalContext`
pour Codex et Claude, mais aucune injection query-specific native avant modèle pour Cursor.

## Décision

Les fichiers YAML sous `~/.local/share/agent-memory/`, hors Git, sont l'autorité humaine unique des
entrées. L'index et le cache sont dérivés, supprimables et reconstruisibles. Arnes est l'unique
frontière Rust : il possède le schéma, l'admission, les sources, le store, l'index, les oracles, le
retrieval et les adaptateurs ; aucun état généré propre à un agent, notamment
`~/.codex/memories/`, n'est édité.

La clé projet est `project_<sha256(realpath(git-common-dir))>`. Elle converge entre worktrees ; deux
clones et un dépôt déplacé restent distincts. Une admission projet est refusée hors Git, si le
résultat est ambigu, non absolu ou impossible à canonicaliser. L'identifiant est déterministe :
`mem_<24 premiers hex de sha256(schema_version, kind, scope.key, statement normalisé)>`. Un même
document canonique retourne `duplicate` ; une même identité associée à un contenu différent retourne
`conflict`.

L'écriture suit l'ordre verrou global, préparation YAML et index, rename YAML, fsync du répertoire,
rename index, puis fsync. Ainsi, aucun index ne peut pointer vers un YAML absent ; après un crash
postérieur au YAML, seul un index périmé est reconstruit au retrieval.

L'oracle automatisé `source-fingerprint` compare toutes les empreintes de preuve. Il retourne
`valid`, `invalid` ou `unavailable`; `invalid` produit exclusivement la transition
`invalidated`. Les transitions métier `achieved`, `abandoned`, `superseded`, `resolved` et
`confirmed` exigent une conclusion humaine `valid`, explicitement typée pour le `kind`. Une entrée
ne transite qu'une fois de `active` vers un statut terminal et n'est jamais réactivée.

Codex et Claude reçoivent le retrieval par hooks synchrones `UserPromptSubmit` et
`additionalContext`. Cursor reçoit une règle user `alwaysApply` minimale et la skill partagée ; leur
effet est mesuré en `3/3` processus frais avant toute promesse d'automatisation. Une règle globale
partagée ne peut être ajoutée sans mesure d'ablation conforme à l'ADR-036.

Le refus sensible est déterministe pour les formes nommées et seulement advisory pour un prompt
privé ou transcript non marqué : aucune détection universelle n'est revendiquée. Une `official-url`
est HTTPS, sans credentials, IP littérale ni fragment, avec au plus cinq redirections HTTPS, 1 Mio,
une connexion de 5 s et une durée totale de 15 s. L'officialité d'un domaine est une décision
utilisateur persistée comme `user-decision`, jamais une inférence du fetch.

Seul un verdict `valid` âgé de strictement moins de 48 h est consommable. À 48 h, il est expiré ;
toute modification locale observable invalide immédiatement le cache.

## Conséquences

- Le comportement de mémoire reste local, auditable et commun aux trois agents sans versionner les
  données utilisateur ou projet.
- Les adaptateurs ne dupliquent ni le schéma, ni la politique de confidentialité, ni les
  transitions ; Cursor reste une capacité comportementale mesurée, non une garantie native.
- Les limites de détection sensible et d'officialité URL sont explicites et leurs échecs ne créent
  ni injection ni écriture implicite.
- La frontière Arnes concentre l'état durable et les échecs associés dans un binaire déjà installé
  par le harnais.

## Alternatives écartées

- Un second binaire ou crate : duplique racines, installation, hooks et diagnostics sans frontière
  métier indépendante.
- SQLite dès cette phase : moins lisible pour l'audit et non justifié avant mesure du processus YAML.
- Un service MCP avec embeddings : daemon, dépendances, coût et surface de confidentialité sans
  besoin de recherche sémantique établi.
- Une règle globale partagée non mesurée : contredit l'admission par ablation de l'ADR-036.
- Un wrapper Cursor limité au CLI : ne prouve pas le retrieval avant l'influence sur le modèle.
- L'écriture dans `~/.codex/memories/` : état généré propre à Codex, non partagé avec Claude et
  Cursor.
