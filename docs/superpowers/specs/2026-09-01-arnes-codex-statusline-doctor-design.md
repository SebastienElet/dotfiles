# Diagnostic de la status line Codex dans Arnes Doctor

## Intention

`arnes doctor statusline` doit vérifier statiquement la status line Codex déclarée par le dépôt.
Le diagnostic compare la liste ordonnée `tui.status_line` attendue à la configuration TOML native,
sans lancer Codex, exécuter de commande ni afficher d'autres réglages.

Claude Code et Cursor sont volontairement hors périmètre. Claude Code représente une status line
par une commande shell, dont la validité réelle ne peut pas être prouvée par une lecture statique.
Cursor expose une fonctionnalité de status line, mais ne publie pas de schéma persistant suffisant
pour construire un contrôle stable. Cette limite est documentée par un commentaire dans le code,
mais ne produit aucun diagnostic `unsupported` dans la sortie utilisateur.

## Source canonique

Le manifeste Arnes ajoute une collection normalisée `statuslines`. Chaque déclaration contient :

- `agent`, qui doit être `codex` ;
- `scope`, `user` ou `project` ;
- `items`, la liste ordonnée et non vide des identifiants attendus.

Une seule déclaration est admise par paire `(agent, scope)`. Chaque item doit être une chaîne non
vide. Les identifiants restent opaques : Arnes ne maintient pas de liste fermée, car la
documentation Codex ne garantit pas que la liste publiée soit exhaustive ou stable.

Le manifeste maintenu déclare uniquement la configuration utilisateur actuellement attendue :

```yaml
statuslines:
  - agent: codex
    scope: user
    items:
      - model-with-reasoning
      - current-dir
      - context-used
      - context-window-size
```

## Configuration observée

Le scope `user` lit `~/.codex/config.toml` et le scope `project` lit
`.codex/config.toml`. Le diagnostic extrait exclusivement `tui.status_line` et ignore toutes les
autres tables et clés.

La valeur doit être une liste TOML de chaînes. Un fichier illisible, un TOML mal formé, une table
`tui` de type inattendu, une valeur `status_line` de type inattendu ou un élément qui n'est pas une
chaîne produit un état `error`. Une configuration absente, une table ou une clé absente, ou une
liste différente produit un état `drift`. Une liste strictement égale, ordre compris, produit un
état `healthy`.

Le message d'une différence peut montrer les identifiants attendus et observés, puisqu'ils sont les
seules valeurs lues et ne sont pas des secrets. Il ne sérialise jamais le document TOML complet.

## Sélection et rendu

Le diagnostic ne considère que les déclarations du manifeste qui correspondent aux filtres
`--agent` et `--scope`. Sans filtre, `arnes doctor statusline` examine toutes les déclarations
Codex. Une combinaison Claude, Cursor ou sans déclaration ne produit aucun diagnostic de ressource :
le rendu commun reste `No diagnostics` en humain, `[]` en JSON et le code de sortie vaut `0`.

La status line n'est pas ajoutée au diagnostic agrégé `arnes doctor` dans cette issue ; cette
agrégation appartient à l'issue #123. Ce choix remplace, pour ce périmètre Codex, l'exigence
initiale de l'issue #122 qui demandait un état explicite pour les combinaisons non supportées.

## Frontières et erreurs

Le contrôle est strictement en lecture seule. Il ne lance ni Codex ni shell, ne résout aucun
exécutable, ne lit aucune variable d'environnement contenue dans ou référencée par la configuration
inspectée, ne modifie aucun fichier et ne contacte aucun réseau. `HOME`, pour résoudre le scope
utilisateur, et `NO_COLOR`, pour le rendu CLI, restent des entrées légitimes. Aucune status line ne
référence d'artefact local dans la représentation Codex prise en charge ; les contrôles de présence
et de permission prévus par l'issue initiale ne s'appliquent donc pas.

Les valeurs externes sont parsées sans valeur de repli permissive. Un format inattendu reste une
erreur observable plutôt que d'être transformé en configuration absente ou divergente. Le
diagnostic ne garantit que l'égalité statique du champ configuré ; il ne prétend pas que Codex
reconnaît chaque identifiant ni que l'interface TUI l'affichera.

## Preuves attendues

Les tests emploient des homes et dépôts isolés et exercent le vrai binaire Arnes. Ils prouvent au
minimum :

- la validation des déclarations Codex utilisateur et projet ;
- le refus de Claude, Cursor, des doublons et des listes ou items vides ;
- une liste saine et la sensibilité à l'ordre ;
- l'absence du fichier, de la table ou de la clé ;
- le TOML mal formé et chaque type inattendu sur le chemin lu ;
- le silence pour Claude, Cursor et les scopes non déclarés ;
- l'ignorance des réglages sans rapport et l'absence de mutation des fixtures.

La vérification finale couvre le formatage, le lint Rust et les tests Arnes sur macOS. Les garanties
sur d'autres plateformes restent limitées aux environnements réellement exercés par la CI.

## Limites

La synchronisation, la validation dynamique d'une status line, l'inspection de commandes Claude,
le support Cursor et l'agrégation dans `arnes doctor` restent hors périmètre. Cette conception
implémente la partie statiquement vérifiable de l'issue #122 sous le contrat en lecture seule de
l'issue parente #111.
