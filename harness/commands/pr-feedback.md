---
description: Analyse les retours de revue et les correctifs poussés par les relecteurs sur mes PR, puis en déduit les correctifs de harnais
---

Tu analyses les retours reçus sur mes pull requests pour en tirer des correctifs de harnais.
Objectif : réduire le volume de retours sur les PR suivantes. Lecture seule — tu proposes, tu
n'édites rien.

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

Toute poussée par quelqu'un d'autre que moi, après ma dernière poussée, est un **correctif de
relecteur**. C'est la source la plus riche : ce sont des retours qui n'ont jamais été écrits.

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

Lis ce diff en entier : chaque hunk est un manque de la conception initiale.

**3. Les commentaires et les tâches** :

```bash
bkt pr comments <id> --json --jq '.comments[] | "\(.user.display_name) | \(.inline.path // "general"):\(.inline.to // "") | \(.content.raw)"'
bkt pr task list <id> --json
gh pr view <id> --json comments,reviews --jq '(.comments[], (.reviews[] | select(.body != ""))) | "\(.author.login) | \(.body)"'
gh api "repos/<slug>/pulls/<id>/comments" --jq '.[] | "\(.user.login) | \(.path):\(.line) | \(.body)"'
```

Trois émetteurs à distinguer, ne les mélange pas :

- **humain** — un relecteur ; poids maximal ;
- **revue IA** (posté par la CI, souvent sous mon compte) ; poids moyen, souvent bruyant ;
- **moi-même** (notes de rebase, auto-commentaires) ; à écarter.

## Analyse, par constat

Pour chaque hunk correctif et chaque commentaire retenu, réponds à **deux** questions. La
seconde est celle qui produit le correctif de harnais ; un constat sans elle n'a aucune valeur.

1. **Qu'est-ce qui manquait ?** Une phrase, en termes de comportement, pas de diff.
2. **Pourquoi était-ce absent du développement initial ?** Une catégorie, et une seule :

   | Catégorie | Signification | Correctif visé |
   | --- | --- | --- |
   | `non-appliquée` | La règle existait, écrite, et n'a pas été suivie | Un contrôle automatique |
   | `non-chargée` | La règle existait, mais dans une couche jamais lue sur ce chemin | Un déplacement de couche |
   | `inconnue` | Aucune règle n'existait | Une règle neuve, à la couche la moins chère |
   | `angle-mort` | Cas limite non pensé : erreur, nul, vide, délai, concurrence, multiplicité | Une étape de skill ou un test canonique |
   | `arbitrage` | Divergence de jugement légitime, pas une erreur | Rien. Le noter et passer |

   Vérifie avant de classer : `grep` la règle supposée dans `AGENTS.md`, `.agents/rules/`,
   `.agents/skills/`, `harness/skills/`, `MEMORY.md`. Une règle que tu supposes existante sans l'avoir trouvée est
   `inconnue`, pas `non-appliquée`.

## Correctifs de harnais — échelle

Un constat récurrent mérite le **barreau le plus bas qui tient**, jamais plus haut :

1. **Un contrôle automatique** — règle de linter, hook pre-commit, étape CI, contrainte de
   schéma, type. Le seul barreau qui ne dépend pas de la mémoire d'un agent. Vise-le d'abord.
2. **Une règle `.agents/rules/*.mdc` à glob étroit** — si le manque ne concerne qu'une famille de
   fichiers.
3. **Une étape dans un skill existant** — si le manque est procédural et que le skill est déjà
   invoqué sur ce chemin. Ne crée pas un skill neuf pour une étape.
4. **Une ligne dans `AGENTS.md`** — seulement si la règle s'applique partout et à plus de 80 % des
   tours. Le budget de lignes est celui que fixe le dépôt : toute ligne ajoutée en chasse une
   autre, dis laquelle.
5. **Une entrée `MEMORY.md`** — pour un piège découvert, non dérivable du code. Descriptif, pas
   prescriptif.

Une seule proposition par constat. Si deux barreaux tiennent, prends le plus bas et n'évoque pas
l'autre.

## Seuil de récurrence

Ne propose un changement durable que si le constat apparaît **au moins deux fois, sur deux PR
distinctes**, ou une seule fois avec conséquence sévère (perte de données, faille, donnée
personnelle, contrat cassé). Le reste va dans une liste « vu une fois » sans proposition — un
harnais gonflé par des cas isolés coûte plus qu'il ne rapporte.

## Sortie

```
## Correctifs de relecteur, par PR
- PR <id> — <relecteur> a poussé <n> hunk(s) après mon dernier SHA
  - <fichier> — <ce qui manquait> → `<catégorie>`
```

```
## Thèmes récurrents
| # | Thème | Occurrences (PR) | Catégorie dominante |
```

```
## Propositions, du barreau le plus bas au plus haut
### <barreau> — <thème>
- Preuve : <PR + fichier, deux occurrences minimum>
- Changement : <fichier de harnais visé, et le texte exact à coller>
- Coût : <ce que ça retire, ou la ligne d'AGENTS.md que ça remplace>
```

```
## Vu une fois — pas de règle
- <constat> (PR <id>)
```

```
## Arbitrages — rien à corriger
- <constat> (PR <id>)
```

Une section vide s'écrit `_néant_`. Pas de préambule, pas de conclusion.

## Contraintes d'exécution

- Une commande par appel : le garde-fou de worktree refuse les boucles shell, les
  substitutions `$(...)` et les heredocs.
- Avec `bkt`, `--jq` exige `--json` dans la même commande.
- Chaque texte à coller respecte la langue de son fichier cible : anglais pour `AGENTS.md`,
  `.agents/rules/`, `.agents/skills/`, `harness/skills/`, `MEMORY.md` ; français pour un ADR.
- Ne modifie aucun fichier. La décision de promouvoir reste à moi.
