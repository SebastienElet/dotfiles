# Intégration de CodeGraph aux agents — Design

**Date** : 2026-08-14 · **Statut** : approuvé oralement, en attente de revue écrite

## Contexte

L'issue [#97](https://github.com/SebastienElet/dotfiles/issues/97) demande une couche de
récupération structurelle locale utilisable par Codex, Claude Code et Cursor. Le benchmark prévu
par l'issue #86 n'a pas produit de preuve comparative valide et ne doit pas être réinterprété comme
une victoire ou un échec de CodeGraph. Le présent travail ne rouvre pas ce benchmark : il intègre
directement CodeGraph 1.5.0, version stable courante au moment de la décision, parce que l'objectif
est désormais d'obtenir une solution simple et réutilisable dans plusieurs dépôts.

L'installation doit fonctionner dans ce dépôt, dans les grands dépôts privés utilisés localement
et dans de futurs dépôts, sans configuration spécifique à chacun avant que l'index soit utile. Le
code, les chemins, les noms de dépôts privés, les requêtes et les traces ne sont jamais publiés.

## Décision

CodeGraph est installé globalement, épinglé à `@colbymchenry/codegraph@1.5.0`, puis déclaré comme
serveur MCP stdio global dans les trois agents. Chaque dépôt conserve son propre index natif dans
`.codegraph/`. Ce répertoire est exclu globalement de Git, mais il n'est ni déplacé, ni remplacé par
un symlink : CodeGraph 1.5.0 ne propose pas de répertoire de données externe par dépôt et ajouter
une indirection locale rendrait la solution plus fragile que l'usage upstream.

Aucune façade MCP, aucun wrapper d'exécution, aucun serveur HTTP partagé et aucun gestionnaire de
daemon maison n'est ajouté. Le serveur upstream démarre à la demande, partage son daemon local
entre les sessions du même dépôt et l'arrête après son délai d'inactivité natif. `rg` et `fd`
restent les outils de recherche exacte et le mode dégradé.

## Écarts explicites avec le texte initial de l'issue

Les clarifications validées après la rédaction de l'issue remplacent quatre critères devenus
incompatibles avec CodeGraph 1.5.0 ou avec l'objectif de simplicité :

- l'état par dépôt réside dans `.codegraph/`, et non hors du dépôt de travail ; l'exclusion Git
  globale garantit qu'il ne soit pas versionné ;
- la surface MCP conserve son défaut upstream, `codegraph_explore` uniquement ; `trace` n'existe
  plus comme outil distinct et ses chemins d'appel sont intégrés à `explore`, tandis que `impact`
  reste disponible en CLI mais n'est pas réactivé dans la liste MCP ;
- aucun plafond maison de cinq régions ou 500 lignes n'enveloppe la réponse ; le contexte borné et
  classé fourni par `codegraph_explore` est vérifié tel quel ;
- le cycle de vie utilise le watcher, le daemon partagé et les commandes de maintenance upstream,
  sans script de démarrage ou d'arrêt supplémentaire.

Ces écarts doivent apparaître dans le compte rendu final de l'issue pour que ses cases historiques
ne soient pas validées artificiellement.

## Installation et configuration globale

Le `Makefile`, source canonique d'installation du dépôt, assure quatre postconditions :

1. Volta installe exactement `@colbymchenry/codegraph@1.5.0` ;
2. `codegraph telemetry off` enregistre durablement le refus de télémétrie ;
3. Codex, Claude Code et Cursor possèdent une entrée MCP stdio nommée `codegraph`, lançant
   `codegraph serve --mcp` avec le chemin de workspace exigé par Cursor ;
4. chaque entrée MCP transmet `CODEGRAPH_TELEMETRY=0`, `CODEGRAPH_NO_UPDATE_CHECK=1` et
   `CODEGRAPH_NO_DOWNLOAD=1` afin d'interdire la télémétrie, la recherche automatique de version
   et le téléchargement de secours au démarrage du serveur.

