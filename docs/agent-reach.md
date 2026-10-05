# Agent-Reach via Moon

L'installation est optionnelle et cible macOS. Le plugin conserve les références de tous les
canaux d'Agent-Reach. Moon installe les outils ; les gestionnaires natifs de Claude Code et Codex
installent le plugin ; Arnes audite sa déclaration externe.

Depuis le checkout canonique, après intégration des changements :

```sh
moon run harness:agent-reach
```

Ne pas lancer cette installation globale depuis un worktree ou un HOME de fixture. Le packaging
et les tests locaux ne nécessitent pas d'installation globale.

## Sources et rejeu

`harness/plugins/agent-reach/source.json` fixe le commit Agent-Reach, la version du plugin et les
coordonnées directes des CLI Python et Node. `uv tool install` isole chaque outil Python sous
`~/.local/share/agent-reach/python-tools/` ; Volta
installe les outils Node. Les dépendances transitives Python et npm ne sont pas verrouillées.
L'image LinkedIn dans `home/.config/agent-reach/compose.yaml` est épinglée par digest multiarchitecture.

Le cache Git et le plugin assemblé résident sous `~/.local/share/agent-reach/`. La marketplace
locale `dotfiles-agent-reach` livre le même contenu aux deux agents. Le packaging conserve un
résultat identique et refuse un fichier, lien ou contenu divergent. Un changement du pin ou du
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

`harness:agent-reach-check` suit les installations et lance les audits Arnes, la commande de
version Agent-Reach et la lecture native de configuration MCPorter. Il ne se connecte à aucune
plateforme et ne prétend pas valider une collecte. Les audits Arnes portent sur l'agent entier et peuvent
signaler un défaut externe sans rapport avec Agent-Reach.

Le Doctor global n'est pas lancé automatiquement : le backend Boss du commit épinglé peut
consulter les cookies de Chrome via CDP. Son invocation exige donc une autorisation explicite
d'inspection des sessions configurées, même lorsqu'on cherche à diagnostiquer un autre canal.

Les tests Bun du packaging et du cache Git exercent le rejeu, les contenus divergents, les
ressources manquantes et les liens symboliques. La syntaxe Moon, les manifestes plugin et la
configuration Compose utilisent leurs validations natives. L'installation globale, le login et
les collectes authentifiées doivent être exercés sur un poste configuré.
