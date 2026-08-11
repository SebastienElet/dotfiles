# ADR-037 — Contrôle du commentaire au niveau des agents

- **Statut** : accepté
- **Date** : 2026-08

## Contexte

`ai/AGENTS.md` énonce « Only comment when explaining *why*, not *what* ». La règle était en
contexte et a été violée trois fois dans la PR #51 (issue #52). L'[ADR-036](036-regles-ia-admises-par-ablation.md)
donne le critère : une règle ne lie que si elle exige un **acte observable**, dont l'absence se
voit dans la réponse finale. Une propriété de la prose, jugée pendant qu'on l'écrit, n'en laisse
aucun. Ajouter des mots à `ai/AGENTS.md` ne changerait donc rien, et c'est le sédiment que
l'[ADR-035](035-agents-md-elague-par-mesure.md) arrête. Il faut une couche qui s'exécute.

Trois emplacements pouvaient l'accueillir :

- **La CI** ne voit que ce dépôt, alors que la règle s'applique partout où l'agent écrit du code.
- **La chaîne `git_hook_push`** ([ADR-020](020-hooks-de-push-maison.md)) n'est atteinte que par
  l'alias fish `gp` ; tout `git push` direct la contourne. Mesure rapportée dans l'issue #52 :
  elle est muette depuis `6af94b9` (2026-01-21), sept mois sans qu'un seul assert s'exécute, et
  sans que rien ne le signale. L'état de `scripts/` est traité par l'issue #54.
- **Les hooks d'agent** s'exécutent dans toutes les sessions et tous les projets, au moment où le
  commentaire est écrit — le seul moment où le corriger est gratuit.

L'[ADR-025](025-agents-md-source-unique.md) faisant de `AGENTS.md` une source agent-agnostique
déployée vers trois agents, un contrôle qui n'en lierait qu'un laisserait la règle sans effet là
où elle s'applique quand même.

## Décision

`scripts/agent_comment_block_check <agent>` refuse un bloc de commentaires **ajouté** de plus de
trois lignes. Un seul chemin déployé, `~/.local/bin/`, et une entrée par agent :

| Agent | Fichier | Événement | Outil observé |
|---|---|---|---|
| Claude Code | `~/.claude/settings.json` | `PostToolUse` | `Edit`, `Write` |
| Codex | `~/.codex/hooks.json` | `PostToolUse` | `apply_patch` |
| Cursor | `~/.cursor/hooks.json` | `afterFileEdit` | — |

- **Opt-out dans le fichier** : `Why:` en première ligne du bloc. L'échappatoire laisse une trace
  dans le diff, contrairement à une variable d'environnement.
- **Couche consultative** : « pourquoi » contre « quoi » n'est pas décidable mécaniquement, et la
  sortie du hook le dit à chaque refus.
- **Canari observable** : le détecteur passe un fixture connu à chaque invocation *et* imprime sa
  confirmation dans le flux. Un `awk` cassé, absent ou qui répondrait sans exécuter le programme
  ne peut donc pas se lire comme « aucune violation ».
- **Fermé par défaut** : outil absent, entrée non-JSON, champ manquant, agent inconnu, matcher
  câblé sur un autre outil — chacun refuse en nommant la cause.
- **Sortie par code 2 et `stderr`**, contrat documenté commun à Claude et Codex : le hook ne peut
  pas annuler l'écriture, son `stderr` est renvoyé à l'agent.
- **Prose hors périmètre** (`.md` et apparentés) : la règle porte sur les commentaires du code.

L'enregistrement est fait par les cibles `claude-code`, `codex` et `cursor` (`--register`), par
fusion `jq` idempotente : un contrôle que personne n'a câblé est un contrôle que personne
n'exécute, et c'est exactement la panne que ce dépôt vient de payer sept mois. Les trois fichiers
portent de l'état local sans rapport avec ce dépôt : ils sont réécrits en place, par exception au
déploiement par symlinks ([ADR-003](003-deploiement-par-symlinks.md)).

### Substrat

Bash, avec `jq` pour les adaptateurs de payload et une passe `awk` pour la détection et la
soustraction ancien/nouveau. 119 lignes pour les trois agents, deux dépendances.

