# Prévention des secrets

Gitleaks CLI complète le secret scanning et la push protection GitHub. Le hook local
est consultatif : `git commit --no-verify`, `core.hooksPath`, un autre clone ou une
modification du hook peuvent le contourner. Le statut Actions **Secret scan** doit
être obligatoire sur `main`. Un administrateur capable de modifier les règles du
dépôt peut toujours les désactiver. Aucun de ces détecteurs ne prouve l'absence de
tout secret : mots de passe arbitraires, formats inconnus et certains contenus
encodés restent des limites.

## Préparation et commandes

Depuis un clone neuf macOS, amorcer Moon selon le README puis exécuter :

```bash
moon run repository:secret-prepare
```

Le profil minimal `repository:install` inclut cette préparation. Homebrew installe
Gitleaks sur macOS ; Ubuntu x64 utilise la release officielle et son SHA-256 dans
`tooling/gitleaks-release.json`. Le scanner qualifié initialement est **8.30.1**,
MIT. Homebrew suit sa formule : relever et qualifier toute nouvelle version avant
d'affirmer son équivalence. Pour Ubuntu, mettre à jour ensemble version et empreinte
officielle, puis exécuter les tests macOS et Ubuntu. L'amont annonce seulement des
correctifs de sécurité : réexaminer le choix si les détecteurs deviennent insuffisants.

La préparation installe uniquement le hook propre au dépôt, sans changer la
configuration Git. Elle refuse un hook tiers, un symlink ou un `core.hooksPath`
configuré et les conserve. Dans ce cas, intégrer manuellement cette commande dans
le hook existant, en propageant son statut :

```bash
bun --config=/dev/null --no-env-file --no-install tooling/secret-scan.ts staged
```

Les worktrees partagent le hook Git ; celui-ci trouve le checkout actif avec Git,
sans enregistrer le chemin de l'installateur. Exécuter la préparation dans chaque
clone indépendant et préparer Bun et les dépendances dans chaque worktree. Un
checkout ancien sans ces fichiers refuse le commit tant que le hook partagé reste
actif. La préparation ne supprime aucun hook ancien d'origine inconnue.

| Commande Moon               | Périmètre                                                                     |
| --------------------------- | ----------------------------------------------------------------------------- |
| `repository:secret-staged`  | Chemins indexés et diff indexé, sans installation                             |
| `repository:secret-current` | Chemins et blobs suivis à `HEAD`, inclus dans `repository:check`              |
| `repository:secret-ci`      | Arbre testé et tous les commits introduits selon l'événement GitHub           |
| `repository:secret-history` | Tous les commits accessibles depuis les refs locales                          |
| `repository:secret-test`    | Vrai scanner et dépôts temporaires contenant uniquement des fixtures fictives |

Les scans ne sont pas cachés. Le workflow s'exécute sur toute PR, push vers `main`,
`merge_group` et déclenchement manuel, sans filtre de chemins ni sélection affectée.
Il contrôle les blobs Git, même masqués par `export-ignore` ou des changements du
working tree. La plage utilise les SHA de l'événement : une ref absente, un checkout
shallow, un événement invalide ou une erreur Git refusent le contrôle. Un push de
création sans base et un lancement manuel déclenchent le scan complet des refs.
Les diffs de merges sont inclus pour détecter les résolutions introduites puis
supprimées. Les chemins privés introduits puis supprimés sont également refusés.

Le hook ne télécharge rien et appelle `gitleaks git --staged --redact=100`. Après
ce diff natif, il scanne aussi les blobs indexés des chemins modifiés : les attributs
Git du working tree, `textconv` et la détection binaire peuvent masquer un secret dans
le mode Git natif. Les scans historiques forcent les diffs texte sans `textconv`,
diff externe ni détection de renommage. Ces contournements ont des tests de refus.
Après préparation, il fonctionne hors ligne. Un scanner absent, une configuration invalide,
une erreur de scan ou un rapport absent/invalide produisent un échec. Les messages
n'exposent que fichier, commit et type ; les rapports temporaires sont supprimés.
La configuration provient de l'index pour le hook et de `HEAD` pour les autres
scans ; une modification seulement dans le working tree ne change pas les règles.
Les variables `GITLEAKS_*`, `.gitleaksignore` et `gitleaks:allow` ne désactivent pas
le contrôle.

