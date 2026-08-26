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
`tooling/daily-routine` sur macOS. `test-deployment.yml` attaque le déploiement
des liens sur macOS et Linux. Sur `main`, `test.yml` exécute deux fois
l'installation par défaut sur un runner macOS. Sur une pull request,
`ci-smoke-targets` repère les blocs `.PHONY` modifiés et les exécute sur des
runners macOS isolés et parallèles. Une sélection ambiguë ou un changement de
l'infrastructure d'installation retombe sur `all`. Les apps payantes sont
exclues (`SKIP_PAID_APPS`). Le job d'installation autorise explicitement un
résultat Docker `skipped` en l'absence de daemon ; lorsque Docker est disponible,
chaque cible vérifie l'artefact qu'elle promet avant de réussir.

## Conséquences

- Le chemin par défaut reste vérifié après chaque intégration dans `main` ; la
  sélection des pull requests n'est qu'une accélération anticipée.
- Le temps d'une pull request devient proportionnel à ses cibles modifiées,
  avec une durée murale bornée par la cible la plus lente lorsqu'il y en a
  plusieurs.
- La CI impose des garde-fous au `Makefile` (approbation des taps, absence de
  daemon Docker) qui n'ont pas d'utilité sur un poste réel.

## Alternatives écartées

- Vérification manuelle avant push : oubliée dès que le changement paraît
  anodin.
- Test d'installation dans une VM locale : plus lent, non déclenché
  automatiquement.
- `make -j` sur un runner unique : Homebrew, `/Applications` et `$HOME`
  constituent un état partagé impropre à des installations concurrentes.
