# Plan du refactor du setup du hook Claude

**Objectif :** remplacer la validation générale de Claude par un setup court qui ne modifie que
notre entrée Stop.

**Architecture :** le script Bash expose une fonction par étape et délègue la seule transformation
JSON à un filtre `jq` court. Le test reste un exécutable direct de la CI; le Makefile ne fournit
qu'une façade phony.

### 1. Réduire le test aux garanties utiles

Fichier : `scripts/setup/claude_handoff_hook_test`

- Conserver création, préservation, migration, idempotence et JSON invalide.
- Ajouter un handler de type inconnu à préserver pour obtenir un échec avec l'implémentation
  actuelle.
- Retirer les tests de validation globale, signaux, liens symboliques, runner sélectif et Makefile.
- Exécuter le test et constater l'échec sur le handler inconnu.

### 2. Simplifier le setup

Fichier : `scripts/setup/claude_handoff_hook`

- Écrire de petites fonctions pour valider l'entrée, lire les réglages, ajouter ou migrer notre
  entrée et installer atomiquement le résultat.
- Limiter le filtre `jq` au chemin `.hooks.Stop` et à la commande reçue.
- Préserver les données étrangères sans tenter de connaître le schéma Claude complet.
- Exécuter le test jusqu'au succès.

### 3. Vérifier les barrières concernées

Fichiers : les quatre scripts Bash, `Makefile` et `.github/workflows/test.yml`.

- Exécuter `bash -n` et le ShellCheck complet sur les quatre scripts.
- Exécuter `scripts/agent_handoff_test` et `scripts/setup/claude_handoff_hook_test` sur macOS.
- Vérifier que le lint CI découvre les scripts imbriqués, le dry-run Make, le YAML et
  `git diff --check`.
- Auto-relire le diff, committer, pousser puis surveiller les checks de la PR.
