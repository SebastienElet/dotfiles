# Installation durable de Semctx par Moon

## Intention

Installer et maintenir les plugins Semctx du poste par une tâche Moon dédiée au harnais, sans
ajouter Semctx aux profils `make minimal` ou `make optional`. Bun porte la version reproductible du
CLI Semctx ; Moon porte l'orchestration ; Arnes observe les capacités externes installées.

Cette tranche introduit le premier point d'entrée `harness:*` dans Moon. Elle ne remplace pas encore
Make comme installateur public du poste et ne modifie aucune ADR en vigueur.

## Architecture

Le répertoire `harness/` devient un projet Moon explicite, enregistré sous l'identifiant `harness`
dans `.moon/workspace.yml`. Son `moon.yml` contient uniquement les tâches qui opèrent sur le harnais
multi-agent ; il ne contient aucune logique d'installation métier.

Le paquet `semctx` est une dépendance de développement exacte du projet Bun racine. `package.json`
est la source de sa version et `bun.lock` rend sa résolution reproductible. Les tâches Moon
invoquent le binaire résolu dans `node_modules`; elles ne recopient pas la version dans leur YAML et
n'utilisent pas `bunx semctx@latest`.

Le manifeste `home/.arnes.yaml` autorise les plugins et skills Semctx exposés par les hôtes détectés.
Cette déclaration observe une capacité externe : elle ne possède ni le plugin, ni son cache, ni sa
configuration native.

## Tâches Moon

### `harness:semctx-install`

Exécute `semctx install --host auto --skip-setup --json`. Le mode `auto` converge tous les hôtes
Codex ou Claude Code détectés sans rendre obligatoire un hôte absent. `--skip-setup` maintient une
frontière stricte entre installation machine et initialisation du dépôt courant.

La tâche possède des effets externes : réseau, marketplaces, caches de plugins et configurations
utilisateur des agents. Elle désactive donc le cache Moon, ne s'exécute pas en CI et utilise un mutex
commun aux mutations du harnais.

### `harness:install`

Agrège les installations du harnais. Dans cette tranche, sa seule dépendance est
`harness:semctx-install`. L'ajout ultérieur d'une capacité exige sa propre tâche avec un contrat
d'effets explicite ; l'agrégateur ne devient pas un script impératif.

### `harness:semctx-setup`

Exécute explicitement `semctx setup --polyglot` depuis le dépôt Git courant. Cette tâche initialise
ou met à jour `.semctx/`, construit l'index et valide le modèle sémantique. Elle est non cachée,
exclue de CI et n'est jamais une dépendance de `harness:install`.

Le choix `--polyglot` crée la configuration v2 pour ce dépôt neuf et conserve séparément les
langages supportés, partiels et non supportés. Il ne transforme pas l'absence d'analyseur Rust en
preuve négative.

### `harness:semctx-status`

Exécute le diagnostic local de livraison des plugins en JSON. Il ne modifie rien et n'effectue pas
d'attestation réseau implicite. Elle utilise `bun run semctx plugin-status --host auto --json` pour
produire ce rapport en lecture seule. La tâche reste non cachée, car son résultat dépend de l'état
du poste hors du graphe Moon.

## Migration du canal local

L'installation actuelle de Codex utilise une marketplace `semctx-stable` liée au checkout local
`/Users/sebastien/Code/semctx`. L'installateur officiel refuse volontairement d'écraser une
marketplace homonyme dont la source n'est pas le dépôt public attendu.

La migration initiale est donc séparée de la tâche durable : inspecter l'état natif, retirer
explicitement le plugin et la marketplace locaux après autorisation, puis lancer
`harness:semctx-install`. Aucun chemin personnel ni commande de suppression n'est inscrit dans le
projet Moon.

## Gestion des erreurs

- Une dépendance Bun absente ou un lockfile désynchronisé échoue avant toute mutation de plugin.
- Un conflit de marketplace reste un échec de l'installateur Semctx et conserve son diagnostic.
- Un hôte absent est ignoré par `--host auto`; un hôte détecté mais non convergé fait échouer la
  tâche.
- `harness:semctx-setup` ne masque ni `SETUP_NOT_READY`, ni une couverture partielle, ni un index
  invalide.
- Une nouvelle session Codex ou un rechargement Claude Code reste nécessaire lorsque le rapport
  Semctx le demande ; le cache installé n'est pas présenté comme la version active de la session.

## Vérification

La modification est vérifiée sans installer réellement de plugin depuis le worktree :

1. installation Bun gelée et cohérence de `package.json` avec `bun.lock` ;
2. résolution du projet et des quatre tâches par Moon ;
3. inspection du graphe de dépendances de `harness:install` ;
4. `semctx install --host auto --skip-setup --dry-run --json` ;
5. `semctx plugin-status --host auto --json` ;
6. Arnes Doctor sur les plugins, skills et MCP déclarés ;
7. formatage YAML/Markdown et gates TypeScript existantes affectées par le lockfile.

L'installation réelle est exécutée depuis le checkout canonique seulement après la migration
explicite de la marketplace locale. L'initialisation `.semctx/` constitue une action séparée et
requiert sa propre exécution volontaire.

## Hors périmètre

- ajout de Semctx à `make minimal` ou `make optional` ;
- remplacement général de Make par Moon ;
- installation automatique de Moon sur un poste neuf ;
- hook bloquant, intégration CI Semctx ou attestation réseau automatique ;
- support d'analyse Rust par Semctx ;
- suppression automatique d'une marketplace ou d'un cache existant.
