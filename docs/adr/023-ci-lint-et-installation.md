# ADR-023 — CI de lint et smoke minimal macOS

- **Statut** : accepté
- **Date** : 2026-08
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

Le smoke historique sélectionnait des cibles à partir du diff du `Makefile` sur les pull requests,
puis exécutait l’agrégat `all` après fusion. Cette accélération reproduisait la sémantique de
l’installateur et devait contourner les applications payantes et Docker.

## Décision

Les workflows spécialisés continuent de vérifier lint, types, tests et déploiements couverts. Le
workflow macOS d’installation contient un job unique sur `macos-latest`, pour chaque pull request et
push vers `main`, et appelle seulement l’oracle public `make smoke-minimal`.

Le smoke vérifie les prérequis fournis par le runner, exécute `make minimal`, contrôle le Brewfile et
les exécutables non-Homebrew, relève les artefacts possédés, puis capture un second `make minimal`.
Ce second passage doit retourner `0`, garder stdout et stderr vides, satisfaire les mêmes
postconditions et laisser les artefacts relevés identiques.

## Conséquences

- La CI exerce exactement le point d’entrée public minimal, sans matrice ni sélection de diff.
- La preuve reste limitée au runner macOS nommé et aux artefacts observés.
- Elle ne garantit ni les optionnels, ni une authentification, ni le démarrage du daemon OrbStack,
  ni les écritures internes de Homebrew.

## Alternatives écartées

- Parser le `Makefile` ou les Brewfiles : duplication de la source canonique.
- Rejouer toutes les applications optionnelles : coût et prérequis sans rapport avec le socle.
- Vérification manuelle avant push : absence de preuve reproductible.
