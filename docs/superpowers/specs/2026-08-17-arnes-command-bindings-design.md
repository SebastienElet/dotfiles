# Validation des bindings de commandes Arnes — Design

**Date** : 2026-08-17 · **Statut** : approuvé

## Objectif

Traiter l'issue #110 en implémentant :

```text
arnes doctor commands [--agent claude|cursor|codex] [--scope user|project]
```

La commande valide les déclarations logiques de commandes et leurs bindings vers les agents, sans
exécuter de commande, modifier le disque ni devenir propriétaire du contenu des prompts. Elle
réutilise la validation des sources et projections livrée par #109 au lieu de reconstruire une
seconde représentation du même prompt.

## Non-objectifs

- implémenter `sync` ;
- exécuter une commande, un prompt ou du shell ;
- diagnostiquer les hooks ;
- adopter ou parcourir les commandes unmanaged et celles appartenant à des plugins ;
- réimplémenter la résolution des includes, le rendu ou la comparaison du corps des prompts ;
- fournir une représentation de commande Cursor ou Codex sans contrat stable documenté.

## Modèle du manifeste

Le manifeste v1 accepte une liste `commands` optionnelle. Une entrée décrit une commande logique et
regroupe ses bindings afin de ne pas répéter son nom, sa description et sa référence de prompt :

```yaml
commands:
  - name: review-pull-request
    description: Review a pull request and report blocking findings
    prompt: review-pull-request
    bindings:
      - agent: claude
        scope: user
      - agent: cursor
        scope: project
```

Une commande porte exactement quatre champs :

- `name`, nom invocable commun à tous ses bindings ;
- `description`, description commune et non vide ;
- `prompt`, identifiant d'un prompt normalisé déclaré dans le même manifeste ;
- `bindings`, liste non vide de couples `{agent, scope}` sans champ de surcharge.

Tous les bindings d'une entrée partagent strictement `name`, `description` et `prompt`. Une variante
fonctionnelle est une autre commande ; aucun override par agent ou scope n'est ajouté.

## Validation statique

La validation du manifeste précède tout accès au disque et s'applique aussi aux capacités qui seront
ensuite signalées `unsupported` :

- `name` respecte `^[a-z0-9]+(?:-[a-z0-9]+)*$` ;
- `description` contient au moins un caractère non blanc ;
- `prompt` référence un identifiant existant ;
- chaque agent et scope appartient aux déclarations `agents` du manifeste ;
- `bindings` n'est pas vide ;
- l'identité développée `(agent, scope, name)` est unique dans tout le manifeste.

Le même nom reste donc autorisé entre deux agents ou scopes. Deux commandes ne peuvent en revanche
pas revendiquer la même invocation pour le même agent et le même scope, y compris par deux bindings
dupliqués dans une seule entrée.

## Capacités

La matrice de capacités de `doctor commands` est distincte de celle des prompts :

| Agent | Scope | État | Contrat |
| --- | --- | --- | --- |
| Claude | `user` | pris en charge | `.claude/commands/<name>.md` sous `HOME` |
| Claude | `project` | pris en charge | `.claude/commands/<name>.md` sous la racine du dépôt |
| Cursor | `user` ou `project` | `unsupported` | aucune description de commande stable à valider |
| Codex | `user` ou `project` | `unsupported` | aucune représentation de commande prise en charge |

Cursor reste validé au niveau du manifeste : nom, description, référence de prompt, cible, scope et
unicité doivent être corrects. `unsupported` concerne uniquement l'absence de contrat stable pour
contrôler son binding sur disque et produit un exit code `0` en l'absence d'un diagnostic plus grave.

## Binding Claude

Pour un binding Claude, Arnes recherche dans le prompt référencé la projection portant le même agent
et le même scope. La destination déclarée par cette projection doit être exactement
`.claude/commands/<name>.md` dans la racine imposée par le scope. Cette projection constitue le
binding : Arnes ne construit pas un second fichier et ne copie pas le corps du prompt dans la
déclaration de commande.

La validation de source, des includes, des variables, de la représentation directe ou rendue, du
type de fichier, des symlinks et de la fraîcheur du corps reste la responsabilité du composant
`prompts`. `commands` appelle cette validation ciblée et transpose son résultat dans un diagnostic
de commande, sans relancer un scan global des prompts.

Lorsque la projection est saine, `commands` lit son frontmatter YAML et vérifie que `description`
est une chaîne exactement égale à celle du manifeste. L'absence de frontmatter, un frontmatter
illisible, une description absente ou différente rendent le binding périmé. Les autres clés du
frontmatter restent hors propriété de la commande et sont ignorées.

## Collisions et propriété

Le tracker de topologie des prompts est étendu ou exposé par une interface ciblée. La destination de
la projection référencée est le seul alias autorisé entre une commande et un prompt. Une autre
commande, projection ou ressource qui revendique cette destination constitue une collision.

