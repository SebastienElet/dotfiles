# ADR-002 — Homebrew et `mas` comme unique source de paquets

- **Statut** : accepté
- **Date** : 2015-02
- **Commits** : `5c64955`, `ab9771e`

## Contexte

Les outils proviennent de canaux hétérogènes : formules Homebrew, casks,
App Store, installateurs éditeur. Multiplier les canaux rend la mise à jour
impossible à automatiser et l'installation impossible à rejouer en CI.

## Décision

Tout paquet passe par Homebrew (formule ou cask), et par `mas` pour ce que seul
l'App Store distribue. Une autre source exige une exception explicite et un
contrat de version, d'intégrité, de mise à jour et de rejeu. L'inventaire de ces
exceptions est un document opérationnel distinct.

Le graphe `make all` est contrôlé automatiquement : toute source absente de
l'inventaire est refusée. Les services sont pilotés par `brew services` et les
applications payantes de l'App Store restent exclues par défaut via
`SKIP_PAID_APPS` afin que la CI puisse rejouer l'installation complète.

## Conséquences

- Une seule commande de mise à jour pour l'ensemble du poste
  ([ADR-004](004-script-upgrade-unique.md)).
- Dépendance forte aux dépréciations Homebrew : plusieurs commits ne font que
  suivre des taps ou des formules retirés en amont.
- Les exceptions ne bénéficient pas du contrat de version et de mise à jour
  Homebrew ; leur contrat plus faible est explicite et contrôlé séparément.

## Alternatives écartées

- Installateurs éditeur ou téléchargements manuels : non rejouables.
- Nix : rupture d'outillage disproportionnée au regard du besoin.
