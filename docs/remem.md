# Retrait de remem

L’utilisateur a retiré remem du harnais le 2026-10-01, faute de plus-value de la mémoire.
La [décision de nettoyage #152](https://github.com/SebastienElet/dotfiles/issues/152#issuecomment-5926721552)
et la [PR #365](https://github.com/SebastienElet/dotfiles/pull/365#issuecomment-5926721948)
portent ce retrait. L’ADR-042 ne fait plus partie des décisions en vigueur ; Git conserve son historique.

Les profils Codex et Claude ne déploient plus remem : installation, configuration, MCP, hooks,
worker et skill `remem-memory` sont retirés. La procédure de
[nettoyage explicite](deployment-clean.md) reconnaît les anciens déploiements après suppression
de leurs déclarations du manifeste.

Les bases, clés, logs et historiques sous `~/.remem/`, les sauvegardes et les paquets tiers
restent conservés. Les réglages `autoMemoryEnabled` et `features.memories` ne sont pas restaurés.
`agent-handoff` et la mémoire historique de Cursor restent indépendants et inchangés.
