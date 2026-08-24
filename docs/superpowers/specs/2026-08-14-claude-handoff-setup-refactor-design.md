# Refactor du setup du hook de handoff Claude

## Objectif

Déclarer les hooks de harness dans le manifeste et les réconcilier par agent dans une seule mutation
sûre. Le setup reste distinct des callbacks runtime de mesure et de handoff.

## Structure

- `home/.arnes.yaml` est la source déclarative des capacités `measurement` et `handoff` et de leurs
  installations par agent.
- `arnes setup hooks --agent <agent>` charge ce manifeste puis délègue le format natif à l'adapter
  Claude, Codex ou Cursor.
- `arnes measure hook` reste uniquement le callback runtime de la capacité de mesure.
- Make déploie les exécutables stables sous `~/.local/bin`, puis appelle le setup Arnes sans porter
  de politique de hook ni de chemin de migration.

Le manifeste ne reproduit aucun JSON fournisseur : les événements, variantes, propriétés possédées
et migrations restent typés dans les adapters Rust.

## Garanties

Arnes crée la configuration native si nécessaire, préserve les réglages sans rapport avec ses
entrées, retire les capacités absentes du manifeste, migre les anciens chemins et doublons, puis
reste idempotent. Il refuse les manifestes, structures ou exécutables invalides sans mutation. Son
échange atomique compare le fichier remplacé au snapshot lu et restaure une écriture concurrente au
lieu de l'écraser.

Les événements et variantes connus de chaque agent sont validés avant mutation; les extensions
inconnues restent opaques et sont conservées.

## Tests et périmètre

Les scénarios couvrent la validation du manifeste, la sélection par agent, la création, le retrait,
la préservation, la migration, les doublons, l'idempotence, les entrées invalides et les écritures
concurrentes. Aucun script Shell ne lit ou n'écrit une configuration native d'agent.
