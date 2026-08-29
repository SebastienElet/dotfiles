# Routage de recherche dans les worktrees

## Contexte

L'issue GitHub #250 établit que CodeGraph peut résoudre l'index d'un autre checkout lorsqu'une
session opère dans un Git worktree. L'ADR-039 impose pourtant qu'une réponse périmée ou issue d'un
autre checkout ne soit jamais servie silencieusement. ColGrep 1.7.0 stocke chaque index hors du
projet, l'identifie par le chemin canonique et le modèle, puis conserve ce chemin dans
`project.json`.

La politique actuelle vit dans `harness/skills/codegraph/` et ses installations Claude Code,
Codex et Cursor sont des projections de cette source. Ce mécanisme est adapté au routage, mais une
instruction reste advisory : la garantie fail-closed doit être portée par l'entrée qui invoque
ColGrep.

## Décision

Conserver le slug canonique `codegraph` et élargir sa politique de récupération :

- une recherche exacte utilise directement `rg` ou `fd` ;
- une recherche conceptuelle dans un linked worktree utilise l'entrée contrôlée ColGrep ;
- une exploration structurelle hors linked worktree conserve le routage CodeGraph existant ;
- toute conclusion conséquente est vérifiée dans les sources et par les oracles adaptés.

Ajouter une petite entrée TypeScript installée par le Makefile. Elle résout le chemin canonique du
dépôt actif avec Git, prouve que `git-dir` et `git-common-dir` désignent un linked worktree, puis
invoque ColGrep avec cette racine explicite. Les variables Git héritées sont retirées des probes afin
que le répertoire déclaré reste l'autorité.

Une recherche conceptuelle constitue la demande d'initialisation : l'entrée lance
`colgrep init -y <racine>` pour construire ou mettre à jour l'index, puis contrôle l'état produit
avant toute lecture de résultats. Elle valide les sorties externes et les métadonnées structurées,
exige que `project.json.project_path` égale exactement la racine canonique et refuse un état absent,
illisible, incomplet, sale, ambigu ou étranger. La recherche finale utilise `--no-update`, une
sortie JSON et la même racine ; une sortie non conforme ou extérieure à cette racine est rejetée en
entier.

Le refus annonce la raison et ordonne un repli borné vers `rg`/`fd`; il ne lance ni CodeGraph ni une
seconde tentative ColGrep. L'entrée refuse aussi son usage hors linked worktree, où la skill reprend
le routage CodeGraph existant.

## Déploiement

Installer ColGrep par la source Homebrew gérée par le dépôt et déployer l'entrée contrôlée sous
`~/.local/bin/`. Ne pas utiliser les installateurs d'agents upstream : ils ajouteraient une seconde
politique et des hooks non partagés. Les trois agents continuent de recevoir la même skill par les
liens Makefile existants.

Aucun hook de création de worktree n'est ajouté ou modifié. L'initialisation ColGrep demeure un
effet du premier besoin conceptuel dans le worktree.

## Vérification

Le cycle TDD exerce l'entrée réelle avec des dépendances de processus substituables et des dépôts
Git temporaires. Les cas de refus couvrent Git ou ColGrep absent, preuve Git vide ou malformée,
checkout principal, sous-module, racine inexistante ou étrangère, `status` ambigu, métadonnées JSON
invalides, index absent, sale ou incompatible, échec d'initialisation et résultat hors racine.

Une intégration opt-in avec ColGrep 1.7.0 crée deux worktrees divergents et un répertoire de données
isolé. Elle prouve que les fichiers suivis modifiés et les fichiers non suivis du worktree actif sont
retrouvés, qu'aucun symbole propre au voisin ne l'est et que l'index du voisin ne contamine pas la
réponse. Les contrats de policy prouvent séparément que les recherches exactes ne lancent ni
ColGrep, ni son initialisation, ni CodeGraph, et que le hook de création de worktree reste absent.

Les barrières comprennent tests Bun, typecheck TypeScript 7, Oxlint/Oxfmt, validation de la skill,
tests de déploiement sur les trois projections, dry-run des cibles Makefile d'installation et
intégration réelle sur les plateformes effectivement exercées.

## Décisions adjacentes

L'ADR-039 est amendée plutôt que remplacée : CodeGraph reste la décision en vigueur hors worktree,
tandis que ColGrep devient la décision worktree. L'issue #220 reste nécessaire pour Cursor hors
worktree. L'issue #121 reste nécessaire pour la validation déclarative des MCP, puisque CodeGraph
demeure un MCP géré et que ColGrep est ici une CLI, pas un MCP.

## Limites

Le graphe lexical ColGrep n'est pas présenté comme un oracle exhaustif d'impact ou de couverture.
Le contrôle protège les invocations passant par l'entrée distribuée ; la skill reste la couche de
routage des agents et ne constitue pas une barrière système contre une invocation volontaire du
binaire brut. Aucun comportement CodeGraph hors worktree et aucun hook de création de worktree ne
change.