Une première version faisait 158 lignes pour un seul agent et six dépendances : elle écrivait en
bash ce que `jq` et `awk` font chacun en une fois, et son canari coûtait 42 lignes pour ce que six
suffisent à prouver. Node et Rust ont été envisagés pour échapper à ce volume ; ils ne le
réduisent pas une fois la structure corrigée, et ils déplacent l'interpréteur dans la surface de
défaillance : un `node` ou un binaire absent sort en 127, que le contrat documente comme *erreur
non bloquante* — donc un échec **ouvert**, précisément la panne visée. Bash est le seul
interpréteur qui ne peut pas manquer sur les plateformes supportées.

## Conséquences

- Coût mesuré sur ce dépôt : réécrits d'un seul bloc, 9 des 67 fichiers de code suivis seraient
  refusés — mesure identique avant et après la réécriture. Les blocs en cause sont pour
  l'essentiel de vrais « pourquoi » — règles de portée de fish, coffre local 1Password, rationnel
  du hook de handoff — qui devront porter le préfixe `Why:`. C'est le prix de l'opt-out.
- **Codex exige une approbation interactive** : chaque entrée de `hooks.json` doit être confiée,
  et la confiance est un `trusted_hash` inscrit dans `config.toml`. Écrire l'entrée ne suffit
  donc pas ; la première session Codex la proposera. Tant qu'elle n'est pas accordée, le contrôle
  est absent de cet agent — sans le dire.
- **L'adaptateur Codex n'est pas vérifié empiriquement** : cette même garde de confiance interdit
  une capture non interactive du payload réel. Il est écrit sur le contrat documenté
  (`tool_input.command` portant le patch) et refuse en nommant la cause s'il n'y trouve pas de
  patch — jamais en silence.
- **Cursor est le maillon faible** : `afterFileEdit` a le bon payload mais est documenté comme
  observationnel, sans schéma de sortie ; `postToolUse`, qui accepte `additional_context`, est
  orienté MCP. La couche y dégrade probablement en trace, sans retour à l'agent. À reprendre si
  Cursor documente une sortie.
- Une `Write` qui écrase un fichier existant traite tout son contenu comme ajouté : au moment où
  le hook s'exécute, l'ancien contenu n'existe plus. `Edit`, `afterFileEdit` et `apply_patch`
  n'ont pas ce défaut, le texte retiré y est disponible.
- Contournements connus, tous assumés puisque la couche est consultative : un bloc réparti sur
  deux éditions, une écriture par shell, un marqueur non reconnu (`;`, `%`, `<!-- -->`), la
  désactivation globale des hooks, un `Why:` mensonger. Les trois premiers sont des cas du test.
- Le hook s'exécute à chaque édition de chaque session, y compris hors de ce dépôt.
- `scripts/agent_comment_block_check_test` (34 cas) s'exécute dans le job CI `Script self-checks`
  (`macos-latest`), qui câble au passage `scripts/claude_handoff_check_test`, orphelin depuis sa
  création.

## Alternatives écartées

- **Un job de CI dans ce dépôt** : ne verrait que ce dépôt, pour une règle globale.
- **Un assert dans la chaîne `git_hook_push`** : option 1 de l'issue #52, réfutée par la mesure
  ci-dessus — la chaîne n'exécutait plus rien depuis sept mois.
- **Un hook `pre-push` par `core.hooksPath`** : complément utile pour les éditions à la main, mais
  il dépend du sort de `scripts/git_hook_push` (#54) ; reportable sans coût.
- **Node ou Rust** : voir *Substrat*. Rust ajoute en outre un artefact compilé, une cible de
  toolchain et une construction dans `make all`, donc dans le smoke test, pour un démarrage en
  microsecondes dont rien ici n'a besoin.
- **`ast-grep` ou tree-sitter** : des nœuds de commentaire réels au lieu d'une heuristique de
  marqueurs, donc moins de faux positifs sur un `#` en chaîne ou en heredoc. Écarté pour une
  dépendance de plus, un `kind` par langage, et parce que la plomberie du hook resterait entière.
- **ESLint, Semgrep, Vale** : le premier ne voit que JS quand le cas déclencheur était un YAML et
  un Makefile ; le deuxième ignore les commentaires, absents de l'AST ; le troisième lint la
  prose, pas la longueur des blocs.
- **Opt-out par variable d'environnement** : ne laisse aucune trace dans le fichier.
- **Un hook de type `prompt` ou `agent`**, qui jugerait vraiment « pourquoi contre quoi » : un
  appel modèle à chaque édition, pour un verdict non reproductible sur une couche consultative.
- **La clause de destination** (option 2 de l'issue #52) : hors périmètre tant que l'ablation
  marginale à six runs de l'ADR-036 n'a pas été menée.
