# ADR-005 — Élagage périodique des cibles inutilisées

- **Statut** : accepté
- **Date** : 2025-12
- **Commits** : `0d6d00c`, `dad06aa`, `0777897`

## Contexte

Onze ans d'ajouts avaient laissé dans le `Makefile` des dizaines de cibles
correspondant à des outils abandonnés (`vagrant`, `virtualbox`, `travis`,
`youtube-dl`, `postman`, `ncdu`…). Chacune allonge la durée d'installation
d'un poste neuf et le temps de la CI, pour un outil qui ne sera pas utilisé.

## Décision

Retirer les cibles dès que l'outil n'est plus utilisé, plutôt que de les
conserver « au cas où ». L'historique git reste la mémoire : une cible
supprimée se restaure par `git revert`.

## Conséquences

- Installation d'un poste neuf plus rapide et représentative de l'usage réel.
- Une réinstallation ancienne n'est plus reproductible telle quelle depuis
  `main` ; il faut remonter dans l'historique.
- L'élagage se fait par vagues plutôt qu'en continu, ce qui produit des séries
  de commits `refactor(makefile)` peu lisibles isolément.

## Alternatives écartées

- Cible `deprecated` conservant les anciens outils : maintient le coût
  d'installation sans bénéfice.
