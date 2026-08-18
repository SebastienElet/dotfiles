# Lisibilité des diagnostics humains Arnes — Design

**Date** : 2026-08-18 · **Statut** : approuvé

## Objectif

Rendre la sortie humaine de `arnes doctor` immédiatement exploitable sans modifier ce qu'Arnes
observe. La vue par défaut indique combien de diagnostics sont sains, masque leur inventaire, puis
affiche tous les diagnostics `error`, `drift` et `unsupported`. L'option globale `-v, --verbose`
réinsère les diagnostics `healthy` dans cette même vue.

La couleur ajoute un signal visuel à cette projection sans remplacer les états, symboles ou textes.
Elle est automatique sur un terminal, contrôlable explicitement et absente du JSON.

Pour `doctor skills`, les diagnostics sont présentés par agent. L'agent qui porte l'état le plus
grave apparaît en premier afin que la configuration effectivement incorrecte ne soit plus noyée
par les capacités saines ou non observables.

## Non-objectifs

- modifier collecte, états, exit codes, schéma JSON ou ordre canonique des diagnostics ;
- réparer une ressource, suggérer une commande ou coupler Arnes à une procédure externe ;
- déduire une structure humaine en parsant `resource` ou `message` ;
- ajouter une dépendance de rendu, ni adopter les ressources externes.

## Interface CLI

`-v, --verbose` et `--color` sont des options de `doctor`, disponibles pour toutes les ressources :

```text
arnes doctor [RESOURCE] [--agent AGENT] [--scope SCOPE] [--format FORMAT]
  [--color auto|always|never] [-v|--verbose]
```

Le raccourci `-v` est libre ; `-V` reste réservé à la version par Clap. L'option ne change que le
rendu humain, jamais `diagnose` ni le calcul de l'exit code.

Le JSON est déjà exhaustif. Toute combinaison de `-v` ou `--verbose` avec `--format json` est donc
refusée avant la collecte avec l'erreur suivante sur stderr et l'exit code `2` :

```text
--verbose cannot be used with --format json
```

`--color` vaut `auto` par défaut. `auto` colore seulement lorsque stdout est un TTY et que
`NO_COLOR` est absent ou vide. `never` ne génère aucune séquence ANSI. `always` colore même une
sortie redirigée et garde priorité sur toute valeur non vide de `NO_COLOR`. La valeur de
`NO_COLOR` est traitée comme un `OsStr` : toute suite d'octets non vide, UTF-8 ou non, désactive
`auto`.

Le JSON accepte `auto` et `never`, reste toujours sans ANSI et refuse `always` avant la collecte,
avant toute lecture de `HOME`, avec stderr et exit code `2` :

```text
--color always cannot be used with --format json
```

Lorsque verbose et `always` sont tous deux incompatibles avec JSON, l'erreur verbose existante
garde priorité.

La validation porte sur les valeurs typées produites par Clap, indépendamment de l'ordre des
arguments ou des formes `--format=json` et `--color=always`. Une option inconnue, une valeur de
format ou de couleur inconnue, ou un argument dupliqué conserve le contrat d'erreur de Clap.

## Sortie humaine commune

Un rapport non vide commence par un en-tête de contexte puis un total `healthy`. Le contexte reprend
la ressource, le scope et le filtre agent réellement demandés, sans prétendre qu'un autre scope ou
agent a été audité ; tous les nouveaux libellés et messages de la CLI restent en anglais :

```text
Skills · user scope · 3 agents
✓ 50 healthy
```

La vue normale suit ces règles :

1. compter tous les diagnostics sans en changer l'ordre canonique ;
2. afficher le total `healthy` même lorsqu'il vaut zéro ;
3. masquer uniquement les lignes `healthy` ;
4. afficher intégralement `error`, `drift` et `unsupported` ;
5. qualifier `unsupported` de non bloquant sans le présenter comme une configuration cassée.

La vue verbose applique les mêmes en-têtes, compteurs, groupes et ordre de présentation, puis ajoute
les lignes `healthy` à la fin de leur section. Elle ne recalcule aucun diagnostic.

Un rapport vide affiche `No diagnostics`. Il ne doit jamais afficher un symbole vert ou laisser
entendre que l'absence de diagnostic prouve un état sain.

Quand la politique active la couleur, `ERROR` est rouge, `DRIFT` jaune, `UNSUPPORTED` cyan et
`HEALTHY` vert. Le renderer colore les libellés d'état et les segments de résumé correspondants,
jamais les messages ou détails canoniques. Un segment `issue(s)` regroupant error et drift est rouge
s'il contient au moins une error, sinon jaune. Les en-têtes restent sans couleur. Après retrait des
séquences ANSI, les octets, espaces, textes, symboles et retours à la ligne sont identiques à la
sortie `never`.

Pour les ressources sans regroupement structuré, les diagnostics visibles conservent leur ordre de
production. Le masquage des lignes saines s'applique néanmoins à toutes les ressources de
`doctor`, y compris `manifest`, `config`, `instructions`, `prompts` et `commands`.

