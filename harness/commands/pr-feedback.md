---
description: Produit un relevé factuel et traçable des problèmes relevés en revue et des correctifs poussés par les relecteurs sur mes PR
---

Tu analyses les retours reçus sur mes pull requests et les correctifs poussés par les relecteurs.
Tu produis un relevé condensé, structuré et transmissible tel quel à l'agent qui améliore le
harnais. Tu établis les problèmes observés et leurs preuves ; tu ne proposes aucune amélioration.
Lecture seule — tu n'édites rien.

## Portée

`$ARGUMENTS` = liste de numéros de PR, ou un nombre de PR à remonter. Par défaut : les 10
dernières PR fusionnées dont je suis l'auteur.

## Forge

Déduis la forge de `git remote get-url origin` et n'emploie que son client. `<slug>` désigne
partout le chemin du dépôt lu sur ce remote (`<espace>/<dépôt>`) ; ne code jamais sa valeur en
dur dans ce fichier. Avec `gh api`, écris littéralement `{owner}/{repo}` : le client les
substitue depuis le remote, et la dérivation manuelle devient inutile.

```bash
bkt pr list --mine --state MERGED --limit 10 --json --jq '.pull_requests[] | "\(.id) | \(.title)"'
gh pr list --author @me --state merged --limit 10 --json number,title
```

Les deux n'ont pas la même portée : sans dépôt, `bkt pr list --mine` remonte **tous** les
dépôts de l'espace, là où `gh pr list` se limite au dépôt courant. Sur Bitbucket, écarte les PR
d'un autre dépôt avant de compter les occurrences.

Pour toute autre forge, mappe les concepts ci-dessous sur son client (`glab`, `tea`, l'API
brute) en vérifiant les options dans son aide. Les concepts ne changent pas ; les endpoints,
si. N'invente pas une option que tu n'as pas lue.

## Collecte, par PR

Une fusion en **squash** efface les commits correctifs d'un relecteur : dans la branche cible
ils réapparaissent sous mon nom. Le signal vit donc côté forge, dans trois endroits :

**1. La séquence des poussées** — qui a poussé quoi, dans quel ordre :

```bash
bkt api "/repositories/<slug>/pullrequests/<id>/activity?pagelen=50" --json --jq '.values[] | select(.update) | "\(.update.date) | \(.update.author.display_name) | \(.update.source.commit.hash) | \(.update.state)"'
gh pr view <id> --json commits --jq '.commits[] | "\(.committedDate) | \(.authors[0] | if (.login // "") == "" then .name else .login end) | \(.oid[0:12]) | \(.messageHeadline)"'
```

Une poussée par quelqu'un d'autre que moi est un **correctif de relecteur potentiel**. Ne la
retiens qu'après avoir vérifié son delta : une fusion de la branche cible, un rebase ou une mise à
jour automatique n'est pas un correctif. Ne te limite pas aux poussées postérieures à ma dernière
poussée ; un correctif peut précéder une reprise de ma part.

**2. Le delta correctif** — ce que le relecteur a ajouté par-dessus mon travail.
L'activité ne liste pas tous mes pushes, et une branche absorbe des fusions de la branche
d'intégration ainsi que des PR fusionnées en squash **sous mon nom** : remonte la chaîne de
commits avant de calculer le delta.

```bash
bkt api "/repositories/<slug>/commits/<sien>?pagelen=10" --json --jq '.values[] | "\(.hash[0:12]) | \(.author.user.display_name // .author.raw) | \(.message | split("\n")[0])"'
gh api "repos/<slug>/commits?sha=<sien>&per_page=10" --jq '.[] | "\(.sha[0:12]) | \(.author.login // .commit.author.name) | \(.commit.message | split("\n")[0])"'
```

La borne basse est le commit qui précède **le premier commit du relecteur** — la fusion de la
branche d'intégration s'il y en a une —, jamais le SHA lu dans l'activité. Vérifie-la sur le
récapitulatif de fichiers avant de demander le `diff`, et filtre par fichier : un delta qui
touche des fichiers étrangers au sujet de la PR est le signe que la borne est mauvaise.

