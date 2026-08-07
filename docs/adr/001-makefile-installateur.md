# ADR-001 — Makefile idempotent comme installateur de poste

- **Statut** : accepté
- **Date** : 2014-08
- **Commits** : `8411881`, `319529c`, `0777897`, `f9ac76b`

## Contexte

Un poste macOS neuf doit être réinstallé à l'identique sans procédure manuelle,
et l'installation doit pouvoir être rejouée sur un poste déjà configuré sans
tout refaire ni rien casser.

## Décision

Le `Makefile` est l'unique point d'entrée de l'installation. Une cible par
outil, regroupée en sections (`terminal`, `work`, `utils`, `personal`,
`extra`), agrégées par la cible `all`. Les cibles reposent sur des fichiers
sentinelles (le binaire installé, le symlink créé) afin que `make` détermine
lui-même ce qui reste à faire ; les cibles sans fichier déclarent `.PHONY`
individuellement. `install.sh` se limite à amorcer Homebrew puis à appeler
`make`.

## Conséquences

- Réinstallation et mise à jour partielle par la même commande, sans état
  externe à maintenir.
- L'idempotence est une propriété à défendre cible par cible : plusieurs
  correctifs ont porté sur des cibles qui se reconstruisaient à chaque appel.
- Le `Makefile` grossit avec le temps et devient la pièce la plus dense du
  dépôt, d'où l'élagage périodique ([ADR-005](005-elagage-des-cibles.md)).

## Alternatives écartées

- Un script shell impératif : pas de reprise partielle, pas de graphe de
  dépendances.
- Ansible ou un gestionnaire de configuration : la cible `ansible` a existé
  puis a été retirée ; disproportionné pour un poste unique.
