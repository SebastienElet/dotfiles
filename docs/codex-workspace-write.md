# Codex : proposition de sandbox macOS

Le [fragment de référence](codex-workspace-write.reference.toml) propose
`workspace-write` avec réseau, caches et répertoires de développement inscriptibles.
Il n'est ni déployé ni chargé automatiquement. La configuration utilisateur locale
reste un fichier distinct ; ne pas la remplacer par ce fragment : elle contient
aussi les réglages MCP, la confiance des projets et la statusline.

## Choix du mécanisme

`home/.arnes.yaml` et `arnes sync config` gèrent actuellement le modèle, l'effort et
les fenêtres de contexte de Codex, sans champs pour la sandbox. La synchronisation
de statusline est également spécifique à cette ressource. Les tâches Codex
existantes ne fusionnent pas de fragment TOML. Une référence documentée évite
d'ajouter un moteur de déploiement pour cette proposition. Arnes conserve les
valeurs non sélectionnées lors de sa synchronisation native.

## Portée et limites

Les essais du 21 octobre 2026 concernent Codex CLI **0.163.0**, sur macOS arm64,
avec Seatbelt et OrbStack. Linux et Windows ne sont pas exercés.

- `never` renvoie les refus à l'agent sans demander d'approbation. L'aide installée
  expose aussi `on-request`, qui laisse le modèle demander une approbation.
  `on-failure` reste accepté mais `config/read` le normalise en `on-request` ; ce
  n'est donc plus une politique distincte de reprise après échec. `untrusted` est
  refusé (code 1) : `approval_policy = "untrusted" is no longer supported; remove this setting`.
- Dans une commande sandboxée, `kill` a terminé un `sleep` enfant ;
  un signal vers le processus extérieur désigné pour l'essai a échoué avec
  `Operation not permitted`. La politique Seatbelt contient
  `(allow signal (target same-sandbox))`. Cela concerne les processus partageant
  cette sandbox, pas une identité de session Codex persistante.
- `pkill -TERM -P $$ -x sleep` a échoué avec `Cannot get process list` et
  `sysmon request failed with error: sysmond service not found` ; l'enfant a
  terminé normalement. Aucune interdiction de `kill` ou `pkill` n'est ajoutée.
  Les racines inscriptibles ne corrigent pas ce refus d'énumération des processus.
  Aucune clé permettant de modifier cette partie de Seatbelt n'a été identifiée.
  L'alternative vérifiée sous Fish est de conserver le PID dès le lancement,
  puis de le signaler directement :

  ```fish
  sleep 30 &
  set child $last_pid
  kill -TERM $child
  wait $child
  ```

  Cet essai dans Seatbelt a retourné `kill_status=0`. Les formes par job ou groupe
  de processus ne sont pas couvertes par cet essai.

- Une `prefix_rule(..., decision="allow")` peut exécuter une commande **hors de
  la sandbox**, même avec `never`. Un essai avec une règle temporaire a créé un
  marqueur hors des racines inscriptibles, refusé sans cette règle. Le fichier de
  règles utilisateur original n'a pas été modifié. Pour les exécutions headless
  devant ignorer ces règles, `codex exec --ignore-rules` est disponible dans cette
  version. Ce flag ignore aussi les règles utilisateur/projet de refus ; ce
  n'est pas une clé persistante du fragment.
- Le socket Docker donne accès au daemon partagé, dont les actions ne sont pas
  confinées par la sandbox du client. Le réseau et les répertoires inscriptibles
  restent volontairement larges. Cette proposition ne garantit donc pas à elle
  seule que toute action possible d'un agent reste limitée à ses processus.

## Chemins et Git

Codex 0.163.0 développe `~` dans `writable_roots`. Les caches Bun, npm et Turbo sont
couverts explicitement ; les caches Turbo locaux restent sous les racines de
développement et ses caches utilisateur sous `.cache` ou `Library/Caches`.
`Library/pnpm` permet au gestionnaire de sélectionner sa version épinglée.
`exclude_tmpdir_env_var = false` conserve le `$TMPDIR` macOS, en plus de `/tmp`.

