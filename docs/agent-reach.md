# Agent-Reach

## Usage

Le parcours optionnel cible macOS ARM64 et installe le plugin pour Claude, Codex
et Cursor depuis le paquet épinglé `@sebastienelet/agent-reach-adapter`.
L'installation ne connecte aucun compte. Les références des canaux sont livrées
avec le plugin ; aucune installation globale ne doit partir d'un worktree.

## Procédure

Depuis le checkout canonique, après intégration :

```sh
moon run harness:agent-reach
moon run harness:agent-reach-check
```

Le contrôle compare d'abord le manifeste déployé à `home/.arnes.yaml`, puis lance
les diagnostics Arnes des trois agents. Un ancien plugin
`agent-reach@dotfiles-agent-reach` n'est pas remplacé automatiquement : conserver
sa marketplace pour le retour arrière, puis retirer explicitement cette ancienne
installation avec les commandes natives de l'agent. Ne pas effacer un cache partagé
pour contourner un refus de projection divergente.

Les connexions restent explicites : extension Chrome OpenCLI et session autorisée,
cookies X/Xueqiu saisis dans le prompt masqué du CLI, clés de transcription
configurées séparément. Ne pas transmettre ces secrets dans la conversation.
Pour LinkedIn :

```sh
moon run harness:agent-reach-linkedin-login
moon run harness:agent-reach-linkedin-start
docker compose --file ~/.config/agent-reach/compose.yaml stop linkedin
```

Le login affiche une URL de viewer ; le profil reste dans le volume Docker nommé
`agent-reach-linkedin-profile`. Le serveur utilise les ports de boucle locale.
MCPorter lit `~/.config/agent-reach/mcporter.json` ; ne pas importer les
configurations privées d'autres agents.

## Limites et vérification

Le paquet fixe le commit amont et les coordonnées directes des outils Python/Node ;
leurs dépendances transitives ne sont pas verrouillées. Une reconstruction de
marketplace après changement de pin reste explicite.

`harness:agent-reach-check` observe la présence et l'activation durable de
Claude/Codex. Pour Cursor, la projection structurelle ne prouve ni l'autorisation
**Allow Local Plugin Imports**, ni sa priorité face à un plugin marketplace homonyme :
vérifier l'activation dans Cursor. Aucun de ces diagnostics ne prouve une collecte
ou un chargement en session.

Le Doctor global Agent-Reach peut consulter les cookies Chrome via CDP ; son
invocation exige une autorisation explicite, même pour diagnostiquer un autre canal.
Les tests Bun de packaging utilisent des fixtures ; login et collectes demandent
un poste configuré. Les choix structurants suivent l'[ADR-031](adr/031-lecture-web-agent-reach.md).
