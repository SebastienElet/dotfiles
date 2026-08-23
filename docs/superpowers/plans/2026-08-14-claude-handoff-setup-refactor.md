# Plan du refactor du setup du hook Claude

**Objectif :** installer l'entrée Stop de handoff sans multiplier les writers de la configuration
Claude.

**Architecture :** le script Bash valide les chemins et délègue à Arnes l'installation atomique des
hooks de mesure et de handoff. Le test reste un exécutable direct de la CI; le Makefile ne fournit
qu'une façade phony.

### 1. Réduire le test aux garanties utiles

Fichier : `tooling/claude-handoff-hook-test`

- Couvrir la délégation exacte vers Arnes.
- Couvrir la résolution du script depuis un répertoire extérieur au dépôt.
- Conserver les garanties de mutation JSON et de concurrence dans les tests Arnes.

### 2. Simplifier le setup

Fichier : `tooling/claude-handoff-hook`

- Valider les trois chemins absolus à la frontière Bash.
- Installer le hook de mesure et le hook de handoff dans la même transformation Arnes.
- Réutiliser l'échange atomique avec comparaison du snapshot pour ne perdre aucune écriture
  concurrente.
- Préserver les données étrangères et migrer l'ancien chemin du hook.
- Exécuter le test jusqu'au succès.

### 3. Vérifier les barrières concernées

Fichiers : les quatre scripts Bash, `Makefile` et `.github/workflows/lint.yml`.

- Exécuter `bash -n` et le ShellCheck complet sur les quatre scripts.
- Exécuter `tooling/agent-handoff-test` et `tooling/claude-handoff-hook-test` sur macOS.
- Vérifier que le lint CI découvre les scripts imbriqués, le dry-run Make, le YAML et
  `git diff --check`.
- Auto-relire le diff, committer, pousser puis surveiller les checks de la PR.
