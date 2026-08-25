# Architecture du dépôt

Ce document décrit l'organisation actuelle du dépôt et le chemin suivi par ses
artefacts jusqu'à leur destination. Les raisons des choix structurants restent
consignées dans les [ADR](adr/README.md), notamment
l'[ADR-038](adr/038-frontieres-home-harness-tooling.md).

## Vue d'ensemble

| Chemin                            | Responsabilité                                                                       |
| --------------------------------- | ------------------------------------------------------------------------------------ |
| `home/`                           | Configuration déployée sous `$HOME`, avec le même chemin relatif que sa destination. |
| `harness/`                        | Instructions, skills user et services partagés entre les différents agents.          |
| `tooling/`                        | Applications locales et exécutables maintenus par ce dépôt.                          |
| `.agents/skills/`                 | Source canonique des skills limitées à ce dépôt.                                     |
| `.claude/`, `.codex/`, `.cursor/` | Adaptateurs placés aux chemins de découverte imposés par chaque agent.               |
| `.github/workflows/`              | Barrières de lint, de test et d'installation.                                        |
| `docs/adr/`                       | Décisions d'architecture encore en vigueur.                                          |

Les points d'entrée restent à la racine :

- `Makefile` est l'installateur canonique et décrit les destinations déployées ;
- `install.sh` amorce une nouvelle machine en clonant le dépôt puis en lançant
  `make all` ;
- `AGENTS.md` porte les instructions de contribution communes, avec
  `CLAUDE.md` comme adaptateur ;
- `.mcp.json` déclare les serveurs MCP découverts depuis la racine ;
- `README.md` expose l'installation et oriente vers cette documentation.

## Configuration du répertoire personnel

`home/` reproduit le chemin relatif de chaque artefact sous `$HOME` :

| Source                             | Destination                     |
| ---------------------------------- | ------------------------------- |
| `home/.config/fish/`               | `~/.config/fish/`               |
| `home/.config/git/config.delta`    | `~/.config/git/config.delta`    |
| `home/.config/git/ignore`          | `~/.config/git/ignore`          |
| `home/.config/nvim/`               | `~/.config/nvim/`               |
| `home/.config/starship.toml`       | `~/.config/starship.toml`       |
| `home/.config/tmux/tmux.conf`      | `~/.config/tmux/tmux.conf`      |
| `home/.config/wezterm/wezterm.lua` | `~/.config/wezterm/wezterm.lua` |
| `home/cspell.json`                 | `~/cspell.json`                 |

Le `Makefile` déploie les artefacts statiques comme cibles fichier ordinaires
et lie les exécutables générés après leur build. Chaque recette vérifie que la
destination est absente avant `ln -s`. La réparation explicite d'un lien erroné
est suivie par l'issue #152.

## Intégrations d'agents

`harness/` contient les sources communes `AGENTS.md`, `SOUL.md` et `USER.md`,
les skills et leurs adaptateurs sous `harness/rules/`, ainsi que les services
associés comme `harness/firecrawl/`. Le `Makefile` les adapte aux contraintes de chaque agent :
Claude reçoit des liens symboliques, tandis que Codex reçoit un
`~/.codex/AGENTS.md` assemblé.

La procédure `agent-instructions` est une rule globale Claude et une skill conditionnelle Codex,
issues de la même source canonique sous `harness/skills/`.
Les [User Rules de Cursor](https://docs.cursor.com/context/rules) vivent dans
Settings et n'ont pas de chemin fichier utilisateur pris en charge ; le
`Makefile` ne peut donc pas y distribuer cette source.

Les skills user vivent dans `harness/skills/` et sont déployées individuellement
vers les répertoires utilisateur de Claude, Cursor et Codex. Les skills propres
au dépôt vivent dans `.agents/skills/` ; `.claude/skills`, `.codex/skills` et
`.cursor/skills` restent leurs adaptateurs de découverte projet. Un slug ne doit
pas exister dans les deux collections, car les agents peuvent alors exposer les
deux occurrences.

## Outillage maintenu

`tooling/` regroupe deux formes d'outils :

- les exécutables directs, sans extension et nommés en kebab-case, comme
  `upgrade`, `agent-handoff` et `git-main-branch` ;
- les applications structurées dans leur propre répertoire, comme le projet
  Rust `daily-routine/`.

Les exécutables destinés au `PATH` sont liés depuis le `Makefile`, généralement
sous `~/.local/bin`. `tooling/upgrade` met à jour le dépôt puis relance
`make all`, ce qui déploie les nouveaux chemins.

## Flux de changement

1. Placer la source dans la zone qui porte sa responsabilité.
2. Mettre à jour le `Makefile` lorsqu'un artefact est installé ou déployé.
3. Mettre à jour tous les consommateurs du chemin source dans le même changement.
4. Ajouter ou adapter la barrière CI qui couvre le type de fichier concerné.
5. Enregistrer une ADR seulement si le changement introduit ou remplace une
   décision structurante.

Pour choisir un emplacement :

- un fichier reproduisant une destination sous `$HOME` va dans `home/` ;
- une instruction ou capacité commune aux agents va dans `harness/` ;
- un exécutable ou une application maintenue va dans `tooling/` ;
- un chemin imposé par un outil ou un point d'entrée du dépôt reste à la racine.
