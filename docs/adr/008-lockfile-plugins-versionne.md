# ADR-008 — Lockfiles versionnés et Dependabot pour Bun

- **Statut** : accepté
- **Dates** : 2024-10, 2026-08
- **Commits** : `bef7198`

## Contexte

Un plugin Neovim suivi sur sa branche par défaut peut casser l'éditeur à la
première mise à jour, sans possibilité de revenir à un état connu. Le reste
des dépendances du dépôt souffre du même problème à une échelle plus lente. Le
projet Bun racine dispose également d'un manifeste et d'un lockfile texte
versionnés, exécutés par une CI qui refuse leur désynchronisation.

## Décision

Committer `home/.config/nvim/lazy-lock.json` : chaque plugin est épinglé à un
commit précis, et les mises à jour produisent un commit dédié
(`chore(nvim): update plugins`) généré par le script d'upgrade.

Dependabot propose chaque semaine les mises à jour des dépendances directes du
projet Bun racine. Sa configuration ne porte aucune liste de paquets : les
dépendances directes présentes et futures restent éligibles après une fenêtre
de refroidissement explicite d'au moins trois jours suivant la publication
d'une version. Les propositions restent des pull requests sans fusion
automatique et déclenchent l'installation gelée, les tests Bun et la
vérification TypeScript communes aux autres pull requests. Cette décision ne
couvre aucun autre écosystème ni les mises à jour du runtime Bun.

## Conséquences

- Retour arrière immédiat par `git revert` du commit de lockfile.
- L'historique est dominé en volume par ces commits automatiques — plus de
  trois cents sur douze ans — ce qui nuit à sa lisibilité.
- Un poste neuf installe exactement les versions validées sur le poste
  courant.
- Une nouvelle version attend au moins trois jours avant de devenir éligible à
  une proposition, ce qui laisse apparaître les compromissions détectées
  rapidement au lieu de suivre immédiatement la publication.
- Le service Dependabot ne devient observable qu'après intégration de sa
  configuration dans la branche par défaut ; une validation YAML locale ne
  prouve ni son exécution ni la production conjointe du manifeste et du
  lockfile.

## Alternatives écartées

- Ne pas versionner le lockfile : aucune reproductibilité, aucun retour
  arrière.
- Mises à jour manuelles plugin par plugin : coût sans bénéfice, puisque le
  lockfile permet déjà l'annulation.
- Renovate pour les dépendances Bun : cette intégration n'est pas retenue ; la
  cible d'installation du CLI local n'en fait pas le gestionnaire de mises à
  jour du dépôt.
