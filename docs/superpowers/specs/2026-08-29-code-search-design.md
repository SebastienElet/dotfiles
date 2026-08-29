# Recherche de code indépendante du moteur

## Intention

Le dépôt ne gère plus CodeGraph. La capacité exposée aux agents s'appelle `code-search` et décrit
un besoin de recherche, pas une technologie. Son contrat est identique pour Claude Code, Codex et
Cursor : `rg`/`fd` pour une cible exacte, ColGrep pour une recherche conceptuelle.

Cette décision remplace entièrement le compromis précédent qui conservait CodeGraph hors linked
worktree. Aucun checkout, agent ou profil d'installation ne dépend encore de CodeGraph.

## Routage

La skill canonique `code-search` classe chaque demande avant tout index :

- littéral, expression régulière, symbole exact, chemin connu ou vérification ciblée : `rg` ou
  `fd`, sans ColGrep ;
- recherche conceptuelle ou cartographie ouverte : point d'entrée contrôlé ColGrep, puis
  vérification des conclusions importantes dans les sources ;
- refus du point d'entrée : repli annoncé et borné vers `rg`/`fd`.

Le routage ne distingue pas checkout principal et linked worktree. Cette distinction était une
conséquence de CodeGraph, pas un besoin utilisateur.

## Frontière ColGrep

Le point d'entrée devient `colgrep-search`. Pour chaque recherche conceptuelle, il retire les
variables Git héritées, résout la racine canonique du checkout actif et refuse une preuve Git vide,
ambiguë ou extérieure. Un checkout principal et un linked worktree sont tous deux acceptés ; un
répertoire hors Git et un superprojet ambigu sont refusés.

L'initialisation ColGrep reste strictement à la demande. Le point d'entrée lance
`colgrep init -y <racine>`, contrôle `colgrep status <racine>`, puis valide que le répertoire
d'index, `project.json`, `state.json` et chaque résultat appartiennent exactement à cette racine.
La recherche finale utilise la même racine, JSON et `--no-update`. Toute sortie partielle,
malformée, sale, vide ou étrangère est retenue en mémoire puis rejetée sans résultat publié.

Aucun hook de création de worktree ou de dépôt n'initialise ColGrep. Aucun agent n'appelle le
binaire brut dans le parcours documenté.

## Suppression de CodeGraph

Supprimer du dépôt :

- installation Volta et cibles Make CodeGraph ;
- génération et tests des configurations MCP Claude Code, Codex et Cursor ;
- mesure de taille, intégrations, scripts et workflow dédiés ;
- skill `codegraph`, projections agent et documentation opérationnelle ;
- exclusion Git `.codegraph` et références actives dans les contrôles ou profils.

L'ADR-039 devient la décision `CodeSearch` et ne conserve aucune portée CodeGraph. Les documents de
validation historiques devenus faux sont supprimés ou remplacés par les preuves ColGrep actuelles.

Le dépôt ne fournit aucun script de migration ou de désinstallation ponctuel. Il cesse simplement
de gérer CodeGraph. Le nettoyage du CLI, des inscriptions MCP, des anciens liens de skill et des
répertoires `.codegraph/` déjà présents reste manuel et hors de cette PR.

## Déploiement et CI

Le profil minimal installe ColGrep par Homebrew et déploie `colgrep-search` ainsi que la skill
`code-search`. Cursor reste optionnel mais reçoit la même skill lorsqu'il est installé ; aucune
configuration MCP de recherche n'est créée.

Le workflow dédié devient `Code search tests`. Il installe ColGrep, exerce le déploiement, les
refus fail-closed et une intégration réelle. Les barrières générales couvrent TypeScript, Makefile,
Brewfile, YAML, Markdown et dictionnaire utilisateur.

## Preuves

Le cycle TDD conserve les refus existants et ajoute le succès du checkout principal. Une
intégration réelle crée un checkout principal et deux linked worktrees divergents, avec fichiers
suivis modifiés et non suivis. Les requêtes de chaque checkout ne publient aucun symbole propre aux
deux autres.

Des scénarios frais pour Claude Code, Codex et Cursor doivent démontrer : recherche exacte sans
index, recherche conceptuelle via `colgrep-search`, aucun appel CodeGraph, repli borné sur refus et
absence d'initialisation par hook.

La PR ferme #250 après merge. #220 devient obsolète puisque Cursor ne reçoit plus CodeGraph. #121
reste ouverte pour le diagnostic générique des autres MCP gérés par le dépôt.
