# ADR-023 — CI de lint et test d'installation

- **Statut** : accepté
- **Date** : 2026-01
- **Commits** : `62a7644` (#41), `8d658d4`

## Contexte

Une erreur de syntaxe dans un fichier Fish ou Lua ne se manifeste qu'à
l'ouverture du shell ou de l'éditeur, souvent après le push. Une cible
`Makefile` cassée ne se découvre qu'à la réinstallation d'un poste, c'est-à-dire
au pire moment.

## Décision

Cinq workflows GitHub Actions. `lint.yml` vérifie la syntaxe et le formatage
Fish sur macOS et Linux, le Lua Neovim, les scripts shell et le chargement de
la configuration CSpell avec son dictionnaire. `test-fish.yml` exécute les
tests Fish sur les deux systèmes lorsque les fichiers couverts changent.
`test-rust.yml` vérifie le formatage, exécute Clippy et les tests de
`tooling/daily-routine` sur macOS. `test-deployment.yml` attaque la migration
des liens sur macOS et Linux. `test.yml` exécute deux fois l'installation
complète sur un runner macOS, apps payantes exclues (`SKIP_PAID_APPS`) et
cibles Docker ignorées en l'absence de daemon (`DOCKER_OR_SKIP`).

## Conséquences

- Le `Makefile` est vérifié en continu, pas seulement lors d'une
  réinstallation.
- Le test d'installation est long, d'où les timeouts relevés à plusieurs
  reprises dans l'historique.
- La CI impose des garde-fous au `Makefile` (approbation des taps, absence de
  daemon Docker) qui n'ont pas d'utilité sur un poste réel.

## Alternatives écartées

- Vérification manuelle avant push : oubliée dès que le changement paraît
  anodin.
- Test d'installation dans une VM locale : plus lent, non déclenché
  automatiquement.
