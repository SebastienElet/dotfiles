# ADR-026 — Agents CLI plutôt que Copilot

- **Statut** : accepté
- **Date** : 2025-10
- **Commits** : `ce71590`

## Contexte

GitHub Copilot, intégré à Neovim depuis 2021 et complété par ses alias CLI en
2024, se limite à la complétion dans le tampon courant. Les agents en ligne de
commande apparus depuis lisent le dépôt, exécutent des commandes et
travaillent sur plusieurs fichiers.

## Décision

Retirer Copilot, de Neovim comme du shell, et travailler avec Claude Code,
Codex et Cursor, installés et mis à jour par le `Makefile`, avec leurs alias
Fish (`c`, `co`).

## Conséquences

- Un seul mode d'interaction avec l'assistance, hors de l'éditeur.
- L'éditeur redevient un éditeur, sans complétion propriétaire dans le tampon.
- Trois agents à installer, configurer et instruire, d'où
  [ADR-024](024-instructions-ia-versionnees.md) et
  [ADR-025](025-agents-md-source-unique.md).

## Alternatives écartées

- Conserver Copilot en complément : deux abonnements et deux modèles
  d'interaction pour un bénéfice marginal.
