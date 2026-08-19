# Audit Arnes des skills et plugins externes

`arnes doctor skills` inventorie des capacités durablement disponibles ou exposées. Il ne déduit
ni leur activation pendant une session, ni les tokens réellement consommés. La divulgation
progressive de Codex charge d'abord le nom, la description et le chemin, puis le `SKILL.md` complet
seulement lorsque le skill est sélectionné. Cette liste initiale est bornée à 2 % de la fenêtre de
contexte ou 8 000 caractères lorsque la fenêtre est inconnue.

Le manifest `.arnes.yaml` sépare trois déclarations :

- `external.roots` borne chaque racine de skills système effectivement auditée ;
- `external.plugins` autorise l'identifiant stable d'un plugin sans autoriser ses futurs skills ;
- `external.skills` autorise un slug standalone, système ou fourni par un plugin déjà autorisé.

L'origine `managed` désigne un skill standalone géré hors d'Arnes mais déjà exposé dans une racine
de projection. Cette autorisation ne l'adopte pas et ne déclenche aucun contrôle qualitatif.

Une autorisation permet une capacité, elle ne la rend pas obligatoire. Son absence ne crée donc
aucun drift. La propriété reste `external`, même pour une capacité autorisée. Les diagnostics
conservent le schéma partagé `resource/state/message` et exposent dans le message l'origine, le
conteneur, la version, l'exposition, la topologie, la politique et la limite d'observation runtime.
La sortie humaine affiche le nombre de diagnostics `healthy`, masque leur détail et inventorie les
problèmes ainsi que les limites `unsupported`. Pour les skills, elle regroupe cet inventaire par
agent et place les agents en défaut en premier. `-v` ou `--verbose` rétablit le détail des
diagnostics `healthy` pour tous les doctors. `--format json` reste exhaustif et conserve l'ordre
canonique ; il ne se combine pas avec `--verbose`. `--color auto|always|never` règle la couleur de
toutes les sorties humaines, avec `auto` par défaut : stdout doit être un TTY et `NO_COLOR` doit être
absent ou vide. `always` garde priorité sur `NO_COLOR`; `never` et le JSON ne produisent aucun ANSI.
`--format json --color always` échoue avant tout diagnostic.

## Codex

La documentation officielle définit les racines repository `.agents/skills` du répertoire courant
jusqu'à la racine Git, `$HOME/.agents/skills`, `/etc/codex/skills` et des skills `SYSTEM` embarqués,
sans chemin physique public pour ces derniers. Elle définit aussi `[[skills.config]]` dans
`~/.codex/config.toml` pour désactiver un skill par chemin.

Le chemin `~/.codex/skills/.system` utilisé dans le manifest de ce dépôt est un contrat
d'implémentation local explicite, pas une convention Codex universelle. Arnes peut donc auditer
`openai-docs` sous cette racine sans transformer une observation de HOME en table Rust. Une autre
installation doit déclarer sa propre racine stable ou accepter un diagnostic `unsupported`.

Codex CLI 0.147.0 expose `codex plugin marketplace list --json` et `codex plugin list --json`. La
première commande fournit les marketplaces considérées ; la seconde fournit la sélection installée,
son état on/off et son identifiant d'artefact. Son champ `source.path` désigne la source marketplace,
pas l'artefact actif, et n'est jamais inspecté par Arnes. Il s'agit du schéma structuré du binaire
installé et de son code source tagué, pas d'une interface publique stable. Un schéma absent, invalide
ou incompatible reste donc `unsupported`.

Arnes joint exhaustivement cette sélection à `~/.codex/config.toml` par l'identifiant complet du
plugin et exige une sélection, une identité et une marketplace uniques. Il reconstruit ensuite le
chemin avec le contrat `PluginStore::plugin_root` de Codex 0.147.0, puis le confine sous
`~/.codex/plugins/cache` avant inspection. L'identifiant d'artefact Codex reste distinct de la
version sémantique de `.codex-plugin/plugin.json` : une révision marketplace `11c74d6b` peut ainsi
contenir un manifeste `5.1.3`. Les skills proviennent uniquement de cet artefact résolu. Le contenu
du cache ne choisit jamais la version, même lorsqu'un plugin ne contient qu'un seul répertoire.

