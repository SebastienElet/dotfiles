# ADR-039 — CodeGraph à la demande pour la récupération structurelle

- **Statut** : accepté
- **Date** : 2026-08

## Contexte

L'issue #86 n'a pas produit de benchmark comparatif valide. L'issue #97 vise désormais une
intégration opérationnelle simple de CodeGraph dans plusieurs dépôts et trois agents. Les
résultats upstream montrent un coût fixe visible autour de 110 fichiers et des gains plus réguliers
à partir d'environ 640 fichiers ; ils motivent un seuil pragmatique, pas une preuve locale de
supériorité.

## Décision

Installer CodeGraph globalement, le mettre à jour avec les autres paquets npm globaux et exposer son
serveur MCP stdio upstream à Codex, Claude Code et Cursor. Utiliser `codegraph_explore` pour
l'exploration structurelle ; conserver `rg` et `fd` pour les littéraux, expressions régulières,
chemins connus et vérifications ciblées.

Lorsqu'une exploration structurelle rencontre un dépôt sans index, initialiser automatiquement à
partir de 50 000 lignes de source ou 500 fichiers source. Sous les deux seuils, ne pas initialiser.
Un index existant reste utilisable sous le seuil après contrôle de fraîcheur.

Conserver l'état natif `.codegraph/`, ignoré globalement par Git. Ne pas ajouter de wrapper MCP,
proxy, répertoire externe par symlink, daemon maison ni règle permanente dans `harness/AGENTS.md`.
Distribuer la politique conditionnelle par la skill partagée `codegraph`.

## Conséquences

- Chaque dépôt ou worktree indexé paie son propre espace disque.
- La télémétrie, le contrôle de version et le téléchargement de secours sont désactivés à
  l'exécution ; installation et mise à jour passent par le flux central `tooling/upgrade`.
- Une panne ou un état périmé produit un repli explicite vers `rg` et `fd`, jamais une réponse
  silencieusement obsolète.
- CodeGraph reste une couche de récupération. Les refactorings sémantiques et le débogage relèvent
  respectivement de LSP et DAP.
- L'intégration complète les ADR-015, ADR-028, ADR-033 et ADR-036 sans les remplacer.

## Alternatives écartées

- Indexer chaque dépôt : coût inutile sur les petits dépôts.
- Laisser l'utilisateur lancer `codegraph init` manuellement : état implicite facile à oublier.
- Stocker les index hors dépôt par symlink : contournement d'une capacité upstream absente.
- Réactiver les outils MCP cachés ou ajouter une façade : surface et maintenance sans besoin établi.
- Ajouter la règle à `harness/AGENTS.md` : instruction permanente sans ablation marginale.
- Épingler CodeGraph : empêche de bénéficier normalement des correctifs et améliorations upstream.