## Vue `doctor skills`

Chaque agent forme une section avec les compteurs de ses états. Les sections sont classées selon
leur état maximal, dans l'ordre `error`, `drift`, `unsupported`, `healthy`; les égalités conservent
l'ordre de découverte. Dans une section, les diagnostics suivent ce même ordre de sévérité et
restent stables à sévérité égale. Ce tri est une projection humaine uniquement.

Vue normale représentative :

```text
$ arnes doctor skills

Skills · user scope · 3 agents
✓ 50 healthy

CURSOR
  1 issue · 3 unsupported · 16 healthy

  DRIFT enforcement-code
    expected  managed skill present
    actual    destination missing
    path      ~/.cursor/skills/enforcement-code

  UNSUPPORTED system skills inventory
  UNSUPPORTED marketplace plugin activation
  UNSUPPORTED extension skill exposure

CLAUDE
  1 unsupported · 25 healthy

  UNSUPPORTED system skills inventory

CODEX
  12 unsupported · 9 healthy

  UNSUPPORTED browser plugin version/cache
  UNSUPPORTED documents plugin version/cache
  ...
```

Vue verbose représentative :

```text
$ arnes doctor skills -v

Skills · user scope · 3 agents
✓ 50 healthy

CURSOR
  1 issue · 3 unsupported · 16 healthy

  DRIFT enforcement-code
    expected  managed skill present
    actual    destination missing
    path      ~/.cursor/skills/enforcement-code

  UNSUPPORTED system skills inventory
  UNSUPPORTED marketplace plugin activation
  UNSUPPORTED extension skill exposure

  HEALTHY merge-verdict · managed skill
  HEALTHY superpowers · enabled plugin
  HEALTHY brainstorming · enabled skill
  ...
```

Les ellipses appartiennent uniquement à cette spécification : la commande affiche chaque diagnostic
`unsupported`, et `-v` affiche chaque diagnostic `healthy`.

## Architecture de rendu

`Diagnostic` reste la donnée canonique utilisée pour le JSON et l'exit code. La présentation humaine
devient une projection distincte recevant des options de rendu, dont la verbosité, et un contexte
d'en-tête fourni par la CLI. Elle travaille sur des références aux diagnostics existants et ne
réordonne jamais le vecteur stocké dans `Report`.

La métadonnée humaine existante est enrichie par une section structurée optionnelle. Une section
porte une clé stable et un libellé ; elle ne se déduit jamais du texte affiché. Les producteurs de
diagnostics skills renseignent la section agent/scope pour les ressources gérées, système et plugin.
Les groupes plus fins, comme un plugin, restent des métadonnées de présentation imbriquées et non
des fragments à extraire du message.

Le renderer suit deux chemins :

- avec sections structurées, il calcule les compteurs et l'ordre de présentation des sections sur
  des références indexées vers les diagnostics ;
- sans section structurée, il conserve les groupes et l'ordre humain existants tout en appliquant
  le total et le filtre `healthy`.

La couche humaine reçoit une politique couleur typée, le résultat injectable de la détection TTY et
la valeur injectable de `NO_COLOR`. La CLI obtient ces valeurs avec `std::io::IsTerminal` et
`std::env::var_os`, sans dépendance supplémentaire. Le renderer associe directement chaque `State`
à son style ; il ne parse jamais une chaîne rendue pour retrouver l'état. Le texte brut est formaté
et paddé avant d'être enveloppé par ANSI afin que sa largeur logique reste inchangée.

Les champs de présentation restent exclus de Serde. `Report::json()` conserve donc exactement
`resource`, `state` et `message`, dans l'ordre de collecte. `main` valide d'abord les combinaisons
format/options, appelle ensuite `diagnose`, puis choisit le renderer ; aucune branche de rendu ne
peut modifier l'exit code déjà dérivé du rapport.

`Diagnostic::Display`, `Diagnostic`, ses messages canoniques et le chemin JSON ne contiennent aucun
code ANSI. La politique et les séquences résident uniquement dans la projection humaine.

## Erreurs et cas limites

- `-v --format json` échoue avant tout chargement du manifeste ou accès à `HOME` ;
- `--color always --format json` échoue au même endroit, sans résoudre TTY ni `NO_COLOR` ;
- `--format json --color auto|never` produit le JSON canonique sans ANSI ;
- `NO_COLOR` vide ne désactive pas `auto`, une valeur non vide le désactive et `always` la surpasse ;
- un rapport sans diagnostic dit `No diagnostics`, sans faux vert ;
- un rapport sans diagnostic sain affiche `✓ 0 healthy` puis ses autres états ;
- un rapport entièrement sain affiche son total seul en mode normal et son inventaire en verbose ;
- un agent entièrement sain reste compté globalement, disparaît de l'inventaire normal et réapparaît
  avec sa section en verbose ;
