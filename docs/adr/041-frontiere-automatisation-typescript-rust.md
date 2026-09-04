# ADR-041 — Frontière d'automatisation entre Shell, Moon, TypeScript et Rust

- **Statut** : accepté
- **Date** : 2026-08

## Contexte

Les utilitaires historiques en Bash deviennent difficiles à lire et à tester dès qu'ils analysent
des données, choisissent un repli, portent une politique ou traduisent des erreurs. Rust fournit une
excellente robustesse pour une CLI durable, mais impose un coût disproportionné aux petits
utilitaires qu'un développeur maîtrisant TypeScript doit pouvoir produire et maintenir rapidement.

[Bun exécute TypeScript directement](https://bun.sh/docs/typescript) et fournit un
[test runner intégré](https://bun.sh/docs/test), mais l'exécution ne remplace pas la vérification
statique. La documentation Bun recommande une configuration TypeScript stricte avec `noEmit`, et
[TypeScript 7 fournit désormais le binaire `tsc`
natif](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/). Le dépôt ne contient
encore aucun utilitaire TypeScript : la toolchain sera introduite avec son premier consommateur
plutôt que comme infrastructure vide.

## Décision

- Moon devient l'orchestrateur unique selon l'ADR-001 ; les parcours Make non migrés sont transitoires.
- Bash se limite à l'amorçage, à l'environnement et à une courte séquence linéaire de commandes.
- Moon porte les tâches d'installation et de développement, leur graphe de dépendances
  et leur sélection affectée. Les tâches simples restent dans le projet racine ; un projet séparé
  correspond à une responsabilité existante, pas à un simple préfixe de commande.
- Bun et TypeScript sont le choix par défaut pour un petit utilitaire cohésif qui dépasse la frontière
  Shell, y compris l'analyse et la validation de données ou une politique bornée.
- Rust est retenu pour une CLI substantielle, un état durable, une concurrence complexe, une
  intégration système privilégiée, une exigence de performance ou la distribution d'un binaire
  autonome.
- Le premier utilitaire TypeScript introduit une version verrouillée de Bun et TypeScript 7, le
  lockfile, des tests `bun test` et une gate CI `tsc --noEmit`. Les tests exercent le véritable point
  d'entrée et ses chemins d'échec.

## Conséquences

- La majorité des nouvelles logiques d'automatisation simples quitte Bash sans payer par défaut le
  coût d'une CLI Rust.
- Bun et `tsc` constituent deux oracles distincts : tests d'exécution et vérification statique.
- Une migration de Bash reste locale au comportement modifié ; les scripts historiques ne sont pas
  réécrits sans besoin.
- La migration vers Moon est progressive : les commandes non migrées conservent leur point d’entrée
  actuel.
- La frontière Rust repose sur les garanties requises, pas sur un seuil arbitraire de lignes.

## Alternatives écartées

- Bash pour tout utilitaire : les branches, parsers et tests Shell reproduisent précisément la dette
  que cette décision doit arrêter.
- Rust pour toute logique : robuste, mais trop coûteux pour les petits utilitaires et moins accessible
  à leur mainteneur principal.
- Bun sans `tsc` : l'exécution TypeScript ne prouve pas que le programme satisfait son contrat de
  types.
- Une toolchain TypeScript immédiatement sans consommateur : infrastructure et CI sans comportement
  à garantir.