L'installateur global CodeGraph ne peut pas être exécuté tel quel pour Claude Code : il écrit dans
`~/.claude/CLAUDE.md`, qui est un symlink vers `ai/AGENTS.md`, et modifierait donc la source commune
d'instructions en contournant l'ADR-036. La configuration utilise les commandes MCP natives de
Claude Code et Codex, qui savent enregistrer les variables d'environnement. Cursor ne fournit pas
de commande d'ajout équivalente : son entrée `~/.cursor/mcp.json` est fusionnée atomiquement avec
`jq`, sans toucher aux autres serveurs. `codegraph install --print-config` sert de référence de
forme lors des tests, mais aucune étape ne nettoie après coup des écritures indésirables de
l'installateur.

Le fichier global Git `~/.config/git/ignore` devient une configuration gérée par ce dépôt et
contient à la fois son entrée existante et `.codegraph/`. Le déploiement suit le modèle de symlink
du dépôt. Une destination préexistante différente n'est jamais écrasée automatiquement : la
première installation s'arrête et demande une réconciliation explicite.

## Activation par dépôt

La décision d'initialiser appartient aux agents, mais seulement lorsqu'une exploration ouverte et
structurelle la justifie. Une recherche de littéral, d'expression régulière ou de chemin connu ne
mesure pas le dépôt et ne déclenche jamais CodeGraph.

Pour une exploration structurelle sans `.codegraph/`, l'agent compte avec `tokei` les lignes et les
fichiers des langages pris en charge par CodeGraph. Le comptage respecte les fichiers ignorés et
exclut les dépendances, sorties générées, répertoires vendored, builds, couverture, documentation,
fixtures et lockfiles. Aucune dépendance supplémentaire n'est ajoutée : `tokei` et `jq` sont déjà
installés par le `Makefile`. La skill versionne la liste des langages reconnue par CodeGraph 1.5.0
et la commande de comptage exacte ; Markdown, texte brut et formats de données ne contribuent ni
aux lignes ni au nombre de fichiers.

L'agent exécute automatiquement `codegraph init` si au moins une condition est vraie :

- 50 000 lignes de code source ou davantage ;
- 500 fichiers source ou davantage.

Le `OR` est intentionnel : un dépôt composé de nombreux petits fichiers comme un dépôt concentré
dans quelques gros fichiers peut bénéficier du graphe. Sous les deux seuils, l'agent poursuit avec
`rg` et `fd`, sauf si un index existe déjà ; un index existant reste utilisable quelle que soit la
taille actuelle du dépôt. Le seuil est une politique d'activation pragmatique, pas une affirmation
de supériorité mesurée : les résultats upstream deviennent nettement plus réguliers autour de 640
fichiers, tandis que les dépôts proches de 110 fichiers montrent encore un coût fixe et des gains
de temps variables.

## Guidance partagée

La politique conditionnelle vit dans une seule skill `.agents/skills/codegraph/`, distribuée vers
les emplacements globaux de Codex, Claude Code et Cursor selon le mécanisme existant des skills
partagées. Elle ne duplique pas la documentation des requêtes : les instructions MCP fournies par
CodeGraph restent la référence pour `codegraph_explore`.

La skill porte seulement les décisions propres à cet environnement :

- reconnaître exploration structurelle, architecture, appels, dépendances et impact comme cas
  CodeGraph ;
- réserver `rg` et `fd` aux recherches exactes et aux vérifications contre la source ;
- appliquer le seuil lorsqu'aucun index n'existe et lancer `codegraph init` au-dessus ;
- contrôler la santé avant usage, synchroniser une fois si nécessaire et annoncer tout mode
  dégradé ;
- préfixer chaque commande CLI CodeGraph par `CODEGRAPH_TELEMETRY=0`,
  `CODEGRAPH_NO_UPDATE_CHECK=1` et `CODEGRAPH_NO_DOWNLOAD=1`, comme les entrées MCP ;
- ne jamais supprimer ou reconstruire silencieusement un index défectueux.

Aucune règle permanente n'est ajoutée à `ai/AGENTS.md` sans l'ablation marginale exigée par
l'ADR-036. Les trois agents découvrent la même skill globale ; aucun adapter agent-spécifique ne
contient une copie de sa politique.

## Fraîcheur et modes de panne

Quand `.codegraph/` existe et que la tâche appelle une exploration structurelle, l'agent commence
par `codegraph status --json`. Un état sain mène directement à `codegraph_explore`. Un état périmé
ou incomplet autorise une seule exécution de `codegraph sync`, suivie d'un nouveau contrôle.

