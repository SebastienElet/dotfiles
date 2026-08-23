# ADR-038 — Frontières `home/`, `harness/` et `tooling/`

- **Statut** : accepté
- **Date** : 2026-08

## Contexte

La racine mélangeait les chemins imposés par les outils, les fichiers déployés
sous `$HOME`, les instructions partagées entre agents et les exécutables
locaux. Le prochain fichier n'avait donc pas d'emplacement déterministe.

L'[ADR-003](003-deploiement-par-symlinks.md) impose le mécanisme de déploiement,
pas les chemins sources. Les ADR-024 et ADR-027 nommaient en revanche `ai/` ;
leur décision fonctionnelle reste valable, mais cette localisation est
remplacée ici.

## Décision

Adopter trois frontières :

1. `home/` contient les artefacts déployés individuellement sous `$HOME` et
   reproduit leur chemin relatif de destination ;
2. `harness/` contient les instructions et capacités partagées entre les
   agents, dont `AGENTS.md`, `SOUL.md`, `USER.md` et Firecrawl ;
3. `tooling/` contient les applications et exécutables locaux maintenus. Les
   exécutables placés directement sous ce répertoire sont sans extension et
   nommés en kebab-case.

Les chemins imposés par un outil (`.agents/`, `.claude/`, `.codex/`,
`.cursor/`, `.github/`) et les points d'entrée du dépôt (`AGENTS.md`,
`CLAUDE.md`, `Makefile`, `README.md`, `install.sh`) restent à la racine.

Le déploiement normal laisse Make créer chaque destination absente avec
`ln -s`. La recette n'écrase aucune destination existante et ne revalide pas le
contenu d'une cible que Make juge à jour. Aucun lien de compatibilité n'est
conservé dans le dépôt.

## Conséquences

- L'emplacement d'un nouvel artefact se déduit de sa destination et de sa
  responsabilité.
- L'arborescence `home/` rend visible la correspondance avec `$HOME` sans
  changer les destinations utilisées par les outils.
- Un déplacement de source est plus coûteux qu'un simple `git mv`, car tous
  les consommateurs et les liens déjà posés doivent migrer ensemble.
- Les chemins de découverte imposés restent des exceptions visibles à la
  racine plutôt que des copies sous `harness/`.

## Alternatives écartées

- Stow, yadm, chezmoi ou Nix : changement de moteur sans nécessité ; le
  `Makefile` couvre déjà le déploiement.
- Répertoires `hosts/`, `profiles/`, `platforms/` ou `modules/` : catégories
  sans besoin actuel.
- Conservation de liens vers les anciens chemins : masque les consommateurs
  oubliés et pérennise deux sources possibles.
- Modularisation du `Makefile` : indépendante du problème de placement.
