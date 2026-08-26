# ADR-002 — Homebrew et `mas` comme unique source de paquets

- **Statut** : accepté
- **Date** : 2015-02
- **Commits** : `5c64955`, `ab9771e`

## Contexte

Les outils proviennent de canaux hétérogènes : formules Homebrew, casks,
App Store, installateurs éditeur. Multiplier les canaux rend la mise à jour
impossible à automatiser et l'installation impossible à rejouer en CI.

## Décision

Par défaut, tout paquet passe par Homebrew (formule ou cask), et par `mas` pour
ce que seul l'App Store distribue. Les sources non-Homebrew du graphe pris en
charge sont exhaustivement déclarées ci-dessous. Les services sont pilotés par
`brew services`. Les applications payantes de l'App Store sont exclues par
défaut via `SKIP_PAID_APPS`, ce qui permet à la CI d'exécuter l'installation
complète. `HOMEBREW_NO_ASK` supprime les invites interactives, et les taps sont
approuvés (`brew trust`) avant toute mise à jour.

Les trois paquets réconciliés suivent la version stable publiée par Homebrew ;
leurs définitions actuelles fixent un checksum qui protège l'intégrité du
téléchargement. `tooling/upgrade` met à jour les formules et les casks, y
compris ceux qui se déclarent auto-mis à jour, et les sentinelles Homebrew
rendent l'installation rejouable. `--adopt` n'est admis que pour reprendre un
artefact existant identique à celui du cask ; une divergence est refusée au
lieu d'être écrasée.

Les sources non-Homebrew admises ont le contrat suivant :

| Source | Version | Intégrité | Mise à jour | Rejeu |
| --- | --- | --- | --- | --- |
| Amorçage Homebrew | Installateur `HEAD` officiel | Script servi par Homebrew en HTTPS | Mise à jour propre à Homebrew après installation | Exécuté seulement lorsque le binaire Homebrew est absent |
| Paquets Node via Volta ou npm ([ADR-016](016-volta-gestionnaire-node.md), [ADR-017](017-npm-pour-paquets-globaux.md)) | Coordonnée déclarée par la cible, avec canal explicite lorsqu'il existe | Registre npm et vérification de la charge utile par le gestionnaire | `volta install node` et `npm update -g` par `tooling/upgrade` | Shims ou binaires sentinelles ; une cible déjà satisfaite n'installe pas une seconde copie |
| Plugins Fish et tmux | Révision obtenue par Fisher ou Git lors de la première installation | Transport GitHub en HTTPS et objets Git | Mise à jour explicite par leur gestionnaire ; absente de `tooling/upgrade` | Le fichier Fisher ou le checkout TPM sert de sentinelle et conserve la révision locale |
| Claude Code | Canal éditeur `latest` résolu à l'installation | Installateur et charge utile servis par l'éditeur en HTTPS | `claude update` par `tooling/upgrade` | La sentinelle du binaire évite une seconde installation ; la mise à jour converge séparément vers le dernier `latest` disponible |
| Images Docker des MCP | Référence déclarée dans le `Makefile` ou le fichier Compose, tag compris | Manifeste et couches vérifiés par leur digest lors du transfert depuis le registre | `docker pull` ou `docker compose pull` explicite ; `make all` ne télécharge qu'une image absente | Une référence déjà présente est réutilisée ; un tag flottant conserve donc le digest local jusqu'à la mise à jour explicite |
| Thèmes Bat | Branche par défaut Catppuccin lors de la première installation | Fichiers servis par GitHub en HTTPS | Changement explicite de la source ou suppression de la sentinelle | Les fichiers de thème servent de sentinelles et conservent leur contenu local |
| Dictionnaires Hunspell | Commit LibreOffice et SHA-256 déclarés dans le `Makefile` | SHA-256 vérifié avant publication atomique | Changement explicite du commit et du checksum | Une destination identique est conservée ; une divergence est refusée |

`tooling/check-software-sources.ts` matérialise les commandes émises par le
graphe `all` en mode à sec et refuse toute référence distante ou tout
gestionnaire visible absent de l'inventaire déclaré. Il contrôle aussi les
images exactes des fichiers Compose atteints et les commandes `$(shell ...)`
sans les exécuter. Une impossibilité de produire le graphe, une configuration
Compose invalide ou l'absence du canal Homebrew fait échouer le contrôle. Les
sources internes à un outil local appelé par le `Makefile` restent sous le
contrat et les tests propres de cet outil, hors de cette barrière.

## Conséquences

- Une seule commande de mise à jour pour l'ensemble du poste
  ([ADR-004](004-script-upgrade-unique.md)).
- Dépendance forte aux dépréciations Homebrew : plusieurs commits ne font que
  suivre des taps ou des formules retirés en amont.
- Les exceptions ne bénéficient pas du contrat de version et de mise à jour
  Homebrew ; leur contrat plus faible est explicite et contrôlé séparément.

## Alternatives écartées

- Installateurs éditeur ou téléchargements manuels : non rejouables.
- Nix : rupture d'outillage disproportionnée au regard du besoin.
