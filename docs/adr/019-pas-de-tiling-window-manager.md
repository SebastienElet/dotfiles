# ADR-019 — Aucun gestionnaire de fenêtres en tuiles

- **Statut** : accepté
- **Date** : 2025-12
- **Révision** : 2026-10-20
- **Commits** : `0b6a5b9`, `163e6df`

## Contexte

Quatre gestionnaires en tuiles se sont succédé entre 2014 et 2022 : slate, kwm
avec khd, chunkwm, puis yabai. Chacun demandait une configuration
substantielle, cassait à chaque version majeure de macOS et, pour yabai,
exigeait la désactivation partielle de la protection d'intégrité du système.

Rectangle Pro avait ensuite été retenu pour le positionnement manuel au clavier.
Le choix est désormais d'utiliser les raccourcis natifs de macOS 27 pour les
demi-écrans, sans conserver Rectangle Pro pour ses fonctions supplémentaires.

## Décision

Abandonner le pavage automatique et utiliser les
[raccourcis natifs de macOS](https://support.apple.com/fr-fr/guide/mac-help/mchl9674d0b0/27/mac/27)
pour positionner les fenêtres au clavier. Rectangle Pro est retiré du profil
optionnel ; la cible `yabai` et le fichier `yabairc` restent supprimés.

## Conséquences

- Aucun gestionnaire de fenêtres tiers à installer, ni d'affaiblissement de SIP.
- Perte du pavage automatique et des espaces virtuels scriptés.
- Les commandes Rectangle Pro d'écran suivant ou précédent avec centrage ne
  sont pas reprises par cette configuration.
- Le retrait du Brewfile ne désinstalle pas une application déjà présente.

## Alternatives écartées

- yabai : coût de maintenance et exigence de désactivation partielle de SIP.
- AeroSpace : non évalué dans l'historique ; réexamen possible.
- Rectangle Pro : ses fonctions supplémentaires ne sont plus retenues pour
  l'usage courant.
