# ADR-039 — Recherche exacte et conceptuelle dans les checkouts Git

- **Statut** : accepté
- **Date** : 2026-08
- **Remplace** : ADR-039 initiale sur CodeGraph, issue #250, 2026-08

## Contexte

Les agents ont besoin d'une recherche exacte et d'une recherche conceptuelle simples. Un index
partagé ou résolu depuis un autre checkout peut produire des résultats appartenant au mauvais état
de code. Multiplier les moteurs, leurs configurations MCP et leurs politiques augmente aussi la
surface de maintenance sans besoin distinct.

## Décision

Supprimer CodeGraph du dépôt, de l'installation, des agents et de la CI. Distribuer une skill
publique `code-search` à Claude Code, Codex et Cursor. Elle route les littéraux, expressions
régulières et chemins connus vers `rg` et `fd`, et les recherches conceptuelles vers
`colgrep-search` dans tout checkout Git, principal ou lié.

`colgrep-search` détermine et canonicalise la racine du checkout avec Git après neutralisation des
variables de routage héritées. Il refuse les répertoires hors Git et les sous-modules, initialise ou
actualise ColGrep à la demande, puis vérifie que l'index et tous les résultats appartiennent à cette
racine avant de publier une sortie complète. Tout doute ou échec produit un refus explicite et un
repli borné vers `rg` et `fd`.

Installer ColGrep et `colgrep-search` dans le profil minimal. Ne jamais initialiser ColGrep depuis
un hook de création de worktree ou un autre hook de cycle de vie.

## Conséquences

- Chaque checkout conceptuellement interrogé possède son index ColGrep.
- Une panne ou un état invalide produit un repli explicite vers `rg` et `fd`.
- La première recherche conceptuelle peut payer l'initialisation ; la création du checkout reste
  sans cet effet de bord.
- Les moteurs restent des couches de récupération. Les refactorings sémantiques et le débogage
  relèvent respectivement de LSP et DAP.
- L'intégration complète les ADR-015, ADR-028, ADR-033 et ADR-036 sans les remplacer.

## Alternatives écartées

- Conserver CodeGraph hors linked worktree : deux moteurs et deux politiques pour un même besoin.
- Appeler ColGrep par MCP ou directement : contourne la preuve de racine et le confinement des
  résultats.
- Initialiser ColGrep depuis le hook de création : coût imposé aux worktrees qui n'effectuent aucune
  recherche conceptuelle.
