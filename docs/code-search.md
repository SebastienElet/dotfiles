# Recherche de code

Les trois agents utilisent la skill `code-search`. Les recherches exactes passent par `rg` et `fd`;
les recherches conceptuelles passent par `colgrep-search '<requête>'` depuis le checkout Git ciblé.

Le point d'entrée initialise ColGrep à la demande, prouve la racine canonique du checkout, valide
l'index et refuse les résultats extérieurs. En cas de refus, utiliser une recherche `rg`/`fd`
bornée. Aucun hook de création de worktree ne doit initialiser ColGrep.
