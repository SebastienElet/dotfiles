# ADR-003 — Déploiement de la configuration par symlinks

- **Statut** : accepté
- **Date** : 2014-10
- **Commits** : `9160770`, `0fb9e89`, `1772db9`

## Contexte

Les fichiers de configuration doivent rester versionnés dans `~/.dotfiles`
tout en étant lus depuis leur emplacement attendu par chaque outil. Copier les
fichiers imposerait une resynchronisation après chaque modification.

## Décision

Le `Makefile` crée des symlinks depuis `$(DOTFILES_PATH)/home/…`, qui reproduit
le chemin relatif à `$HOME`, vers l'emplacement attendu sous `~/…`. Les
instructions partagées et les exécutables suivent le même mécanisme depuis
`harness/` et `tooling/` ([ADR-038](038-frontieres-home-harness-tooling.md)).
L'édition se fait dans le dépôt, l'effet est immédiat.

Chaque artefact statique lié est une cible fichier ordinaire du `Makefile`,
avec sa source comme dépendance et son répertoire parent comme dépendance
d'ordre. Make décide si la recette doit s'exécuter ; elle vérifie seulement que
la destination est absente avant `ln -s`. Les exécutables générés sont liés par
leur cible de build après compilation. Le contenu d'un lien existant n'est pas
revalidé à chaque installation ; sa réparation explicite est suivie par
l'issue #152.

Une exception documentée : `~/.codex/AGENTS.md` est **assemblé** par
concaténation à l'installation. Le commit `1772db9` en donne la raison —
« Codex loads AGENTS.md but ignores its @import directives, verified with a
control prompt » —, la concaténation étant alors le seul mécanisme disponible.

## Conséquences

- `tooling/upgrade` relance `make all` après son `git pull`, afin de poser les
  nouveaux liens et de régénérer les fichiers assemblés.
- Un fichier assemblé se périme silencieusement : il faut relancer la cible
  après modification des sources.
- Les liens déjà corrects restent inchangés ; la seconde installation ne doit
  ni les recréer ni réinstaller un outil déjà présent.
- Un fichier ou lien inattendu peut satisfaire la cible selon sa date ; une
  cible obsolète fait échouer `ln -s` sans écraser la destination.

## Alternatives écartées

- `stow` ou `chezmoi` : une dépendance de plus pour ce que quelques règles
  `make` couvrent déjà.
- Copie des fichiers à l'installation : divergence garantie entre dépôt et
  poste.
- Revalidation forcée par un helper : logique et tests disproportionnés pour
  une course externe au graphe de Make.