Si l'index manque au-dessus du seuil, `codegraph init` construit l'index initial puis `status`
confirme qu'il est utilisable. Si l'initialisation, la synchronisation ou le second contrôle
échoue, l'agent nomme l'échec et repasse explicitement à `rg` et `fd`. Il ne présente jamais un
résultat CodeGraph comme frais après un contrôle en erreur.

Une corruption, un verrou inexpliqué ou une incompatibilité de format n'autorise ni
`codegraph uninit --force` ni `codegraph index` automatiquement, car ces commandes détruisent ou
reconstruisent l'état. L'agent expose le diagnostic et demande une décision avant cette action. Les
conclusions importantes issues du graphe sont vérifiées dans les fichiers source avant une
modification.

Les chemins de maintenance documentés restent ceux de l'outil : `codegraph status`,
`codegraph sync`, `codegraph daemon`, `codegraph uninit` et `codegraph uninstall`. Aucun processus
Ollama, serveur de modèle ou service distant n'est installé ni démarré.

## Confidentialité et réseau

CodeGraph analyse et interroge les sources localement. La configuration désactive ses trois sorties
réseau automatiques connues : télémétrie, contrôle de version et téléchargement de binaire de
secours. Le téléchargement du paquet épinglé pendant l'installation reste autorisé et distinct de
l'exécution sur un dépôt.

Une vérification réseau observe l'indexation, la synchronisation et les requêtes MCP sur une
fixture publique locale. Elle doit montrer qu'aucune connexion sortante n'est ouverte par le
processus CodeGraph pendant ces opérations. Cette preuve est produite sur macOS ; Linux reste une
cible supportée par l'outil mais non validée tant qu'aucune exécution Linux n'est réalisée.

## Vérification

La vérification comprend quatre niveaux :

1. **Installation statique** : `make -n codegraph`, contrôle de la version épinglée, validation du
   JSON Cursor, du TOML Codex et de l'entrée Claude Code dans un environnement isolé qui ne lance
   aucun installateur global.
2. **Politique** : tests de la skill sur les deux côtés de chaque seuil, sur le `OR`, sur un index
   existant sous le seuil et sur une recherche exacte qui ne doit jamais initialiser ; passage du
   doctor et de la synchronisation de l'index des skills.
3. **MCP réel** : fixture publique avec appels et dépendances, puis édition, renommage, suppression,
   changement de branche, redémarrage du serveur et interruption du watcher ; chaque mutation doit
   être visible après réconciliation et aucun état périmé ne peut être rendu silencieusement.
4. **Adoption** : smoke test dans Codex, Claude Code et Cursor montrant la disponibilité de
   `codegraph_explore`, son usage pour une exploration ouverte et l'usage de `rg` ou `fd` pour une
   recherche exacte.

Les mesures locales consignent temps d'indexation initiale, temps de synchronisation, latence de
requête, CPU, RSS et volume disque sans publier de donnée issue d'un dépôt privé. Le test de cycle
de vie confirme le retour à zéro processus CodeGraph non désiré après l'arrêt explicite ou le délai
d'inactivité upstream.

La barrière finale couvre effectivement chaque extension modifiée : validation Makefile,
ShellCheck pour tout shell ajouté, parseurs JSON et TOML, doctor des skills, tests fonctionnels MCP,
`git diff --check` et `git status`. Chaque preuve nomme macOS et les cibles non exercées.

## Livrables

L'implémentation reste découpée en changements révocables :

1. installation épinglée, configuration MCP globale et exclusion Git ;
2. skill partagée et tests de seuil ;
3. fixture, probe MCP et tests de fraîcheur ;
4. ADR-038, documentation d'exploitation et résultats locaux expurgés.

L'ADR-038 enregistre CodeGraph comme couche de récupération à la demande, son seuil d'activation,
sa relation avec `rg` et `fd`, le stockage natif `.codegraph/` et la décision de ne pas ajouter de
wrapper. Le périmètre reste la récupération et la compréhension structurelle ; les renommages
sémantiques, code actions et débogage interactif relèvent d'une intégration LSP/DAP distincte.