Sources : [Build skills](https://learn.chatgpt.com/docs/build-skills),
[Plugins](https://learn.chatgpt.com/docs/plugins),
[Package your plugin](https://developers.openai.com/plugins/build/plugins),
[`plugin_cmd.rs` de Codex CLI 0.147.0](https://github.com/openai/codex/blob/rust-v0.147.0/codex-rs/cli/src/plugin_cmd.rs),
[`store.rs` de Codex CLI 0.147.0](https://github.com/openai/codex/blob/rust-v0.147.0/codex-rs/core-plugins/src/store.rs).

## Claude Code

L'état durable combine les installations et `enabledPlugins`. Les réglages user, project et local
sont lus depuis leurs fichiers documentés ; le réglage le plus proche du projet prévaut dans le
périmètre observable par un doctor lancé dans ce projet. Les réglages managed, serveur ou MDM ne
sont pas déduits depuis HOME.

L'interface officiellement supportée pour l'inventaire est `claude plugin list --json`. Le schéma
statique de `~/.claude/plugins/installed_plugins.json` n'est pas publié. Arnes reconnaît seulement la
version 2 observée par l'installation couverte par cette slice ; toute autre version devient
`unsupported`. Le registre sélectionne l'unique `installPath` effectif. Les anciennes versions du
cache et les market places connues ne sont jamais prises pour des plugins actifs.

Un plugin de skills-directory est reconnu par `.claude-plugin/plugin.json` directement sous une
racine `.claude/skills`; il porte l'identité `<name>@skills-dir`. Il n'est pas aussi rapporté comme
skill standalone unmanaged. Les skills externes ne passent jamais par les contrôles qualitatifs ou
de ressources locales réservés aux skills possédés par Arnes.

Sources : [Settings](https://code.claude.com/docs/en/settings),
[Plugins reference](https://code.claude.com/docs/en/plugins-reference),
[Discover plugins](https://code.claude.com/docs/en/discover-plugins).

## Cursor

Cursor documente les skills builtin et plusieurs racines de skills locales, mais aucun chemin
filesystem stable pour les builtins. Il documente le chargement de plugins de développement sous
`~/.cursor/plugins/local/<plugin>` ; Arnes audite uniquement cette racine explicite.

Customize reste l'interface officielle des plugins marketplace installés. Aucun registre
filesystem stable ne publie leur activation, et `workspaceOpen` peut produire des chemins
dynamiquement. Les plugins marketplace, extensions, builtins et capacités dynamiques sont donc
`unsupported` plutôt que déduits depuis un cache ou exécutés par le doctor.

Sources : [Skills](https://cursor.com/docs/skills),
[Plugins](https://cursor.com/docs/plugins),
[Plugin reference](https://cursor.com/docs/reference/plugins).

## Frontières de lecture

Chaque scan part d'une racine déclarée, d'un registre installé ou d'une configuration effectivement
lue par l'agent. Les symlinks absolus et relatifs sont acceptés seulement si leur cible canonique
reste dans cette racine. Un lien pendant ou une sortie par un composant intermédiaire est rapporté
sans être traversé. `doctor` n'analyse aucun cache orphelin et ne charge aucune capacité. La seule
frontière exécutée est le résolveur JSON Codex pour ses plugins ; elle reçoit le HOME injecté, est
lancée depuis HOME pour exclure les réglages du projet appelant, et possède une limite de cinq
secondes et de 1 Mio par flux. Codex 0.147.0 crée et supprime pendant son démarrage des alias
temporaires sous `CODEX_HOME/tmp/arg0` ; Arnes n'effectue aucune installation ni écriture durable
dans le repository ou HOME.
