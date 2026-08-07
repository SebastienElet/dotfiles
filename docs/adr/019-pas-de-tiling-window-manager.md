# ADR-019 — Aucun gestionnaire de fenêtres en tuiles

- **Statut** : accepté
- **Date** : 2025-12
- **Commits** : `0b6a5b9`, `163e6df`

## Contexte

Quatre gestionnaires en tuiles se sont succédé entre 2014 et 2022 : slate, kwm
avec khd, chunkwm, puis yabai. Chacun demandait une configuration
substantielle, cassait à chaque version majeure de macOS et, pour yabai,
exigeait la désactivation partielle de la protection d'intégrité du système.

## Décision

Abandonner le pavage automatique. La cible `yabai` et le fichier `yabairc` sont
supprimés ; Rectangle Pro couvre le besoin réel — positionner une fenêtre au
raccourci clavier — sans configuration ni concession de sécurité.

## Conséquences

- Plus de rupture à chaque mise à jour de macOS, ni d'affaiblissement de SIP.
- Perte du pavage automatique et des espaces virtuels scriptés.
- Une dépendance payante de plus, exclue des installations par défaut
  ([ADR-002](002-homebrew-source-unique.md)).

## Alternatives écartées

- yabai : coût de maintenance et exigence de désactivation partielle de SIP.
- AeroSpace : non évalué dans l'historique ; réexamen possible.
