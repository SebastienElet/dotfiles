# Plan du refactor du setup du hook Claude

**Objectif :** réconcilier les hooks de harness depuis une source déclarative sans multiplier les
writers des configurations natives.

**Architecture :** le manifeste déclare les capacités, Make orchestre l'installation et
`arnes setup hooks` réconcilie atomiquement les formats Claude, Codex et Cursor.

### 1. Réduire le test aux garanties utiles

Fichier : `tooling/makefile-test`

- Couvrir l'appel `setup hooks` sans politique dupliquée dans Make.
- Couvrir le déploiement des exécutables stables depuis un répertoire extérieur au dépôt.
- Conserver les garanties de mutation JSON et de concurrence dans les tests Arnes.

### 2. Déclarer et réconcilier les hooks

Fichiers : `home/.arnes.yaml`, le domaine `tooling/arnes/src/hooks` et ses tests

- Valider les capacités fermées et leurs installations dans le manifeste.
- Ajouter `arnes setup hooks --agent <agent>` et garder `measure hook` au runtime.
- Réconcilier mesure et handoff dans une même transformation par adapter.
- Réutiliser l'échange atomique avec comparaison du snapshot pour ne perdre aucune écriture
  concurrente.
- Préserver les données étrangères, retirer les capacités non déclarées et migrer les anciens
  chemins.
- Exécuter le test jusqu'au succès.

### 3. Vérifier les barrières concernées

Fichiers : Arnes, `Makefile` et `.github/workflows/lint.yml`.

- Exécuter le formatage, Clippy et les tests Arnes.
- Exécuter `tooling/agent-handoff-test`, `tooling/makefile-test` et les tests Arnes sur macOS.
- Vérifier que le lint CI découvre les scripts imbriqués, le dry-run Make, le YAML et
  `git diff --check`.
- Auto-relire le diff, committer, pousser puis surveiller les checks de la PR.
