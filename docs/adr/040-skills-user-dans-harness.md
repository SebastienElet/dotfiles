# ADR-040 — Sources et projections des skills user

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-10-18 — consommation du paquet publié, décisions #546 et #548, lot #549.

## Contexte

Les skills personnelles installées dans les répertoires utilisateur de Claude,
Cursor et Codex avaient leur source dans `.agents/skills/`. Elles étaient donc
aussi découvertes comme skills projet dans ce dépôt. La
[documentation officielle Codex](https://learn.chatgpt.com/docs/build-skills#where-codex-loads-local-skills)
précise que les emplacements dépôt et utilisateur sont chargés séparément et
que deux skills portant le même nom ne sont pas fusionnées : les deux
occurrences peuvent apparaître.

L'ADR-028 reste pertinente pour les skills dont la portée est le dépôt, tandis
que l'ADR-038 place les capacités partagées du harnais sous `harness/`.

## Décision

Séparer les sources par portée :

- les skills génériques proviennent du paquet npm `@sebastienelet/agent-skills`, acquis par Bun
  à version exacte et verrouillé dans `bun.lock` ; leurs sources restent dans leur dépôt autonome ;
- `proof-integrity-review` et `handoff` proviennent respectivement des paquets npm
  `@sebastienelet/proof-integrity-review@0.0.1` et `@sebastienelet/agent-handoff@0.0.2`,
  acquis par Bun à version exacte et verrouillés dans `bun.lock` ;
- `harness/skills/` conserve le compagnon local `herdr-issue-worktree` ;
- `.agents/skills/` contient uniquement les skills propres au dépôt ;
- les tâches du projet Moon `harness` déploient chaque skill user par un lien feuille vers la
  `source` relative au checkout déclarée dans le manifeste Arnes ; en son absence, la source est
  `harness/skills/<slug>` ; pour les capacités externes gérées, les liens ciblent
  `node_modules/@sebastienelet/proof-integrity-review/skills/proof-integrity-review` et
  `node_modules/@sebastienelet/agent-handoff/skills/handoff` ;
- `skill-manager` est consommé depuis le paquet et installé au niveau user ; aucune skill user
  ne doit aussi exister dans la collection projet ;
- Arnes dérive les skills projet de `.agents/skills/` et les installations user
  des déclarations du manifeste.

Pour la capacité externalisée dans #550, la source de `handoff` est le paquet
publié `@sebastienelet/agent-handoff`, installé par Bun à version exacte et
verrouillée. Moon déploie sa skill par liens feuilles vers le paquet ; Arnes
la déclare comme skill externe gérée. Sa source ne reste pas dans `harness/skills/`.

## Conséquences

- Une skill user n'est plus exposée une seconde fois par le checkout dotfiles.
- Le paquet, les compagnons locaux et la collection projet gardent chacun leur index dérivé.
- Ajouter une skill user exige de déclarer ses installations par agent dans le manifeste Arnes ;
  les tâches Moon déploient les liens correspondants. Une skill projet n'est jamais installée globalement.
- Le déplacement d'une skill entre portées migre simultanément sa source, ses
  projections et ses liens déjà installés.
- Les anciens liens et slugs renommés sont retirés manuellement avant le déploiement ; aucune
  destination divergente n'est remplacée automatiquement. Les slugs publiés sont
  `code-review-fix` et `claude-development`.

## Alternatives écartées

- Conserver toutes les sources sous `.agents/skills/` : confond les portées et
  expose deux occurrences des skills déjà installées au niveau user.
- Lier `harness/skills/` en bloc dans le dépôt : recrée une portée projet pour
  toutes les skills personnelles.
- Copier les skills vers chaque agent : introduit plusieurs sources modifiables
  et une synchronisation manuelle.
