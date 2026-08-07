# ADR-009 — Lint et format par LSP natif et conform

- **Statut** : accepté
- **Date** : 2026-07
- **Commits** : `f3a547a`

## Contexte

La chaîne de qualité dans l'éditeur a traversé quatre générations : jshint et
jscs, puis ESLint (2015-03), syntastic puis ALE (2019-04), coc.nvim
(2019-11). Chacune apportait son propre gestionnaire d'extensions, en doublon
avec celui des plugins Neovim.

## Décision

S'appuyer sur le LSP natif de Neovim pour le diagnostic et sur `conform.nvim`
pour le formatage, tous deux fournis par les extras LazyVim. Les outils
proviennent de Mason. Depuis 2026-07, oxlint et oxfmt sont configurés en
parallèle d'ESLint et Prettier, chaque projet imposant sa chaîne.

## Conséquences

- Un seul mécanisme d'extension, celui de Neovim, plus de gestionnaire
  parallèle.
- La configuration du projet prime sur celle de l'éditeur : plusieurs commits
  retirent des formateurs codés en dur au profit du Prettier local.
- Deux chaînes de lint coexistent pendant la transition vers oxlint.

## Alternatives écartées

- coc.nvim : écosystème d'extensions séparé, redondant avec le LSP natif.
- ALE et syntastic : antérieurs au LSP, diagnostic moins riche.