## Refus, templates et exceptions

Les `.env` et `.env.*` sont ignorés et refusés par leur chemin, même vides ou ajoutés
avec `git add -f`, quelle que soit la casse du nom privé. Seuls les fichiers exacts `.env.example`, `.env.sample` et
`.env.template` à la racine sont autorisés ; leur contenu reste scanné. `version.env`
et les clés publiques sont autorisés. Les noms de clés privées SSH usuels, `.p12`,
`.pfx`, `home/.aws/credentials` et les credentials applicatifs gcloud sont privés.
Une clé PEM reste soumise au détecteur de contenu, sans interdire toutes les clés
publiques PEM par extension.

Pour un refus local, retirer le secret de l'index et du fichier, remplacer sa valeur
par une référence à l'environnement ou à 1Password, puis relancer le scan indexé.
Si la valeur a été publiée, la révoquer/rotater auprès du fournisseur ; la suppression
du fichier ne révoque rien. Ne pas copier sa valeur dans une issue, une PR ou un log.

Les exceptions de `.gitleaks.toml` sont limitées à une règle, un chemin précis et
un contenu inoffensif documenté. Elles conservent les règles par défaut. Aucun dossier
de documentation, fixture ou template n'est exclu globalement ; aucune baseline
n'est utilisée. Toute nouvelle exception demande une revue et un test montrant
qu'un nouveau token voisin reste refusé. Ne pas accepter un secret actif comme
exception, ni régénérer automatiquement une baseline.

## Historique et limites de preuve

Avant un rescan historique, actualiser le clone complet, inventorier les branches
distantes et tags avec `git ls-remote --heads --tags origin`, puis relever les refs
locales avec `git for-each-ref`. Les refs GitHub non récupérées, objets inaccessibles,
forks et reflogs ne sont pas inclus implicitement par `--all`. Consigner SHA, refs,
version, environnement et durée ; examiner chaque finding sans exposer sa valeur.
Un vrai secret nécessite une décision du propriétaire et un traitement traçable.
**Ne jamais réécrire automatiquement l'historique.**

Le scan initial du 11 octobre 2026, sur macOS Darwin arm64, a couvert **2 576 commits**
depuis le 14 août 2014, après fetch des huit branches distantes publiées par `origin`
et des tags (aucun tag publié). `origin/main` était
`01326d15b64b3f8d7cc2a56e7c9102e61472e26f`. Les autres refs locales accessibles
étaient également incluses. Le premier scan utilisait les règles natives sans
exclusion historique, sans ignore inline et avec redaction intégrale.

Le premier passage natif trouvait neuf occurrences. Le passage renforcé, incluant
les merges et les renommages sans conversion, en trouve **21 en 2,145 s** : toutes
proviennent des mêmes faux positifs, réintroduits dans différents commits. Il s'agit
d'empreintes SHA-256 dans trois manifests d'évaluation, d'un identifiant AWS
alphabétique fictif dans deux tests et d'en-têtes PEM sans clé dans des tests Rust. Les
exceptions ciblées sont décrites dans la configuration. Aucun secret réel n'a été
identifié dans ce périmètre ; ce résultat ne garantit pas l'absence de secrets.
Le rescan avec ces seules exceptions retourne zéro finding en **1,981 s**.
Les [216 refs inventoriées](evidence/issue-476-secret-scan-refs.tsv) et les
[21 occurrences triées](evidence/issue-476-secret-scan-findings.tsv) sont conservées
sans valeur ni contexte de secret, avec uniquement commit, fichier et type pour les findings.

Les tests exécutent le vrai scanner sans contacter les fournisseurs, avec des tokens
construits comme fixtures et une clé privée générée à l'exécution. Ils établissent
les refus via `git commit` et la même commande CI, y compris l'ajout puis la suppression
d'un token dans deux commits. Un job vert sur ces témoins prouve que la commande de
scan retourne un échec ; il ne remplace pas une tentative de fusion serveur refusée.
La PR de livraison relève les runs Actions, versions, durées et contrôles GitHub,
ainsi que les preuves serveur ou plateformes non exercées.
