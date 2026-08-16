# ADR-011 — Starship comme prompt unique

- **Statut** : accepté
- **Date** : 2025-01
- **Commits** : `2fd5580`

## Contexte

Le prompt zsh maison, construit puis ajusté entre 2016 et 2018, était lié au
shell et devait être réécrit pour Fish. Il portait aussi ses propres appels
externes, coûteux à chaque affichage.

## Décision

Adopter Starship, configuré par un unique `home/.config/starship.toml`
versionné et indépendant du shell.

## Conséquences

- Un changement de shell ne remet plus le prompt en cause.
- Le prompt affiche contexte git, versions de runtimes et durée d'exécution
  sans code maison.
- Dépendance à un binaire supplémentaire dans le chemin critique de chaque
  invite.

## Alternatives écartées

- Prompt maison porté vers Fish : réécriture à recommencer au prochain
  changement de shell.
- Powerlevel10k : lié à zsh, donc écarté par [ADR-010](010-fish-shell-par-defaut.md).