- `unsupported` reste exit `0` en l'absence d'un état plus grave et reste toujours inventorié ;
- une erreur de sortie, dont un pipe fermé, conserve le traitement I/O et l'exit code `2` actuels ;
- les chemins humains continuent d'utiliser `~` lorsque le producteur connaît `HOME`, sans modifier
  le chemin absolu conservé dans le diagnostic canonique.

## Analyse des classes d'échec

1. Atomicité et ordering : non applicable, aucune écriture ni décision transactionnelle.
2. Retry idempotence : non applicable, aucune mutation ni retry.
3. Invariant without a constraint : non applicable, aucune donnée persistée.
4. Authorization as a side effect : non applicable, aucune autorisation.
5. Tenant scope : non applicable, aucun tenant ni identifiant client.
6. Error contract : tient parce que les combinaisons verbose/JSON et always/JSON, leur priorité,
   leur stderr et leur exit code sont documentés et testés au niveau CLI avant toute collecte.
7. Deferred functionality : non applicable, la remédiation est explicitement hors contrat et aucun
   contrôle métier ou réglementaire n'est différé.
8. Parsing versus assertion : tient parce que Clap produit le booléen et les enums typés, puis la
   validation refuse les couples canoniques `(verbose, json)` et `(always, json)` sans valeur
   permissive par défaut. `NO_COLOR` n'est pas converti en UTF-8 : son emptiness est évaluée sur
   l'`OsStr`, y compris pour une valeur non UTF-8.
9. Upstream control as a trust boundary : non applicable, le rendu ne fait confiance à aucun
   allowlist upstream pour valider une ressource.
10. Claim stronger than mechanism : tient parce qu'un rapport vide ne prétend pas être sain,
    `unsupported` est décrit comme une limite d'observation non bloquante et la couleur reste un
    signal additif dont l'égalité textuelle est testée après retrait ANSI.

## Tests

Les tests unitaires du renderer couvrent :

- les totaux de chaque état et le total `healthy` nul ou non nul ;
- le masquage strict des lignes saines en mode normal ;
- leur insertion à la fin de chaque section en verbose ;
- l'ordre des sections par sévérité maximale et sa stabilité à égalité ;
- l'ordre des diagnostics par sévérité dans une section et sa stabilité à égalité ;
- le fallback sans section structurée ;
- un rapport vide sans affirmation de santé ;
- l'échappement des retours chariot et sauts de ligne existant ;
- les modes `auto`, `always` et `never` avec détection TTY injectée ;
- `NO_COLOR` absent, vide, non vide et non UTF-8, ainsi que la priorité de `always` ;
- chaque `State`, les résumés globaux et de section, en vue normale et verbose ;
- l'égalité byte-for-byte entre la sortie plate et la sortie colorée après retrait ANSI ;
- l'absence de contamination du JSON après un rendu humain coloré.

Les tests d'intégration CLI couvrent :

- `-v` et `--verbose`, avant ou après la ressource ;
- `--color` avant ou après la ressource, avec forme séparée ou `=` ;
- la sortie normale et verbose de `doctor skills` avec plusieurs agents et états ;
- une ressource entièrement saine dans les deux modes ;
- le comportement générique sur au moins une ressource autre que `skills`, dont `commands` ;
- le JSON byte-for-byte inchangé sans `-v` ;
- le JSON sans ANSI avec `auto` ou `never` ;
- le refus de `always` avec JSON avant lecture de `HOME`, dans chaque ordre d'arguments ;
- le refus de `-v --format json`, `--format=json --verbose` et de l'ordre inverse, avec stdout vide,
  stderr documenté, exit code `2` et aucun accès à `HOME` ;
- les exit codes inchangés pour `healthy`, `unsupported`, `drift` et `error` ;
- l'absence de mutation de `HOME` et du dépôt via les snapshots avant/après existants ;
- le défaut `auto`, les choix affichés dans l'aide, les valeurs inconnues et les options dupliquées ;
- `always` en redirection malgré `NO_COLOR`, et son égalité avec `never` après retrait ANSI.

Les fixtures de sortie plate historiques sont remplacées ou complétées par des fixtures qui rendent
le contrat normal et verbose intentionnel. Les tests ne doivent pas se limiter à des fragments si
l'ordre, les groupes ou les compteurs font partie du contrat.

## Vérification

Depuis `tooling/arnes`, l'implémentation devra exécuter les barrières couvrant les fichiers Rust :

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Ces résultats vaudront pour macOS dans le checkout canonique. Le workflow Arnes devra exercer Ubuntu
avant toute affirmation de portabilité ; une réussite locale ne sera pas présentée comme une preuve
Linux ou CI.

## Livraison

Aucun ADR n'est requis : le changement porte sur une option CLI et une projection humaine sans
modifier le modèle canonique, la propriété des ressources ou une décision structurelle en vigueur.
Le changement fonctionnel reste distinct de tout nettoyage adjacent. Aucun commentaire de code
n'est attendu ; si un fait externe impose exceptionnellement un commentaire, il devra être listé
dans la livraison avec sa source.