```bash
bkt api "/repositories/<slug>/diffstat/<sien>..<borne>?pagelen=100" --json --jq '.values[] | "\(.status) +\(.lines_added) -\(.lines_removed) \(.new.path // .old.path)"'
bkt api "/repositories/<slug>/diff/<sien>..<borne>?path=<fichier>"
gh api "repos/<slug>/compare/<borne>...<sien>" --jq '.files[] | "\(.status) +\(.additions) -\(.deletions) \(.filename)"'
gh api "repos/<slug>/compare/<borne>...<sien>" --jq '.files[] | select(.filename == "<fichier>") | .patch'
```

Lis ce diff en entier. Un hunk ne devient un constat que s'il corrige ou prévient un comportement
identifiable ; regroupements de commits, formatage mécanique et synchronisations de branche ne
sont pas des problèmes de conception.

**3. Les commentaires et les tâches** :

```bash
bkt pr comments <id> --json --jq '.comments[] | "\(.user.display_name) | \(.inline.path // "general"):\(.inline.to // "") | \(.content.raw)"'
bkt pr task list <id> --json
gh pr view <id> --json comments,reviews --jq '(.comments[], (.reviews[] | select(.body != ""))) | "\(.author.login) | \(.body)"'
gh api "repos/<slug>/pulls/<id>/comments" --jq '.[] | "\(.user.login) | \(.path):\(.line) | \(.body)"'
```

Trois émetteurs à distinguer, sans les mélanger :

- **humain** — un relecteur ;
- **revue IA** (postée par la CI, souvent sous mon compte) ;
- **moi-même** (notes de rebase, auto-commentaires) ; à écarter.

## Constitution des constats

Transforme chaque commentaire retenu et chaque hunk correctif en un problème formulé en termes de
comportement, jamais en termes de solution. Un constat répond uniquement à ces questions :

1. **Qu'a relevé ou corrigé le relecteur ?** Une phrase factuelle.
2. **Quelle preuve l'établit ?** Lien du commentaire ou SHA, fichier et résumé du changement.
3. **Quelle suite est observable ?** Corrigé par moi, corrigé par le relecteur, écarté explicitement,
   resté sans suite, ou indéterminé.
4. **Quel impact a été observé ?** Seulement s'il est explicite dans la revue ou démontré par le
   correctif ; sinon `non établi`.

Déduplique un commentaire et le commit qui le corrige en un seul constat, tout en conservant les
deux preuves. Regroupe un même problème apparu sur plusieurs PR, sans effacer ses occurrences. Ne
déduis ni la cause du développement initial, ni la règle qui aurait manqué, ni le changement de
harnais à effectuer. Une interprétation non prouvée va dans `Incertitude`, jamais dans le constat.

## Sortie

```
# Retours de review — <dépôt>

## Périmètre
| PR | Titre | Relecteur(s) | Fusion | Sources examinées |
| --- | --- | --- | --- | --- |
| <id + lien> | <titre> | <identités> | <date, auteur de la fusion> | <n commentaires, n tâches, n commits tiers> |
```

```
## Constats consolidés

### C01 — <problème formulé en comportement>
- Occurrences : <PR concernées>
- Preuves :
  - <PR + lien> · <commentaire humain|revue IA> · <auteur> · <fichier:ligne> — <reformulation fidèle>
  - <PR + lien> · correctif du relecteur · <auteur> · <SHA> · <fichier> — <comportement avant → après>
- Suite observée : <statut et auteur du correctif, sans causalité supposée>
- Impact observé : <fait établi|non établi>
- Incertitude : <limite précise|aucune>
```

```
## Inventaire par PR
- PR <id> — <n constats : C01, C03> ; <n signaux écartés>
```

```
## Signaux écartés
- <PR + lien> · <signal> — <raison factuelle : doublon, administratif, auto-commentaire, synchronisation de branche>
```

```
## Limites de collecte
- <source indisponible, pagination incomplète, attribution ou borne de diff incertaine>
```

Une section vide s'écrit `_néant_`. Pas de préambule, pas de conclusion.

## Contraintes d'exécution

- Une commande par appel : le garde-fou de worktree refuse les boucles shell, les
  substitutions `$(...)` et les heredocs.
- Avec `bkt`, `--jq` exige `--json` dans la même commande.
- Chaque affirmation renvoie à une PR, un commentaire ou un SHA ; sans preuve, écris la limite au
  lieu de compléter par vraisemblance.
- N'inspecte pas le harnais pour expliquer les constats et ne formule aucune recommandation,
  priorité, règle ou texte à coller.
- Ne modifie aucun fichier.
