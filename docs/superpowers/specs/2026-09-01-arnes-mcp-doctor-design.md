# Diagnostic MCP dans Arnes Doctor

## Intention

Arnes doit considérer les inscriptions MCP gérées par le dépôt comme une ressource de diagnostic à
part entière. `arnes doctor mcp` contrôle une sélection explicite, tandis que `arnes doctor` inclut
les MCP déclarés dans son diagnostic courant afin qu'une dérive ne reste pas invisible tant que
l'utilisateur ne connaît pas la sous-commande spécialisée.

Le diagnostic reste strictement en lecture seule. Il ne lance aucun serveur MCP, n'appelle aucun
CLI d'agent, n'inspecte ni ne démarre aucun conteneur, ne tire aucune image, ne contacte aucun réseau
et ne modifie aucune configuration.

## Source canonique

Le manifeste Arnes porte une collection normalisée d'inscriptions MCP. Chaque inscription déclare
explicitement :

- le nom MCP visible par l'agent ;
- l'agent et le scope ciblés ;
- la commande locale attendue ;
- les arguments attendus dans leur ordre exact ;
- les noms des variables d'environnement référencées ;
- l'état activé attendu lorsque l'agent représente cet état dans sa configuration.

Une inscription représente exactement une projection `(agent, scope, nom)`. Des inscriptions
séparées expriment les différences réelles entre agents ou scopes ; le manifeste ne déduit pas de
valeurs communes et n'applique pas d'override implicite. Cette forme volontairement explicite évite
qu'une abstraction de projection masque une différence de commande ou d'arguments.

Le manifeste ne contient aucune valeur de secret. Une variable d'environnement est représentée par
son nom et par la référence attendue dans la configuration de l'agent, jamais par sa valeur résolue.
Une déclaration qui embarque une valeur sous un champ dont le nom indique un secret est refusée par
la validation existante du manifeste.

## Configurations observées

Le diagnostic lit directement les fichiers natifs sans appeler les agents :

| Agent | Scope utilisateur | Scope projet |
| --- | --- | --- |
| Claude Code | `~/.claude.json` | `.mcp.json` |
| Cursor | `~/.cursor/mcp.json` | `.cursor/mcp.json` |
| Codex | `~/.codex/config.toml` | `.codex/config.toml` |

Les formats JSON et TOML sont validés avant toute comparaison. Une racine, une table ou une entrée
de type inattendu produit une erreur explicite ; une valeur inconnue ne devient jamais une valeur
par défaut plausible. Les clés sans rapport avec MCP et les inscriptions dont le nom n'est pas
déclaré dans le manifeste sont conservées hors du périmètre du diagnostic.

Pour Claude Code, l'état désactivé enregistré pour le projet courant est pris en compte lorsqu'il
est observable sans exécution. Pour Codex, le champ `enabled` de l'inscription est comparé. Cursor
n'expose pas d'état activé portable dans ses fichiers MCP documentés ; aucune santé n'est donc
affirmée sur cet état.

## Comparaison

Une inscription est saine lorsque son nom, sa commande, ses arguments ordonnés, ses références de
variables d'environnement et, lorsqu'il existe, son état activé correspondent au manifeste. Chaque
différence produit un diagnostic actionnable qui nomme le champ et la valeur attendue sans rendre
une valeur sensible observée.

La disponibilité d'une commande locale est contrôlée sans l'exécuter :

- une commande absolue ou contenant un séparateur de chemin doit désigner un fichier exécutable ;
- un nom de commande simple doit résoudre vers un fichier exécutable dans le `PATH` injecté ;
- un chemin relatif est résolu depuis la racine du scope, home pour `user` et dépôt pour `project` ;
- une référence de chemin non résoluble ou ambiguë est une erreur de diagnostic, jamais un succès ;
- un wrapper couvert par l'ADR-031 est traité comme toute autre commande locale, sans inspection de
  son image, de son daemon ou de ses conteneurs.

Le chemin réellement résolu est la valeur contrôlée. Une simple présence textuelle dans le fichier
de configuration ne suffit pas à prouver la disponibilité du lanceur.

## Doublons et collisions

Le manifeste refuse deux déclarations portant le même triplet `(agent, scope, nom)`. Le parseur
refuse également les clés MCP dupliquées dans un même objet JSON ou une même table TOML au lieu de
conserver silencieusement la dernière valeur.

Lorsqu'un même nom géré apparaît dans plusieurs scopes chargés par un agent, le diagnostic signale
la collision et la priorité susceptible de masquer une inscription. Une inscription non gérée ou
fournie par un plugin n'est jamais signalée pour sa seule présence. Son contenu ne devient pertinent
que s'il occupe un nom déclaré et peut donc masquer l'inscription gérée attendue.

## Sélection et rendu

`arnes doctor mcp --agent <agent> --scope <scope>` limite le diagnostic à la combinaison demandée.
Une combinaison explicitement demandée mais non supportée ou non déclarée produit un état
`unsupported`, distinct d'une inscription saine et non bloquant conformément au contrat Doctor.

Sans ressource explicite, `arnes doctor` ajoute une section `MCP` aux sections existantes. Sans
filtre `--scope`, cette section contrôle toutes les projections MCP déclarées, y compris les MCP de
projet ; les autres ressources conservent leur comportement de sélection actuel. Un `--agent` ou
un `--scope` fourni explicitement filtre aussi la section MCP.

Le rendu normal masque les détails sains selon les conventions Doctor. Le rendu verbose expose les
inscriptions saines sans secret. Le JSON conserve un diagnostic par constat et les codes de sortie
existants : `0` pour sain ou unsupported seul, `1` pour une dérive, `2` pour une erreur
opérationnelle ou un format invalide.

## Preuves attendues

Les tests emploient des homes, dépôts et `PATH` isolés et exercent le vrai binaire Arnes. Ils
établissent au minimum :

- une projection saine pour chaque format JSON et TOML supporté ;
- l'absence, la commande divergente, les arguments désordonnés, les références d'environnement
  divergentes et l'état désactivé inattendu ;
- une commande absolue, un wrapper relatif et un nom résolu dans le `PATH`, puis leurs absences et
  permissions non exécutables ;
- les doublons et collisions de scopes sans bruit sur les entrées étrangères ou fournies par un
  plugin ;
- une combinaison unsupported distincte d'un succès vide ;
- l'absence de valeur de secret dans les sorties humaine et JSON ;
- l'inclusion de la section MCP dans `arnes doctor` sans ressource explicite ;
- l'absence de mutation des fixtures et l'absence d'exécution du lanceur référencé.

Une vérification d'ensemble couvre le formatage, le lint Rust et les tests Arnes sur macOS. Les
garanties portables restent limitées aux plateformes effectivement exercées par la CI ; la présence
d'une application macOS optionnelle peut légitimement produire une dérive sur une autre plateforme.

## Limites

La synchronisation des configurations, le démarrage et la santé protocolaire des serveurs, les
connexions OAuth, les approbations interactives, les conteneurs, les images et le réseau restent hors
périmètre. Le diagnostic prouve la conformité statique de l'inscription et la disponibilité locale
du lanceur ; il ne prétend pas prouver qu'un serveur démarrera ni qu'un outil MCP répondra.

Cette conception implémente l'issue #121, sous le contrat en lecture seule de l'issue parente #111,
et respecte la topologie des wrappers couverts par l'ADR-031 sans l'étendre aux autres MCP.
