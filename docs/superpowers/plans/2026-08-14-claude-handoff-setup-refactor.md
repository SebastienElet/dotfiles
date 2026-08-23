# Plan du refactor du setup du hook Claude

**Objectif :** installer l'entrée Stop de handoff sans multiplier les writers de la configuration
Claude.

**Architecture :** le Makefile délègue directement à Arnes l'installation atomique des hooks de
mesure et de handoff. Arnes reste le seul applicatif qui lit, valide et écrit la configuration
Claude.

### 1. Réduire le test aux garanties utiles

Fichier : `tooling/makefile-test`

- Couvrir l'appel exact d'Arnes sans intermédiaire Shell.
- Couvrir la résolution des chemins depuis un répertoire extérieur au dépôt.
- Conserver les garanties de mutation JSON et de concurrence dans les tests Arnes.

### 2. Intégrer le setup à Arnes

Fichiers : `tooling/arnes/src/measure/install.rs` et ses tests

- Valider les chemins absolus dans le CLI typé.
- Installer le hook de mesure et le hook de handoff dans la même transformation Arnes.
- Réutiliser l'échange atomique avec comparaison du snapshot pour ne perdre aucune écriture
  concurrente.
- Préserver les données étrangères et migrer l'ancien chemin du hook.
- Exécuter le test jusqu'au succès.

### 3. Vérifier les barrières concernées

Fichiers : Arnes, `Makefile` et `.github/workflows/lint.yml`.

- Exécuter le formatage, Clippy et les tests Arnes.
- Exécuter `tooling/agent-handoff-test`, `tooling/makefile-test` et les tests Arnes sur macOS.
- Vérifier que le lint CI découvre les scripts imbriqués, le dry-run Make, le YAML et
  `git diff --check`.
- Auto-relire le diff, committer, pousser puis surveiller les checks de la PR.