La détection réutilise les identités de chemins planifiées et canoniques existantes afin de couvrir
les alias et les symlinks sans sortir de `HOME` ou de la racine du dépôt. Un fichier présent au
chemin explicitement déclaré appartient au binding et peut donc être diagnostiqué. Aucun sibling ni
répertoire unmanaged n'est parcouru ; les bindings de plugins restent hors manifeste et hors
périmètre.

## Flux de diagnostic

`doctor commands` suit cet ordre :

1. charger et valider le manifeste complet ;
2. développer chaque commande logique en bindings tout en conservant l'ordre du manifeste ;
3. appliquer `--agent` et `--scope` avant les accès au disque ;
4. retourner `unsupported` pour une sélection sans contrat stable ;
5. pour Claude, résoudre le prompt et sa projection correspondante ;
6. valider la topologie, déléguer le contrôle de projection à `prompts`, puis contrôler le binding ;
7. produire le rapport dans les formats humain ou JSON existants.

Une sélection explicite qui ne correspond à aucun binding déclaré produit un diagnostic
`unsupported`, conformément au comportement actuel de `doctor prompts`. Des diagnostics mixtes
conservent la règle existante : la sévérité maximale fixe l'exit code.

## États et erreurs

| État | Exit code | Cas principaux |
| --- | --- | --- |
| `healthy` | `0` | projection Claude saine, destination et description conformes |
| `unsupported` | `0` | binding Cursor ou Codex, ou sélection sans binding déclaré |
| `drift` | `1` | fichier absent, mauvais type, corps périmé, frontmatter ou description non conforme |
| `error` | `2` | manifeste invalide, prompt absent, projection Claude absente ou incompatible, collision, chemin non sûr ou I/O illisible |

Une projection déclarée avec une représentation sans contrat stable conserve l'état `unsupported`
retourné par `prompts`. Aucun échec n'est transformé en écriture corrective et aucune absence n'est
masquée par un scan de découverte.

## Découpage interne

- `manifest/commands.rs` porte les déclarations désérialisées et les vues normalisées de commandes ;
- `manifest/validation/commands.rs` porte les invariants purs et l'unicité des bindings ;
- `commands.rs` orchestre filtres, ordre et production des diagnostics ;
- `commands/capability.rs` décrit la matrice agent/scope et les destinations attendues ;
- un composant de binding Claude valide destination et metadata ;
- `prompts` expose une primitive interne ciblée de validation de projection, sans exposer son
  orchestration CLI ni ses diagnostics de ressource.

Parsing, politique, I/O et orchestration restent ainsi séparés. Aucun de ces composants ne doit
devenir un second propriétaire du contenu des prompts ni étendre le générique historique
`resources kind: commands`.

## Tests

Les tests de manifeste couvrent :

- une commande à plusieurs bindings sans duplication des métadonnées ;
- un nom vide ou hors kebab-case ASCII et une description vide ou blanche ;
- une liste de bindings vide, une cible ou un scope non déclaré et un prompt absent ;
- un doublon `(agent, scope, name)` dans une entrée ou entre deux commandes ;
- le même nom accepté sur des agents ou scopes distincts ;
- les champs inconnus refusés par `deny_unknown_fields`.

Les fixtures de diagnostic couvrent :

- un binding Claude sain en scopes `user` et `project` ;
- une projection Claude absente, incompatible, manquante, périmée ou illisible ;
- une description absente, mal typée ou différente ;
- les collisions entre commandes, prompts et ressources, dont les alias par symlink ;
- Cursor et Codex explicitement `unsupported` après validation statique ;
- les filtres agent/scope, l'ordre stable et la dominance des exit codes ;
- un binding unmanaged ou de plugin voisin qui reste ignoré ;
- l'absence de lecture du vrai `HOME` grâce aux fixtures `env_clear`, `HOME` et CWD temporaires ;
- le routage CLI de `doctor commands`, qui ne doit plus tomber dans le fallback vide.

## Vérification

Depuis `tooling/arnes`, l'implémentation devra exécuter les barrières qui couvrent réellement les
fichiers Rust touchés :

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Le développement local exercera macOS, plateforme principale et filesystem généralement insensible
à la casse. Le workflow Arnes existant exercera Ubuntu ; sa réussite ne sera revendiquée qu'après
exécution effective en CI. Les résultats finaux nommeront séparément chaque environnement vérifié et
ne déduiront jamais la portabilité de l'un à partir de l'autre.

## Livraison

Le changement fonctionnel reste limité à #110. Aucun ADR n'est requis : il ajoute une capacité
prévue à l'architecture Arnes sans modifier une décision structurelle en vigueur. L'issue ne sera
fermée qu'après implémentation vérifiée et intégrée, jamais sur la seule base de ce design.
