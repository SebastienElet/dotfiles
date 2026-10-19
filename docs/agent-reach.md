# Agent-Reach via Moon

L'installation est optionnelle et cible macOS ARM64. Le plugin conserve les références de tous les
canaux d'Agent-Reach. Moon utilise `@sebastienelet/agent-reach-adapter`, épinglé dans `package.json`
et `bun.lock` ; les gestionnaires natifs de Claude Code et Codex installent le plugin. Harness copie
ses ressources et son manifeste `.cursor-plugin/plugin.json` sous `~/.cursor/plugins/local/agent-reach`.
Arnes exige ensuite sa présence pour les trois agents et son activation durable observable pour
Claude et Codex.

Limite actuelle : Cursor ne publie aucune source locale supportée certifiant l'autorisation
d'import et la priorité effective du plugin. Pour Cursor, le contrôle obligatoire valide la
présence et l'intégrité structurelle de la projection locale, avec `activation=unobserved`.
L'activation effective reste à vérifier dans Cursor ; cette limite ne bloque pas le contrôle.

Depuis le checkout canonique, après intégration des changements :

```sh
moon run harness:agent-reach
```

Ne pas lancer cette installation globale depuis un worktree ou un HOME de fixture. Le packaging
et les tests locaux ne nécessitent pas d'installation globale.

## Sources et rejeu

Le fichier `resources/source.json` du paquet npm fixe le commit Agent-Reach et les coordonnées
directes des CLI Python et Node. `uv tool install` isole chaque outil Python sous
`~/.local/share/agent-reach/python-tools/` ; Volta
installe les outils Node. Les dépendances transitives Python et npm ne sont pas verrouillées.
L'image LinkedIn dans `home/.config/agent-reach/compose.yaml` est épinglée par digest multiarchitecture.

Le cache Git et le plugin assemblé résident sous `~/.local/share/agent-reach/`. La marketplace
locale `agent-reach-adapter` livre le même contenu à Claude et Codex depuis
`adapter/<version-du-paquet>/marketplace`. Cursor reçoit une copie locale des mêmes ressources.
Le packaging et la projection Cursor conservent un
résultat identique et refusent un fichier, lien ou contenu divergent. Un changement du pin ou du
packaging exige une reconstruction explicite de la marketplace générée ; l'installation ne
supprime pas automatiquement l'ancienne pour contourner ce refus. Les caches des agents restent
gérés par leurs commandes natives.

Le packaging adapte la description pour conserver la priorité des outils web/GitHub existants,
remplace les installations automatiques par Moon et conserve la licence MIT amont. Les références
amont détaillées restent majoritairement en chinois.

## Configuration des canaux

L'installation ne se connecte à aucun compte et ne démarre pas le serveur LinkedIn. Un Docker
indisponible fait échouer l'installation du backend ; il n'est pas assimilé à une image installée.

- **OpenCLI** : installer manuellement son [extension Chrome](https://chromewebstore.google.com/detail/opencli/ildkmabpimmkaediidaifkhjpohdnifk), puis utiliser une session déjà connectée et explicitement autorisée.
- **Twitter/X et Xueqiu** : fournir explicitement les cookies à la commande de configuration, dans
  son prompt masqué ; ne pas les transmettre dans une conversation.
- **YouTube et Bilibili** : les commandes du plugin activent Node pour `yt-dlp` ; `bili` traite Bilibili.
- **Transcription** : ffmpeg est installé ; la clé Groq ou OpenAI doit être configurée séparément.
- **Boss Zhipin** : le CLI utilise le commit recommandé par Agent-Reach ; Chrome, le profil CDP
  dédié et le login restent à configurer explicitement selon la référence empaquetée.

MCPorter utilise `~/.config/agent-reach/mcporter.json`, sans import des configurations d'autres
agents. Cette configuration contient Exa et le service LinkedIn local ; les recettes du plugin
la sélectionnent explicitement. Le fichier MCPorter est copié comme fichier régulier, car la
lecture de configuration d'Agent-Reach refuse les symlinks. Compose utilise un symlink ordinaire.
Les recettes RSS et le script podcast utilisent le Python de l'environnement Agent-Reach ; le
script podcast est également déployé au chemin attendu par le diagnostic amont.

## LinkedIn

Le navigateur reste dans l'image Docker. Le conteneur `agent-reach-linkedin` et le volume
`agent-reach-linkedin-profile` sont nommés ; les ports HTTP et de login ne sont publiés que sur
la boucle locale. La fermeture du navigateur inactif après 300 secondes n'arrête pas le conteneur.

```sh
moon run harness:agent-reach-linkedin-login
moon run harness:agent-reach-linkedin-start
```

La première commande arrête le serveur éventuel, ouvre un conteneur de login nommé temporaire
et affiche une URL de viewer à ouvrir dans votre navigateur. Vous effectuez le login ; le profil
reste dans le volume. La seconde démarre le serveur HTTP. Pour l'arrêter :

```sh
docker compose --file ~/.config/agent-reach/compose.yaml stop linkedin
```

## Vérification

`moon run harness:agent-reach-check` lance uniquement les audits Arnes de Claude, Codex et Cursor,
sans dépendance d'installation ni cache. Il exige un binaire Arnes à jour et compare d'abord le
manifeste utilisateur au fichier canonique `home/.arnes.yaml` : une politique absente ou ancienne
fait échouer le contrôle. Un plugin `required: true` manque ou est désactivé : le contrôle échoue ;
une activation indéterminable Claude/Codex échoue également. Pour Cursor, la projection locale ne prouve pas
l'autorisation d'import ni l'absence de priorité d'un plugin marketplace homonyme. Arnes rapporte
donc `activation=unobserved` sans rendre ce point bloquant lorsque la projection est présente et
structurellement valide. Cette limite est
distincte du chargement dans une session, qui n'est observé pour aucun agent.

`harness:agent-reach-install-check`, appelé par le parcours d'installation complet, attend les
installations puis lance ces audits, la commande de version Agent-Reach et la lecture native de
configuration MCPorter. Il ne se connecte à aucune plateforme et ne prétend pas valider une
collecte. Les audits Arnes portent sur l'agent entier et peuvent
signaler un défaut externe sans rapport avec Agent-Reach.

Le Doctor global n'est pas lancé automatiquement : le backend Boss du commit épinglé peut
consulter les cookies de Chrome via CDP. Son invocation exige donc une autorisation explicite
d'inspection des sessions configurées, même lorsqu'on cherche à diagnostiquer un autre canal.

Les tests Bun du packaging et du cache Git exercent le rejeu, les contenus divergents, les
ressources manquantes et les liens symboliques. La syntaxe Moon, les manifestes plugin et la
configuration Compose utilisent leurs validations natives. L'installation globale, le login et
les collectes authentifiées doivent être exercés sur un poste configuré.
