# Migration remem : découverte et blocage

État au 21 septembre 2026 : **migration non déployée**. La version publiée
`v0.6.93` ne partage pas automatiquement l'identité mémoire entre un checkout
principal et ses worktrees. La demande exclut un adaptateur maison et toute
modification de remem ; la bascule et le retrait de l'ancienne mémoire restent
donc suspendus. Cette note est un résultat de découverte, pas une nouvelle ADR.

## Version et exécuteur vérifiés

- [Release v0.6.93](https://github.com/majiayu000/remem/releases/tag/v0.6.93), publiée
  le 8 septembre 2026 ; commit `dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d`.
- Binaire publié `remem-darwin-arm64.tar.gz`, version affichée `0.6.93`, schéma
  `v92`. SHA-256 vérifié contre le fichier publié `SHA256SUMS` :
  `246b5429fa2b141424ba95186b2f2317f796c0f4b27b1c14ee5d8b9a10167d81`.
- Exécuteur `codex-cli`, modèle explicite `gpt-5.6-luna`, raisonnement `low`.
  Les profils des deux hôtes pointent vers le même profil Codex dans la base
  temporaire. Trois diagnostics natifs `remem model test --live` répondent `ok`.
- Le processus observé lance réellement la CLI officielle installée :

  ```text
  codex --ask-for-approval never exec --ephemeral --ignore-user-config --ignore-rules --skip-git-repo-check --sandbox read-only --json --output-last-message <fichier-temporaire> --model gpt-5.6-luna -c model_reasoning_effort="low" -
  ```

L'[exécuteur publié](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/ai/codex_cli.rs)
laisse la CLI gérer l'authentification et fixe `REMEM_DISABLE_HOOKS=1` dans son
enfant. Le [routeur](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/ai.rs)
propage l'erreur de l'exécuteur sélectionné, sans bascule automatique vers HTTP.
Aucun fichier d'authentification ni secret du trousseau n'a été lu par le diagnostic.

## Source canonique et inventaire local

| Composant                     | Source ou destination                                                                                                                                          | Traitement prévu si la migration devient possible                                                    |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Moteur actuel                 | `tooling/agent-memory/`, tâches Moon, binaire `~/.local/bin/agent-memory`                                                                                      | Remplacer après validation du nouveau parcours                                                       |
| Souvenirs du moteur           | `~/.local/share/agent-memory/`                                                                                                                                 | `agent-memory audit --include-terminal --format json` retourne zéro entrée ; réauditer avant bascule |
| Hooks mémoire Claude et Codex | Arnes, sous `tooling/arnes/src/hooks/`, appelé par `.moon/tasks/harness-{claude,codex}.yml`                                                                    | Remplacer les handlers `UserPromptSubmit` dédiés à `agent-memory` ; préserver les autres hooks       |
| Configuration générée         | `~/.claude/settings.json`, `~/.codex/hooks.json`                                                                                                               | Régénérer par le déploiement canonique, pas uniquement éditer les destinations                       |
| Gouvernance mémoire           | `harness/skills/memory-governance/` et règle Cursor `harness/rules/memory-governance-cursor.mdc`                                                               | Réconcilier tous les consommateurs avant retrait du moteur partagé                                   |
| Déploiement des skills        | `tooling/install-agent-skills.ts`, tâches Moon ; Make pour Cursor                                                                                              | Préserver les skills métier ; retirer uniquement les références remplacées                           |
| Mémoire native Claude         | Neuf fichiers Markdown dans cinq répertoires `~/.claude/projects/*/memory/`                                                                                    | Sauvegarder et examiner leur provenance avant toute sélection ou migration                           |
| Mémoire native Codex          | `~/.codex/memories/` vide ; `codex features list` indique `memories=false`                                                                                     | Aucun souvenir natif à importer dans cet état local                                                  |
| MCP et tâches de fond         | Aucun enregistrement remem trouvé dans les configurations des agents inspectées ; aucun LaunchAgent mémoire trouvé ; répertoire `~/.codex/automations/` absent | Ne pas confondre cet inventaire local avec un inventaire des services distants                       |
| Instructions stables          | `harness/AGENTS.md`, `SOUL.md`, `USER.md` ; liens Claude et assemblage `~/.codex/AGENTS.md`                                                                    | Préserver                                                                                            |
| Relais et suivi des tâches    | `agent-handoff`, documents de projet, notes Obsidian                                                                                                           | Préserver : hors mécanisme remplacé                                                                  |

Les [ADR-003](adr/003-deploiement-par-symlinks.md) et
[ADR-038](adr/038-frontieres-home-harness-tooling.md) définissent le déploiement.
L'[ADR-042](adr/042-memoire-durable-locale-partagee.md), encore acceptée, décrit
le moteur actuel et son identité fondée sur `git-common-dir`. Une migration
effective devra remplacer cette décision explicitement.

## Blocage : identité des worktrees

Dans la version publiée, [l'identité projet](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/project_id.rs#L24-L33)
est la racine du worktree. La résolution utilise
[`git rev-parse --show-toplevel`](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/git_util.rs#L358-L363).
Le [test upstream](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/src/project_id.rs#L491-L501)
attend explicitement le chemin du worktree lié.

Les alias existent dans la bibliothèque, mais leur écriture est exposée par
[un exemple Rust](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/examples/project_alias_apply.rs),
pas par une commande du binaire publié ni un outil MCP. Ils demandent une
correspondance exacte préenregistrée et ne regroupent pas les nouveaux worktrees.
Forcer `--project` lors d'une recherche ponctuelle ne corrige pas l'identité
utilisée par les hooks de capture.

Reproduction avec le binaire publié, une base SQLCipher temporaire et des données
fictives :

| Répertoire interrogé | Identité affichée par `context --debug` | Résultats de `search NEBULA-742 --project … --json` |
| -------------------- | --------------------------------------- | --------------------------------------------------- |
| Checkout A           | Chemin absolu de A                      | 2                                                   |
| Worktree lié à A     | Chemin absolu du worktree               | 0                                                   |
| Dépôt indépendant B  | Chemin absolu de B                      | 0                                                   |

Les deux entrées ont été créées par les interfaces publiques `preferences add`
et MCP `save_memory`, puis retrouvées par un autre processus et exportées en
Markdown. Elles sont classées `legacy_unverified`, avec
`current_context_eligible=false` : aucun rappel automatique n'est démontré par
ces écritures manuelles. Le résultat sur B établit seulement l'isolation de
cette recherche explicitement filtrée.

La reprise exige une version publiée offrant une identité commune native pour
les worktrees, ou une modification explicite du besoin. Ne pas installer un
wrapper, compiler un outil d'alias permanent ou supprimer le filtre projet pour
masquer cette limite.

## Vérifications et limites

Environnement exercé : macOS arm64, Codex CLI `0.154.0`, Claude Code `2.1.236`.
L'application de bureau se trouve dans `ChatGPT.app`, version `26.908.70816`,
avec le Codex embarqué `0.154.0-alpha.6.2`.

La [note upstream sur Desktop](https://github.com/majiayu000/remem/blob/dc5bfc562a0eef4f652a9e6d9aedc5c6f9ab7e1d/docs/research/codex-app-sessionstart-visibility-2026-06-29.md)
rapporte une injection `SessionStart` visible du modèle, mais pas un bloc visible
dans la conversation. Elle concerne une version antérieure et ne valide pas
l'application installée ici. Aucun parcours Desktop réel n'a été lancé.

- Recherche CLI : environ 20 ms par invocation sur cette base de deux entrées.
  Rendu de contexte sans mémoire admissible : 40–50 ms. Ce ne sont ni une mesure
  de rappel dans un agent ni une démonstration de gain de qualité.
- Trois appels LLM de diagnostic ont réussi. Le premier précède l'initialisation
  SQLCipher et son usage n'a pas été enregistré, avec avertissement explicite.
  `remem usage` rapporte pour les deux autres : 10 514 tokens d'entrée, 27 136
  de cache lu et 10 de sortie, soit 37 660 selon sa comptabilité. Le tarif du
  modèle est inconnu de remem ; le montant affiché `0.0000` ne prouve pas un coût.
- Quota Codex du compte : 84 % disponibles avant les sondes, 83 % ensuite sur
  la fenêtre hebdomadaire. Ce delta inclut le travail de la session et toute
  utilisation concurrente ; il n'est pas attribuable aux seuls diagnostics.
- `remem doctor` inspecte la base, mais retourne un échec global : hooks/MCP non
  installés, contrôle Cursor en échec et enrichissement des entrées en attente.
  Aucun état « installation saine » n'est revendiqué.
- Capture Claude → nouvelle session Codex et trajet inverse, correction ultérieure,
  reprise après redémarrage de l'agent et retrait de l'ancienne mémoire : **non
  exécutés**, car la bascule dépend de l'identité projet bloquée.
- Les limites de 90 s par appel LLM, les baux, les tentatives bornées et le budget
  `worker --once` sont présents dans le code publié. Le budget de 180 s est
  vérifié entre éléments de travail, pas une borne absolue sur un appel en cours.
  Aucun scénario de quota épuisé ou de crash worker n'a été exercé.

## Artefacts et commandes de diagnostic

Aucun composant retiré, aucun souvenir utilisateur migré, aucune configuration
du harnais modifiée. Les neuf fichiers Claude sont conservés ; leur utilité n'a
pas été évaluée. Aucune sauvegarde de migration n'a été créée, puisqu'aucune
écriture sur ces données ou configurations n'a été engagée. Une sauvegarde
vérifiée reste un préalable à toute bascule ultérieure.

Le téléchargement, la configuration de test, la base chiffrée, les captures de
diagnostic et l'export fictif sont locaux, hors Git, sous
`/tmp/remem-discovery-20260921/`. Ce chemin temporaire n'est pas une sauvegarde
durable. Aucun service de fond n'a été installé.

Pour consulter ces seuls artefacts tant qu'ils existent :

```sh
export REMEM_DATA_DIR=/tmp/remem-discovery-20260921/store
/tmp/remem-discovery-20260921/remem model current
/tmp/remem-discovery-20260921/remem search NEBULA-742 --project /private/tmp/remem-discovery-20260921/project-a --json
/tmp/remem-discovery-20260921/remem status --json
/tmp/remem-discovery-20260921/remem doctor
/tmp/remem-discovery-20260921/remem usage
/tmp/remem-discovery-20260921/remem export --markdown --output /tmp/remem-discovery-20260921/export --project /private/tmp/remem-discovery-20260921/project-a
```

Retour arrière : aucune action nécessaire sur le harnais. Fermer le shell de
diagnostic retire la variable exportée ; les artefacts temporaires peuvent être
supprimés après consultation. L'installation reproductible par Moon et le choix
du canal macOS restent à effectuer seulement lorsque le blocage est levé.
