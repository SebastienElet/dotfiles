# ADR-008 — `lazy-lock.json` versionné et Renovate

- **Statut** : accepté
- **Date** : 2024-10
- **Commits** : `bef7198`

## Contexte

Un plugin Neovim suivi sur sa branche par défaut peut casser l'éditeur à la
première mise à jour, sans possibilité de revenir à un état connu. Le reste
des dépendances du dépôt souffre du même problème à une échelle plus lente.

## Décision

Committer `nvim/lazy-lock.json` : chaque plugin est épinglé à un commit précis,
et les mises à jour produisent un commit dédié (`chore(nvim): update plugins`)
généré par le script d'upgrade. Renovate prend en charge les dépendances
restantes du dépôt.

## Conséquences

- Retour arrière immédiat par `git revert` du commit de lockfile.
- L'historique est dominé en volume par ces commits automatiques — plus de
  trois cents sur douze ans — ce qui nuit à sa lisibilité.
- Un poste neuf installe exactement les versions validées sur le poste
  courant.

## Alternatives écartées

- Ne pas versionner le lockfile : aucune reproductibilité, aucun retour
  arrière.
- Mises à jour manuelles plugin par plugin : coût sans bénéfice, puisque le
  lockfile permet déjà l'annulation.
