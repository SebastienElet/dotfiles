# ADR-037 — Commentaire absent par défaut, comptabilisé dans le compte rendu

- **Statut** : accepté
- **Date** : 2026-08
- **Commits** : `3068016`

## Contexte

`ai/AGENTS.md` portait trois puces sous `## Code Style` : « All comments and documentation in
English », « Prefer self-documenting code over comments » et « Only comment when explaining _why_,
not _what_ ». Les deux dernières sont des propriétés de la prose, jugées pendant qu'on l'écrit ;
l'[ADR-036](036-regles-ia-admises-par-ablation.md) pose qu'une règle ne lie que si elle exige un
**acte observable**, dont l'absence se voit dans la réponse finale.

La mesure existe. La règle était en contexte et a été violée trois fois dans la PR #51 (issue #52).
Une première ADR-037, révoquée depuis, y répondait par un hook `PostToolUse` refusant tout bloc de
commentaires ajouté de plus de trois lignes, déployé vers Claude, Codex et Cursor. Elle a été
retirée dans #59 sur trois constats :

- le hook n'attrapait qu'une des trois violations de #51 — les deux autres étaient une paraphrase
  d'une et de deux lignes, et seule la longueur est décidable mécaniquement ;
- abaissé à _tout_ commentaire ajouté, il refusait 42 des 67 fichiers de code suivis, soit 247 blocs
  dont 175 d'une seule ligne, majoritairement de vrais _why_ sur du comportement tiers qu'aucun
  renommage n'exprime ;
- Codex exige une approbation interactive par entrée de `hooks.json`, et l'`afterFileEdit` de Cursor
  est documenté comme observationnel : l'objection n'atteint jamais cet agent.

La décision de #59 était d'attendre un correctif côté modèle plutôt que de compenser localement,
`anthropics/claude-code#65961` rapportant le commentaire verbeux comme un défaut par défaut que les
instructions ne suppriment pas. Le présent ADR ne revient pas sur le hook : il traite la prose
restée en place, qui redisait mot pour mot ce qui avait déjà été mesuré comme inopérant.

## Décision

`harness/AGENTS.md` § `Code Style` est réécrit en deux puces :

```markdown
- **Write no comment.** One is admissible only when it records a fact living outside the file
  — upstream defect, protocol quirk, deliberate deviation — and names that fact. Doc comments
  a project's tooling requires are out of scope.
- List in the delivery note every comment you added, and the outside fact each one records.
  When no comments were added, omit only this inventory; retain the checks performed,
  their execution environment and the supported targets not exercised.
```

Le défaut est inversé : zéro commentaire, une exception nommée. La seconde puce est le seul acte
observable du bloc — c'est elle qui distingue cette rédaction de celle que #59 a mesurée comme
inopérante, et c'est la seule qu'une ablation peut isoler. Le mécanisme visé est indirect : rendre
le commentaire coûteux à justifier pour qu'il ne soit pas écrit.

Deux clauses disparaissent. La règle de langue, parce que la question se tranche là où elle est
décidable : `docs/AGENTS.md` impose le français sous `docs/` ici, l'`AGENTS.md` du projet décide
ailleurs, et un défaut global n'atteint que les projets muets. L'interdiction explicite de commenter
l'édition (« added », « now handles »), parce qu'un tel commentaire enregistre un fait _interne_ au
fichier et que le test d'admissibilité le refuse déjà ; la garder reposait sur l'intuition qu'un
contre-exemple concret lie mieux qu'un test abstrait, c'est-à-dire sur le sédiment que
l'[ADR-035](035-agents-md-elague-par-mesure.md) arrête.

## Conséquences

- **Admission initiale non mesurée, contrairement à ce qu'exige l'ADR-036.** L'écart d'origine
  est assumé : à l'adoption, ces deux puces étaient « supposées portantes », pas « constatées ».
  La comptabilité dans le compte rendu est une facette à isoler par ablation : issue #78,
  qui porte le protocole, les facettes et les quatre issues possibles.
- Le risque symétrique est la sur-correction : les 175 commentaires d'une ligne mesurés dans #59
  sont précisément ce que la clause d'exception doit préserver. Une mesure qui les verrait
  disparaître infirmerait la rédaction, pas seulement son intensité.
- La seconde puce exige le compte rendu des commentaires effectivement ajoutés et de leur
  justification. En l'absence de commentaire ajouté, cet inventaire est omis ; la règle
  d'admissibilité et l'obligation de justifier chaque ajout restent inchangées.
- La règle de langue applicable à ce dépôt n'est plus énoncée qu'à deux endroits : la contrainte du
  skill `dotfiles` pour l'anglais, `docs/AGENTS.md` pour l'exception française. Le skill a été
  corrigé en conséquence, sa formulation absolue contredisant l'exception.
- Le numéro 037 est réemployé. L'ADR-037 précédente portait sur le même sujet et n'est plus en
  vigueur ; le `README.md` n'enregistrant que les décisions en vigueur, elle ne laisse pas de trace
  à préserver. Les corps de commit `#55` et `#59` restent la source sur ce point.

## Alternatives écartées

- **Mesurer avant d'écrire**, comme l'ADR-036 le demande : écarté par décision explicite, la
  rédaction d'abord. Reste ouvert.
- **Garder « why, not what »** : mesuré inopérant, la règle étant en contexte lors des trois
  violations de #51.
- **Réintroduire le hook** : traité et tranché par #59, sur mesure. Un correctif côté modèle est
  attendu.
- **Un plafond chiffré** (« at most one comment per changed hunk ») : comptable et vérifiable, mais
  il autorise mécaniquement un commentaire là où zéro suffisait, et déplace la question de
  l'admissibilité vers la quantité.
