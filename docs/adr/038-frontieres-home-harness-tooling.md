# ADR-038 — Frontières `home/`, `harness/` et `tooling/`

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-09-07

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
   agents, dont `AGENTS.md`, `SOUL.md` et `USER.md` ;
3. `tooling/` contient les applications et exécutables locaux maintenus. Les
   exécutables placés directement sous ce répertoire sont sans extension et
   nommés en kebab-case.

Les chemins imposés par un outil (`.agents/`, `.claude/`, `.codex/`,
`.cursor/`, `.github/`) et les points d'entrée du dépôt (`AGENTS.md`,
`CLAUDE.md`, `Makefile`, `README.md`, `install.sh`) restent à la racine.

Conformément à l'[ADR-003](003-deploiement-par-symlinks.md), le déploiement par
Moon ou, pendant la transition, par Make crée une destination absente, conserve
le lien attendu et refuse une destination divergente. Les configurations
spécifiques conservent les comportements explicités par l’ADR-003. Aucun lien
de compatibilité n’est ajouté aux chemins sources.

Les manifestes Moon de ces répertoires décrivent leurs tâches ; ils ne sont pas
déployés sous `$HOME`. `home/moon.yml` porte les déploiements, `harness/moon.yml`
les capacités des agents et les projets de `tooling/` les outils et oracles.

## Conséquences

- L'emplacement d'un nouvel artefact se déduit de sa destination et de sa
  responsabilité.
- L'arborescence `home/` rend visible la correspondance avec `$HOME` sans
  changer les destinations utilisées par les outils.
- Un déplacement de source impose d'aligner les consommateurs versionnés, puis
  de reconstruire explicitement les destinations concernées avant le déploiement
  voulu ; le nettoyage existant n’est pas une remise à zéro générale.
- Les chemins de découverte imposés restent des exceptions visibles à la
  racine plutôt que des copies sous `harness/`.

## Alternatives écartées

- Stow, yadm, chezmoi ou Nix : ajout d'un autre moteur sans nécessité pour
  les déploiements portés par Moon.
- Répertoires `hosts/`, `profiles/`, `platforms/` ou `modules/` : catégories
  sans besoin actuel.
- Conservation de liens vers les anciens chemins : masque les consommateurs
  oubliés et pérennise deux sources possibles.
- Modularisation du `Makefile` : indépendante du problème de placement.
