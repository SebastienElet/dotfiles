# ADR-040 — Skills user dans `harness/`

- **Statut** : accepté
- **Date** : 2026-08

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

- `harness/skills/` contient les skills personnelles installées au niveau user ;
- `.agents/skills/` contient uniquement les skills propres au dépôt ;
- le `Makefile` déploie chaque skill user par un lien feuille vers sa source sous
  `harness/skills/` ;
- `skill-manager` vit dans `harness/skills/`, est installé au niveau user et gère
  les deux collections sans autoriser un même slug dans les deux ;
- Arnes dérive les skills projet de `.agents/skills/` et les installations user
  des déclarations du manifeste.

## Conséquences

- Une skill user n'est plus exposée une seconde fois par le checkout dotfiles.
- Les deux collections ont chacune leur index dérivé et leur source canonique.
- Ajouter une skill user exige de déclarer ses agents cibles et ses liens dans
  le `Makefile` ; une skill projet n'est jamais installée globalement.
- Le déplacement d'une skill entre portées migre simultanément sa source, ses
  projections et ses liens déjà installés.

## Alternatives écartées

- Conserver toutes les sources sous `.agents/skills/` : confond les portées et
  expose deux occurrences des skills déjà installées au niveau user.
- Lier `harness/skills/` en bloc dans le dépôt : recrée une portée projet pour
  toutes les skills personnelles.
- Copier les skills vers chaque agent : introduit plusieurs sources modifiables
  et une synchronisation manuelle.
