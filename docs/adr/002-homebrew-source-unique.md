# ADR-002 — Homebrew et `mas` comme unique source de paquets

- **Statut** : accepté
- **Date** : 2015-02
- **Commits** : `5c64955`, `ab9771e`

## Contexte

Les outils proviennent de canaux hétérogènes : formules Homebrew, casks,
App Store, installateurs éditeur. Multiplier les canaux rend la mise à jour
impossible à automatiser et l'installation impossible à rejouer en CI.

## Décision

Tout paquet passe par Homebrew (formule ou cask), et par `mas` pour ce que
seul l'App Store distribue. Les services sont pilotés par `brew services`. Les
applications payantes de l'App Store sont exclues par défaut via
`SKIP_PAID_APPS`, ce qui permet à la CI d'exécuter l'installation complète.
`HOMEBREW_NO_ASK` supprime les invites interactives, et les taps sont
approuvés (`brew trust`) avant toute mise à jour.

## Conséquences

- Une seule commande de mise à jour pour l'ensemble du poste
  ([ADR-004](004-script-upgrade-unique.md)).
- Dépendance forte aux dépréciations Homebrew : plusieurs commits ne font que
  suivre des taps ou des formules retirés en amont.
- Quelques outils échappent à la règle (installateur natif de Claude Code,
  images Docker des MCP) et constituent des exceptions assumées.

## Alternatives écartées

- Installateurs éditeur ou téléchargements manuels : non rejouables.
- Nix : rupture d'outillage disproportionnée au regard du besoin.
