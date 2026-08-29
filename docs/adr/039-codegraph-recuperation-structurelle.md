# ADR-039 — Récupération structurelle selon le type de checkout

- **Statut** : accepté
- **Date** : 2026-08
- **Amendement** : issue #250, 2026-08

## Contexte

L'issue #86 n'a pas produit de benchmark comparatif valide. L'issue #97 vise désormais une
intégration opérationnelle simple de CodeGraph dans plusieurs dépôts et trois agents. Les résultats
upstream montrent un coût fixe visible autour de 110 fichiers et des gains plus réguliers à partir
d'environ 640 fichiers ; ils motivent un seuil pragmatique, pas une preuve locale de supériorité.

L'issue #250 a ensuite établi qu'un index CodeGraph pouvait décrire un autre checkout lors d'une
exploration depuis un linked worktree. Le contrôle `worktreeMismatch` protège un index interrogé,
mais ne constitue pas une frontière suffisante pour autoriser CodeGraph dans ce contexte.

## Décision

Hors linked worktree, installer CodeGraph globalement, le mettre à jour avec les autres paquets npm
globaux et exposer son serveur MCP stdio upstream à Codex et Claude Code dans le profil minimal.
L'installation optionnelle de Cursor ajoute son intégration sans être une dépendance de CodeGraph.
Utiliser `codegraph_explore` pour l'exploration structurelle ; conserver `rg` et `fd` pour les
littéraux, expressions régulières, chemins connus et vérifications ciblées.

Lorsqu'une exploration structurelle rencontre un dépôt sans index, initialiser automatiquement à
partir de 50 000 lignes de source ou 500 fichiers source. Sous les deux seuils, ne pas initialiser.
Un index existant reste utilisable sous le seuil après contrôle de fraîcheur.

Conserver l'état natif `.codegraph/`, ignoré globalement par Git. Ne pas ajouter de wrapper MCP,
proxy, répertoire externe par symlink, daemon maison ni règle permanente dans `harness/AGENTS.md`.
Distribuer la politique conditionnelle par la skill partagée `codegraph`.

Dans un linked worktree, interdire CodeGraph et réserver `rg`/`fd` aux recherches exactes et au
repli borné. Pour une recherche conceptuelle, utiliser ColGrep exclusivement via le point d'entrée
`colgrep-worktree`. Celui-ci doit prouver la racine canonique du linked worktree, initialiser ou
actualiser l'index à la demande, valider que l'état ColGrep appartient exactement à cette racine et
refuser tout résultat extérieur avant de publier la sortie. Tout doute ou échec impose le repli
borné vers `rg`/`fd`.

Installer ColGrep et le point d'entrée avec le profil minimal. Ne pas initialiser ColGrep depuis un
hook de création de worktree : l'initialisation reste un effet de la première recherche
conceptuelle. Les issues #121 et #220 restent applicables à CodeGraph hors linked worktree.

## Conséquences

- Chaque checkout indexé paie son propre espace disque dans le moteur qui lui est attribué.
- La télémétrie, le contrôle de version et le téléchargement de secours sont désactivés à
  l'exécution de CodeGraph ; installation et mise à jour passent par les flux centraux.
- Une panne ou un état périmé produit un repli explicite vers `rg` et `fd`, jamais une réponse
  silencieusement obsolète.
- Une recherche conceptuelle en linked worktree peut payer l'initialisation ColGrep lors de sa
  première exécution ; la création du worktree reste sans cet effet de bord.
- Les moteurs restent des couches de récupération. Les refactorings sémantiques et le débogage
  relèvent respectivement de LSP et DAP.
- L'intégration complète les ADR-015, ADR-028, ADR-033 et ADR-036 sans les remplacer.

## Alternatives écartées

- Indexer chaque dépôt avec CodeGraph : coût inutile sur les petits dépôts.
- Laisser l'utilisateur lancer `codegraph init` manuellement : état implicite facile à oublier.
- Stocker les index CodeGraph hors dépôt par symlink : contournement d'une capacité upstream
  absente.
- Réactiver les outils MCP cachés ou ajouter une façade : surface et maintenance sans besoin
  établi.
- Ajouter la règle à `harness/AGENTS.md` : instruction permanente sans ablation marginale.
- Épingler CodeGraph : empêche de bénéficier normalement des correctifs et améliorations upstream.
- Conserver CodeGraph dans les linked worktrees : la résolution d'index ne fournit pas la frontière
  canonique exigée par #250.
- Appeler directement ColGrep : contourne la preuve de racine et le confinement des résultats.
- Initialiser ColGrep depuis le hook de création : coût imposé aux worktrees qui n'effectuent aucune
  recherche conceptuelle.
