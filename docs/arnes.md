# Arnes

## Usage

Arnes vérifie la configuration de Claude Code, Codex et Cursor sans la réparer.
Le manifeste déployé provient de
[`home/.arnes.yaml`](../home/.arnes.yaml).
Moon acquiert le paquet publié `@sebastienelet/arnes` à la version exacte
déclarée dans `package.json` et verrouillée dans `bun.lock`.

## Procédure

Depuis le checkout canonique, déployer le lanceur publié et le manifeste :

```sh
moon run harness:arnes
arnes --version
arnes doctor
arnes doctor skills --agent codex --scope user --format json
arnes doctor instructions --agent claude --scope project
```

Le lien `$HOME/.local/bin/arnes` cible `node_modules/.bin/arnes`.
Le déploiement conserve le lien attendu et refuse une destination divergente.
Sur un poste installé avant ce changement, supprimer le lien `~/.local/bin/arnes`, puis relancer l'installation.

Les hooks s’installent manuellement. Vérifier ensuite leur configuration avec
`moon run harness:doctor`, hors du profil d’installation.
Ce diagnostic ne les installe pas et ne les exécute pas.
Les évaluations appartiennent à
[`@sebastienelet/agent-experiments`](arnes-eval.md).

## Limites et vérification

Le manifeste est `$HOME/.arnes.yaml`. `ARNES_MANIFEST` permet de sélectionner
un autre chemin absolu. Le répertoire courant définit les sources par défaut ;
`ARNES_REPOSITORY_ROOT` et `ARNES_SOURCE_ROOT` permettent de les préciser.
Le lien du manifeste ne suffit pas à déterminer le checkout source.

Doctor observe les fichiers sans démarrer les agents, hooks ou serveurs MCP.
Une projection conforme ne prouve pas son chargement en session.
Le code `0` signale un diagnostic sain, `1` une divergence et `2` une entrée
invalide ou illisible. Examiner également les ressources non prises en charge :
un résultat sain ne prouve pas une couverture complète.

`moon run tooling:deployment-test` exerce le déploiement du lanceur publié,
le rejeu et les collisions dans des HOME temporaires sur macOS et Ubuntu.
Les tests internes de la CLI appartiennent au paquet Arnes.
Depuis un worktree, appliquer les
[précautions de validation](../README.md#contributing-from-a-worktree).
