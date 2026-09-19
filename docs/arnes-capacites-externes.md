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

Un diagnostic `unsupported` rapporte une observation qui a échoué : version de registre inconnue,
résolveur illisible, exposition indéterminable. Une capacité dont aucun inventaire n'est observable
n'en produit aucun : elle est documentée ici et reste silencieuse à l'exécution. Une racine non
déclarée est donc absente du rapport plutôt que signalée, le manifeste restant la source unique de
ce qui est audité.

Une autorisation permet une capacité, elle ne la rend pas obligatoire. Son absence ne crée donc
aucun drift. La propriété reste `external`, même pour une capacité autorisée. Les diagnostics
conservent le schéma partagé `resource/state/message` et exposent dans le message l'origine, le
conteneur, la version, l'exposition, la topologie, la politique et la limite d'observation runtime.
Les options, formats, états et codes de sortie communs sont décrits dans la
[référence Doctor](arnes-doctor.md#formats-états-et-sorties). Pour les skills, la sortie humaine
regroupe l'inventaire par agent et place les agents en défaut en premier.

## Codex

La documentation officielle définit les racines repository `.agents/skills` du répertoire courant
jusqu'à la racine Git, `$HOME/.agents/skills`, `/etc/codex/skills` et des skills `SYSTEM` embarqués,
sans chemin physique public pour ces derniers. Elle définit aussi `[[skills.config]]` dans
`~/.codex/config.toml` pour désactiver un skill par chemin.

Le chemin `~/.codex/skills/.system` utilisé dans le manifest de ce dépôt est un contrat
d'implémentation local explicite, pas une convention Codex universelle. Arnes peut donc auditer
`openai-docs` sous cette racine sans transformer une observation de HOME en table Rust. Une autre
installation déclare sa propre racine stable ; sans déclaration, aucun skill système n'est audité.
L'activation des plugins de projet n'a aucun registre filesystem : le scope project ne rapporte donc
aucun plugin Codex.

Doctor ne lance aucune commande Codex pour inventorier les plugins. Les commandes de résolution
introduites par #166/#168 ont été retirées selon D1 de #111 : leurs effets externes ne pouvaient
pas être déduits des snapshots. L'inventaire actif reste explicitement `unsupported`, même lorsque
la configuration ne mentionne aucun plugin ; ce silence local ne prouve pas une installation vide.

Arnes lit les entrées de plugins dans `~/.codex/config.toml` et distingue `enabled=true`,
`enabled=false` et l'absence du champ. Une entrée activée hors politique reste `drift` ; une entrée
autorisée, désactivée ou d'exposition inconnue reste `unsupported` faute de topologie observable.
Pour une entrée activée, `exposure=enabled` décrit le réglage, tandis que `activation=unknown`
signale l'absence d'observation de disponibilité. Une configuration malformée reste une erreur.

Installation active, version, artefact, chemin, topologie et skills d'un plugin Codex ne sont plus
résolus. Aucun cache n'est inspecté pour choisir un artefact, même solitaire. Les racines déclarées
de skills standalone/système et leurs réglages `[[skills.config]]` restent audités séparément.

Sources : [Build skills](https://learn.chatgpt.com/docs/build-skills),
[Plugins](https://learn.chatgpt.com/docs/plugins),
[Package your plugin](https://developers.openai.com/plugins/build/plugins).

## Claude Code

L'état durable combine les installations et `enabledPlugins`. Les réglages user, project et local
sont lus depuis leurs fichiers documentés ; le réglage le plus proche du projet prévaut dans le
périmètre observable par un doctor lancé dans ce projet. Les réglages managed, serveur ou MDM ne
sont pas déduits depuis HOME.

Les _bundled skills_ sont embarqués dans le binaire natif, sans répertoire ni sous-commande
d'énumération ; `disableBundledSkills` et `skillOverrides` documentent leur exposition, pas leur
inventaire. Les reconstituer exigerait une table de noms en Rust liée à une version du binaire :
Arnes n'audite donc aucun skill système Claude.

L'interface officiellement supportée pour l'inventaire des plugins est `claude plugin list --json`.
Le schéma statique de `~/.claude/plugins/installed_plugins.json` n'est pas publié. Arnes reconnaît seulement la
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
filesystem stable ne publie leur activation, `cursor-agent plugin marketplace list` n'énumère que
les marketplaces, et `workspaceOpen` peut produire des chemins dynamiquement.
`~/.cursor/extensions/extensions.json` liste les extensions installées mais pas leur activation,
qui réside dans le stockage global non documenté de l'éditeur ; aucune extension n'expose de skill.
Le répertoire `~/.cursor/skills-cursor` contient bien les builtins synchronisés, mais il est non
documenté, décrit par deux manifestes internes divergents, et Cursor ne documente aucun moyen de
désactiver un builtin : un inventaire y serait donc un drift permanent sans remède. Les plugins
marketplace, extensions, builtins et capacités dynamiques ne sont donc ni audités, ni déduits depuis
un cache, ni obtenus en exécutant l'agent.

Sources : [Skills](https://cursor.com/docs/skills),
[Plugins](https://cursor.com/docs/plugins),
[Plugin reference](https://cursor.com/docs/reference/plugins).

## Frontières de lecture

Chaque scan part d'une racine déclarée, d'un registre installé ou d'une configuration effectivement
lue par l'agent. Les symlinks absolus et relatifs sont acceptés seulement si leur cible canonique
reste dans cette racine. Un lien pendant ou une sortie par un composant intermédiaire est rapporté
sans être traversé. `doctor` n'analyse aucun cache orphelin et ne charge aucune capacité.
Le chemin Doctor possédé ne lance aucun résolveur externe, hook, serveur MCP ou statusline.
La non-exécution de Codex est exercée par un double dont le lancement laisserait un témoin
hors des snapshots ; les états finaux du dépôt et du HOME sont vérifiés séparément. La
[référence Doctor](arnes-doctor.md#vérification-et-portée-des-preuves) borne les preuves actuelles
d'absence de modification aux fixtures effectivement exercées, sans les assimiler à une session réelle.
