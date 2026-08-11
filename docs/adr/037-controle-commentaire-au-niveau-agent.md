# ADR-037 — Contrôle du commentaire au niveau de l'agent

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
- **Le hook d'agent** s'exécute dans toutes les sessions et tous les projets, au moment où le
  commentaire est écrit — le seul moment où le corriger est gratuit.

## Décision

`scripts/claude_comment_block_check`, hook `PostToolUse` sur `Edit|Write`, refuse un bloc de
commentaires **ajouté** de plus de trois lignes.

- **Opt-out dans le fichier** : `Why:` en première ligne du bloc. L'échappatoire laisse une trace
  dans le diff, contrairement à une variable d'environnement.
- **Couche consultative** : « pourquoi » contre « quoi » n'est pas décidable mécaniquement, et la
  sortie du hook le dit à chaque refus.
- **Canari à chaque invocation** : le détecteur est passé sur un fixture connu — quatre blocs
  fautifs (`#`, `//`, `--`, `/* */`), un bloc exempté, un bloc de trois lignes — et toute
  divergence refuse. Un détecteur cassé par une différence d'`awk` ou de locale ne peut donc pas
  laisser passer en silence.
- **Fermé par défaut** : outil absent, entrée non-JSON, champ manquant, matcher câblé sur un autre
  outil — chacun refuse en nommant la cause. Aucun `command -v` qui saute le contrôle.
- **Sortie par code 2 et `stderr`**, conformément au contrat documenté : un `PostToolUse` ne peut
  pas annuler l'écriture, et son `stderr` est renvoyé à l'agent. Rien à sérialiser, donc rien à
  produire quand `jq` manque.
- **Prose hors périmètre** (`.md` et apparentés) : la règle porte sur les commentaires du code.

L'enregistrement dans `~/.claude/settings.json` est fait par la cible `claude-code`, par fusion
`jq` idempotente, et non à la main comme celui du hook `Stop` : un contrôle que personne n'a câblé
est un contrôle que personne n'exécute, et c'est exactement la panne que ce dépôt vient de payer
sept mois. Le fichier porte de l'état local sans rapport avec ce dépôt : il est donc réécrit en
place, par exception au déploiement par symlinks ([ADR-003](003-deploiement-par-symlinks.md)).

## Conséquences

- Coût mesuré sur ce dépôt : réécrits d'un seul bloc, 9 des 67 fichiers de code suivis seraient
  refusés. Les blocs en cause sont pour l'essentiel de vrais « pourquoi » — règles de portée de
  fish, coffre local 1Password, rationnel du hook de handoff — qui devront porter le préfixe
  `Why:`. C'est le prix de l'opt-out, payé sur du code déjà écrit dès qu'on le réécrit.
- Une `Write` qui écrase un fichier existant traite tout son contenu comme ajouté : au moment où
  le hook s'exécute, l'ancien contenu n'existe plus. Une `Edit` n'est pas concernée, les blocs
  déjà présents dans `old_string` sont soustraits.
- Contournements connus, tous assumés puisque la couche est consultative : un bloc réparti sur
  deux éditions, une écriture par `Bash`, un marqueur non reconnu (`;`, `%`, `<!-- -->`),
  `disableAllHooks`, un `Why:` mensonger. Les trois premiers sont des cas du test ; les deux
  derniers ne sont pas mesurables ici.
- Le hook s'exécute à chaque `Edit`/`Write` de chaque session, y compris hors de ce dépôt.
- Codex ne lit pas `~/.claude/settings.json` : la règle reste sans contrôle de son côté.
- `scripts/claude_comment_block_check_test` s'exécute dans le job CI `Script self-checks`
  (`macos-latest`), qui câble au passage `scripts/claude_handoff_check_test`, orphelin depuis sa
  création.

## Alternatives écartées

- **Un job de CI dans ce dépôt** : ne verrait que ce dépôt, pour une règle globale.
- **Un assert dans la chaîne `git_hook_push`** : option 1 de l'issue #52, réfutée par la mesure
  ci-dessus — la chaîne n'exécutait plus rien depuis sept mois.
- **Un hook `pre-push` par `core.hooksPath`** : complément utile pour les éditions à la main, mais
  il dépend du sort de `scripts/git_hook_push` (#54) ; reportable sans coût.
- **Opt-out par variable d'environnement** : ne laisse aucune trace dans le fichier, donc rien à
  voir en revue.
- **Un hook de type `prompt` ou `agent`**, qui jugerait vraiment « pourquoi contre quoi » : un
  appel modèle à chaque édition, pour un verdict non reproductible sur une couche consultative.
- **La clause de destination** (option 2 de l'issue #52) : hors périmètre tant que l'ablation
  marginale à six runs de l'ADR-036 n'a pas été menée.
