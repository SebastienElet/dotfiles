# Superviseur de tâches Codex

## Usage

`agent-supervisor` lance des sessions **non interactives** neuves pour une tâche explicitement
autorisée. Il consomme un checkpoint structuré et conserve mandat, candidat Git, décisions et
brouillons dans un répertoire privé extérieur au worktree. Il ne contrôle aucune saisie Herdr.

Prérequis : macOS, Git, Codex CLI avec `exec --json --output-schema`, authentification existante,
`agent-handoff@0.0.3` et CodexBar avec `usage --provider codex --source cli --json`. Les contrôles portables tournent
également sous Ubuntu. Le modèle, les permissions, le sandbox, les budgets et l'échéance sont
choisis par le propriétaire de la tâche, sans bascule automatique de fournisseur.

## Procédure

Depuis le checkout canonique `~/.dotfiles`, installer avec `moon exec harness:agent-supervisor`.
Depuis un worktree, construire et vérifier sans déploiement global selon le README racine.

Depuis le dépôt à traiter, créer un mandat dans un support privé durable déjà choisi pour la
tâche. Son répertoire parent doit exister ; le répertoire de tâche doit être absent. Adapter
objectif, permissions, propriétaire, modèle et chemins des exécutables avant lancement :

```bash
task_directory="$HOME/.local/state/my-task"
jq -n --arg repository "$PWD" --arg codex "$(command -v codex)" \
  --arg codexbar "$(command -v codexbar)" --arg handoff "$(command -v agent-handoff)" \
  '{task_id:"my-task", owner:"task-owner", repository:$repository,
    goal:"YOUR AUTHORIZED TASK", permissions:["YOUR EXPLICIT PERMISSIONS"],
    codex:$codex, codexbar:$codexbar, handoff:$handoff, model:"YOUR_MODEL", sandbox:"workspace-write",
    expires_at:((now|floor)+3600), max_turns:3, max_capacity_probes:10,
    retry_seconds:60, turn_timeout_seconds:600}' |
  agent-supervisor init "$task_directory"
agent-supervisor run "$task_directory"
```

`run` attend la capacité dans le budget indiqué et sous l'échéance du mandat. Un checkpoint
`handoff` autorise la session suivante ; `decision` et `dependency` terminent le lanceur avec
une condition de reprise lisible par `agent-supervisor status "$task_directory"`.
Le paquet fournit le schéma, la validation et sa propre skill via `note-schema`, `note-validate`
et `skill-path`. Le workflow expose le contexte courant et la destination par tentative :
la skill y publie sa note avec `note-write`. Une autre tâche ou des permissions différentes
sont refusées avant toute continuation. Une réponse finale seule ne suffit pas pour un handoff.
Après décision humaine ou événement externe vérifié, fournir la réponse et le numéro
`attempts` affiché par `status`, puis relancer :

```bash
printf '%s\n' '{"attempt":1,"answer":"DECISION OR OBSERVED EVENT"}' |
  agent-supervisor resolve "$task_directory"
agent-supervisor run "$task_directory"
```

Pour arrêter depuis un autre terminal : `agent-supervisor stop "$task_directory"`.
`SIGINT`, `SIGTERM` et `SIGHUP` demandent aussi l'arrêt du superviseur actif. Un mandat arrêté
ne peut pas être réactivé par `resolve`. Une nouvelle tâche reçoit un nouveau mandat et un
nouveau répertoire ; ne pas copier l'ancien état comme source d'autorité.

## Limites et vérification

- Les reçus séparent la soumission acceptée (`accepted`), le démarrage observé (`started`
  avec `fresh_thread`) et la complétion (`completed`). Ils conservent ces observations même
  lorsqu'une interruption, une sortie tronquée ou un résultat invalide laisse la tentative
  `unconfirmed`, sans renvoi automatique.
- Les continuations provoquées par Stop restent dans le même processus. `turns_started`
  les compte lorsqu'elles sont exposées comme des événements distincts par l'hôte ; un nouveau
  thread reste nécessaire pour confirmer la session suivante. Codex conserve son transcript natif
  afin que le hook puisse lire l'usage ; ce transcript reste géré par Codex.
- Le verrou protège les superviseurs de cette commande sur le même worktree. Il n'exclut pas
  les éditions humaines ou un autre outil. Une branche ou un HEAD modifié hors du tour est refusé.
- Un arrêt tue le groupe de processus possédé et attend sa sortie avant de restituer l'état.
  `SIGKILL`, une panne de machine et les descendants qui se détachent du groupe ne sont pas
  rattrapables par ce contrat. Après interruption brutale, inspecter les processus et le candidat
  avant tout nouveau mandat ; un processus orphelin peut encore travailler.
- Les brouillons et la décision survivent à la suppression du worktree. Le lanceur ne recrée
  pas le checkout ; une cible absente empêche le lancement. Les fichiers privés restent à la
  charge du propriétaire, sans export télémétrique ni purge automatique.
- `status` et la sortie de `run` contiennent des informations privées. Ne pas les publier tels
  quels. Les événements d'items et stderr des sous-processus sont jetés ; les checkpoints sont
  du travail privé, distinct du store Arnes. `complete` est une déclaration de l'agent et
  ne remplace pas l'oracle d'acceptation du projet.
- La skill et le hook restent fournis par leur dépôt propriétaire. Ce lanceur consomme leurs
  notes pour les mandats opt-in, sans activation de la reprise des sessions interactives déjà ouvertes.
  Il ne corrige pas une gate Stop externe dont le producteur n'est pas identifié.

Exécuter `moon run supervisor:check` dans un environnement préparé : Rustfmt, Clippy et tests
du véritable point d'entrée avec processus et stockage locaux. Les doubles Codex/CodexBar
exercent le protocole et les refus ; ils ne qualifient ni un quota fournisseur réel ni le
comportement d'une session au seuil de contexte. Les essais live restent séparés et bornés.
