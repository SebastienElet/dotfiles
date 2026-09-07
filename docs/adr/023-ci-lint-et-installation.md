# ADR-023 — CI de lint et smoke minimal macOS

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-09-07
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

Les workflows spécialisés couvrent lint, types, tests et déploiements. Le smoke historique
passait par Make pour compléter les dépendances qui n'avaient pas encore migré vers Moon.

## Décision

Les workflows choisissent les runners, amorcent leurs runtimes et invoquent les tâches Moon.
Leur préparation des outils de contrôle appartient au graphe Moon. Les plateformes existantes
sont conservées : le poste complet est macOS ; les suites portables continuent sur Ubuntu.

Le workflow d'installation conserve un job sur `macos-latest`, pour chaque pull request
et push vers `main`, et appelle `tooling:smoke-minimal`.
Cet oracle exerce le point d'entrée public `repository:install` avec l'entrée standard fermée.
Il contrôle le Brewfile, les exécutables observés et le défaut Node hors projet, relève les
artefacts possédés déjà observés par le smoke, puis capture un second passage du même profil.
Le second passage doit retourner zéro, garder stdout et stderr vides, satisfaire les
postconditions et laisser les artefacts relevés identiques.

## Conséquences

- Le smoke exerce le profil minimal public sans sélection des installations à partir du diff.
- Les tests spécialisés continuent d'exercer leurs erreurs et comportements sur leurs runners.
- Le résultat d'un graphe ou d'une validation de syntaxe ne remplace pas le smoke exécuté.
- La preuve reste limitée au runner nommé et aux observations effectuées ; elle ne couvre pas
  les optionnels, l'authentification, le démarrage d'OrbStack ou les écritures internes de Homebrew.

## Alternatives écartées

- Parser Make, les Brewfiles ou Moon pour en recopier le graphe dans un test.
- Installer toutes les applications optionnelles dans le smoke minimal.
- Déduire la réussite du profil d'une vérification manuelle avant push.
