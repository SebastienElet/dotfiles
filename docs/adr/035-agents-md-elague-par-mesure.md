# ADR-035 — `AGENTS.md` élagué par mesure du no-op

- **Statut** : accepté
- **Date** : 2026-08
- **Commits** : `#50`

## Contexte

L'`AGENTS.md` racine avait atteint 32 lignes, dont plusieurs redisaient ce que
`docs/AGENTS.md`, `docs/adr/README.md` et les descriptions de skills portent
déjà — au contraire de ce que pose l'[ADR-025](025-agents-md-source-unique.md),
« une règle s'écrit à un seul endroit ». Le rappel de la règle de langue était
même documenté comme une duplication volontaire, « pour qu'un agent la
connaisse avant même d'ouvrir un fichier du répertoire ».

Le *test du no-op* de Matt Pocock donne le critère : supprimer cette ligne
change-t-il le comportement par rapport au comportement par défaut ? Il est
comportemental et non esthétique, et il ne se tranche qu'en exécutant le
document. Le protocole reprend celui appliqué au coffre `~/Brain` le
2026-08-07 : 36 exécutions Opus 5 via `claude -p`, un clone jetable par run,
`AGENTS.md` et `CLAUDE.md` retirés du disque en condition *sans*, hook `Stop`
du dépôt neutralisé dans les deux conditions, six scénarios × deux conditions ×
trois réplicats.

Résultats, condition *avec* contre condition *sans* :

| Règle | Avec | Sans |
|---|---|---|
| Lire l'ADR concernée avant un changement structurel | 3/3 | 3/3 |
| Écrire une ADR pour une décision structurante | 1/3 | 2/3 |
| Rédiger en français sous `docs/` | 3/3 | 3/3 |
| Créer les skills dans `.agents/skills/` | 3/3 | 3/3 |
| Tenir à jour l'index `.agents/skills/README.md` | 3/3 | 3/3 |
| Invoquer `skill-manager` | 3/3 | 3/3 |

Un run privé du fichier a même énoncé le conflit ADR-010/ADR-012 plus
complètement qu'un run qui l'avait en contexte. Ce qui porte réellement ces
comportements est mesurable : `docs/AGENTS.md`, ouvert spontanément (3/3 sur le
scénario de rédaction, 2/3 sur celui de décision structurante),
`docs/adr/README.md`, l'arborescence de `docs/adr/`, et les descriptions des
skills `dotfiles`, `neovim` et `skill-manager`, qui se déclenchent sans routeur.

## Décision

Une règle n'est gardée dans l'`AGENTS.md` racine que si son retrait change le
comportement, constaté par exécution et non supposé. Les six règles ci-dessus
sont retirées ; le fichier passe de 32 à 26 lignes.

Restent, faute de mesure les infirmant : la règle de conflit, les trois règles
de portée, la définition de `docs/adr/`, l'interdiction de contredire une ADR
en silence, la dispense pour les changements de routine, les deux règles
`~/Brain` et l'interdiction de dupliquer une règle dans un répertoire propre à
un agent.

## Conséquences

- Réinscrire une règle retirée demande une mesure qui la montre portante, non
  une intuition. Le mode d'échec inverse — le sédiment, ces couches qu'on
  ajoute parce que retirer paraît dangereux — est ce que ce critère arrête.
- `docs/adr/README.md` ne promet plus le rappel de la règle de langue dans
  l'`AGENTS.md` racine.
- La mesure vaut pour Opus 5 sous Claude Code, skills disponibles. Un agent
  sans mécanisme de skills — Codex lit le même fichier — peut avoir un
  comportement par défaut différent : une divergence constatée là-bas se
  traite par une mesure sur cet agent, pas par une réinscription d'office.
- La seule règle de portée mesurée porte au bloc, pas à la ligne : sans le
  fichier, deux runs sur trois s'arrêtent pour demander le périmètre au lieu de
  livrer. L'ablation menée est au fichier ; attribuer cet effet à une phrase
  précise demanderait deux ablations à la ligne.

## Alternatives écartées

- **Garder par précaution** : c'est la définition du sédiment, et le coût est
  payé à chaque tour de chaque agent.
- **Déplacer les six règles dans les skills** : elles y sont déjà, ou l'index
  et l'arborescence les portent ; les y réécrire recréerait la duplication que
  l'[ADR-025](025-agents-md-source-unique.md) refuse.
- **Raccourcir le fichier à l'œil** : un agent à qui l'on demande d'alléger
  optimise la longueur, seule chose qu'il voit, et coupe la fonctionnalité avec
  le reste.