Codex protège les chemins `.git`, `.codex` et `.agents` immédiatement sous ses
racines de workspace. Pour un worktree lié, ajouter seulement le `.git` principal
**n'a pas suffi** : son répertoire administratif propre restait protégé.
Le wrapper Fish versionné dans `home/.config/fish/conf.d/codex.fish` ajoute
automatiquement `--add-dir` pour les deux chemins absolus fournis par Git depuis
le répertoire de travail effectif (`-C`/`--cd`, sinon le répertoire courant) :

```sh
git rev-parse --path-format=absolute --git-common-dir
git rev-parse --absolute-git-dir
```

Lorsque ces chemins sont identiques, une seule option est ajoutée. Cette injection
concerne la TUI, `exec` (alias `e`), `resume` et `fork`, conformément à leur aide
0.163.0. Pour `exec resume`, `exec fork` et `exec review`, les options sont placées
sur le parent `exec`. Les autres sous-commandes restent inchangées. Hors dépôt,
ou si Git ne fournit pas les deux chemins, les arguments d'origine sont conservés.

Un `codex exec` réel lancé via cette fonction depuis un worktree jetable a créé
un commit sans `--add-dir` manuel, avec un `CODEX_HOME` temporaire contenant
seulement cette référence et l'authentification. Le `.git` canonique dotfiles a
donc été retiré du fragment : le wrapper découvre les métadonnées du dépôt courant
au lieu de publier un inventaire de dépôts.

## Essai sans configuration live

1. Créer un répertoire temporaire avec `mktemp -d` et y copier **uniquement** le
   fragment comme `config.toml`. Pour un essai qui utilise le modèle, copier
   seulement `auth.json` si nécessaire, avec permissions restrictives ; ne pas
   copier la configuration MCP ni ses secrets.
2. Fournir ce répertoire comme `CODEX_HOME` et lancer
   `codex sandbox -c 'sandbox_mode="workspace-write"' -- <commande>`.
   L'override est indispensable : sans lui, `codex sandbox` utilise `read-only`
   avec cette configuration historique, même si le fichier indique
   `workspace-write`. Exécuter la commande depuis le workspace à tester.
3. Pour un essai headless, ouvrir une session `fish --no-config`, y charger
   uniquement `home/.config/fish/conf.d/codex.fish` depuis le checkout avec
   `source`, puis lancer depuis le worktree, avec le même `CODEX_HOME` temporaire :

   ```fish
   codex exec --strict-config --ephemeral --ignore-rules \
     --sandbox workspace-write -c 'approval_policy="never"' '<instruction>'
   ```

   Un refus de commande peut être correctement rapporté par l'agent alors que
   `codex exec` termine avec le code 0 : vérifier le résultat de la commande.

4. Retirer uniquement les marqueurs, branches, worktrees et ressources créés pour
   l'essai, puis supprimer le répertoire temporaire contenant l'authentification.

Les essais ont permis `docker ps`, `docker run --rm hello-world`, la lecture de
l'authentification GitHub et Bitbucket, et une requête HTTPS vers npm. Un worktree
jetable a permis la création Git et un commit avec les racines Git découvertes
par le wrapper ; ses hooks étaient désactivés uniquement pour ce dépôt jetable.
L'installation des dépendances avec le gestionnaire épinglé et un sous-ensemble
Turbo de dix tests ont réussi. La cible `preflight` était absente du ref testé ;
cette absence ne constitue pas une validation du preflight d'autres branches.
Pour une stack de développement partagée, inspecter d'abord Make et Compose : un nom de projet unique ne suffit
pas à isoler les ports publiés, réseaux externes, images et montages. Préférer
`docker compose -p <nom-unique> config --quiet` puis `--dry-run` lorsque le
démarrage réel peut affecter d'autres utilisateurs.

Sources : aide locale de `codex`, `codex sandbox`, `codex exec` et `codex debug` ;
[sandboxing](https://developers.openai.com/codex/sandboxing),
[règles](https://developers.openai.com/codex/agent-configuration/rules),
[référence des clés](https://developers.openai.com/codex/config-reference/),
politique embarquée `seatbelt_base_policy.sbpl` du binaire installé.
