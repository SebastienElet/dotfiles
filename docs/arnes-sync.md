# Arnes Sync : ressources déclarées

`arnes sync` synchronise les ressources prises en charge déclarées dans
`$HOME/.arnes.yaml`. La ressource, l'agent et la portée sont obligatoires :

```sh
arnes sync skills --agent codex --scope user
arnes sync skills --agent claude --scope project --format json
arnes sync rules --agent cursor --scope user
arnes sync statusline --agent codex --scope project
arnes sync config --agent claude --scope user
arnes sync instructions --agent codex --scope user
arnes sync prompts --agent cursor --scope project
arnes sync commands --agent claude --scope project
arnes sync mcp --agent claude --scope project
```

Les skills user sont des liens feuille vers `harness/skills/` du checkout
identifié par le manifeste déployé. Les skills project sont des liens racine
vers `.agents/skills/` du dépôt courant. Les trois agents sont pris en charge.
Les rules restent limitées à Claude et Cursor user ; leurs sources restent
relatives au dépôt courant, comme dans Doctor.

Les [configurations natives et MCP](arnes-sync-native.md) ainsi que les
[instructions, prompts et commandes](arnes-sync-markdown.md) ont leurs limites
de représentation et de propriété détaillées séparément. Config synchronise les
valeurs user déclarées ; les instructions couvrent Claude user/project et Codex
user. Prompts couvre Claude user/project et Cursor project ; commands reste
limité à Claude user/project. MCP traite les inscriptions locales des trois agents,
avec leurs limites d'état enabled. Aucun de ces parcours ne lance une session
d'agent ou un serveur MCP.

Les [ADR-001](adr/001-makefile-installateur.md),
[ADR-003](adr/003-deploiement-par-symlinks.md),
[ADR-028](adr/028-skills-ssot.md), [ADR-038](adr/038-frontieres-home-harness-tooling.md)
et [ADR-040](adr/040-skills-user-dans-harness.md), acceptées, définissent
l'orchestration, les frontières et les sources. Moon reste l'orchestrateur
d'installation. Sync n'installe aucun agent, outil, dépendance ou plugin.

## Mutations et refus

Pour les liens de skills et rules, le manifeste entier et les sources sélectionnées
sont validés avant toute création de projection. `SKILL.md` et ses ressources relatives doivent être
présents et confinés. Une source invalide ou une collision refuse la sélection
avant création ; les projections conformes restent préservées.

Une destination absente reçoit un symlink vers la source canonique. Un lien
correct, absolu ou relatif, reste inchangé. Un fichier, répertoire, hardlink,
lien divergent ou lien pendant est préservé avec un refus. La racine de portée
et les parents de destination doivent être des répertoires sans symlink.
Une destination dans une source sélectionnée ou dans la collection canonique
de skills concernée est également refusée.

La création des parents et du lien passe par des descripteurs de répertoires
ouverts sans suivi de liens. La publication du symlink ne remplace jamais une
destination apparue entre validation et écriture. Une modification concurrente
de source, de parent ou de lien observée provoque un échec explicite. Ces
contrôles ne constituent pas une transaction globale : des liens déjà publiés
et des répertoires créés peuvent subsister si une publication suivante échoue.
Sync ne supprime aucune ressource et ne restaure pas automatiquement un état
concurrent.

La statusline est limitée à Codex user/project. Seule la liste ordonnée
`tui.status_line` est synchronisée ; les autres valeurs TOML sont conservées.
Un fichier absent est créé et un fichier conforme garde ses octets, son inode
et sa date de modification. Lors d'une modification, le TOML est rendu à
nouveau : la présentation et les commentaires ne sont pas conservés.
Une configuration malformée ou un type incompatible est refusé sans écrire
la configuration. Un hardlink ou un lien symbolique de fichier ou de parent
provoque un échec d'ouverture sécurisé, avec le code `2`.
Une sélection vide ou un autre agent ne crée aucun fichier.

La publication de la configuration réutilise la frontière I/O des hooks :
verrou par fichier, lecture sans suivi de liens, fichier temporaire puis
renommage avec contrôle d'identité. Les permissions du fichier existant sont
conservées. L'identité du HOME et de son répertoire de configuration est
revérifiée après l'attente du verrou et aux frontières de publication ; un
déplacement observé provoque un échec. Ces contrôles ne garantissent pas
l'absence de toute concurrence possible. Les erreurs sont signalées sans
exposer les valeurs TOML. Un HOME à l'intérieur du dépôt de déploiement ou
d'une source `home/`, `harness/` ou d'une collection de skills du dépôt courant
est refusé pour préserver les sources.

Les skills retirés du manifeste, les ressources locales, les plugins et les
skills système sont préservés. Un inventaire externe indisponible n'autorise
aucune mutation ; Sync ne lance aucun résolveur d'agent. Le manifeste n'est pas
généré depuis l'installation observée.

## Rapports et vérification

Le rapport human ou JSON annonce la ressource, l'agent et la portée sélectionnés.
Chaque entrée distingue `applied`, `current`, `refused`, `failed`, `empty` et
`unsupported`. La sortie vaut `0` uniquement si toutes les entrées sont applied
ou current, `1` pour refus, sélection vide ou combinaison non prise en charge,
et `2` pour erreur de lecture du manifeste, d'environnement, de publication ou
d'écriture de sortie. Un résultat partiel n'est jamais présenté comme succès.

Doctor reste une commande séparée en lecture seule :

```sh
arnes doctor skills --agent codex --scope user
arnes doctor rules --agent cursor --scope user
arnes doctor statusline --agent codex --scope project
```

Le diagnostic skills peut aussi signaler des capacités externes ; un refus ou
une limite d'inventaire externe ne signifie pas que Sync possède ces ressources.
La conformité des fichiers ne prouve ni leur activation ni leur chargement par
une session d'agent.

Les tests `sync_links`, `sync_errors`, `sync_statusline`, `sync_config`, `sync_mcp`,
`sync_instructions`, `sync_prompts`, `sync_commands` et `sync::links::tests` exercent la CLI,
Doctor ciblé, le rejeu, les collisions, les références invalides, la préservation
des voisins et les changements intervenant après validation. Les tests utilisent
des dépôts et HOME temporaires. La livraison doit nommer macOS et Ubuntu comme
exercés ou non vérifiés ; ces fixtures n'attestent pas une installation complète
du poste ou une session réelle d'agent.
