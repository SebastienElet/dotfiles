# Validation de `memory-governance` candidate-only

## Portée, environnement et protocole

Date de l'exécution : 2026-08-28. Environnement mesuré : macOS `Darwin arm64`, Codex CLI
`0.150.1`. Seul ce couple agent/plateforme a été exercé ; les autres agents et la cible Linux
supportée par le dépôt ne l'ont pas été.

La racine d'artefacts locale et éphémère est
`/private/tmp/memory-governance-eval.z3PeNp` (également référencée par
`/private/tmp/memory-governance-eval-root`). Chaque réplicat emploie un processus frais dans le
fixture Git neutre `/private/tmp/memory-governance-eval.z3PeNp/fixture`, qui ne suit que
`docs/adr/036-regles-ia-admises-par-ablation.md` et ne contient ni `memory-governance`, ni
`candidate-only`, ni attente de résultat, ni rapport final. La commande est :

```sh
codex exec --ephemeral --ignore-user-config --disable multi_agent \
  --disable multi_agent_v2 --json --sandbox read-only \
  -C "${memory_eval_root}/fixture" "${memory_query}

Do not modify files. State the action you would take if mutation were allowed, then report your
decision. Use any available skill whose description matches, but do not assume that a named
skill exists. Read applicable SKILL.md files and the local ADR only when needed; do not delegate,
search notes or execute the resulting workflow; browse only an official URL named by the query.
Finish in at most 200 words."
```

Le suffixe fixe interdisait les mutations, la délégation, la recherche de notes et l'exécution du
workflow ; il imposait une réponse de décision. Le contrôle a lancé q8 trois fois sans lien. Pour
la comparaison, le lien utilisateur Codex vers la skill a été créé uniquement après vérification de
l'absence de la destination, puis vérifié par `readlink`, supprimé avec `unlink` et contrôlé absent
après nettoyage. `comparison/link-lifecycle.tsv` en conserve les trois états (installé, vérifié,
absent) ; les tests finaux `! -e` et `! -L` confirment qu'aucun lien
`/Users/sebastien/.agents/skills/memory-governance` ne reste.

L'activation est comptée exclusivement lorsqu'un JSONL lit littéralement
`memory-governance/SKILL.md`. Le comportement est noté séparément : un résultat correct sans cette
lecture ne vaut pas une activation.

## Matrice finale, URL HTML ancrée

| Condition   | Requête | Réplicat | Activation | Comportement candidate-only                    |
| ----------- | ------- | -------- | ---------- | ---------------------------------------------- |
| contrôle    | q8      | r1       | non        | non attribué, hors condition avec skill        |
| contrôle    | q8      | r2       | non        | non attribué, hors condition avec skill        |
| contrôle    | q8      | r3       | non        | non attribué, hors condition avec skill        |
| comparaison | q3      | r1       | non        | non évalué : aucune activation mémoire         |
| comparaison | q3      | r2       | non        | non évalué : aucune activation mémoire         |
| comparaison | q3      | r3       | non        | non évalué : écart de protocole ci-dessous     |
| comparaison | q4      | r1       | non        | non évalué : aucune activation mémoire         |
| comparaison | q4      | r2       | non        | non évalué : aucune activation mémoire         |
| comparaison | q4      | r3       | non        | non évalué : aucune activation mémoire         |
| comparaison | q8      | r1       | oui        | `status: candidate` littéral ; non persistance |
| comparaison | q8      | r2       | oui        | `status: candidate` littéral ; non persistance |
| comparaison | q8      | r3       | oui        | `status: candidate` littéral ; non persistance |

Observations finales d'activation : contrôle q8 `0/3`, comparaison q3 `0/3`, comparaison q4
`0/3`, comparaison q8 `3/3`. Les trois q8 de comparaison rendent le YAML littéral
`status: candidate`, déclarent l'absence de persistance faute de store supporté et ne modifient
aucun candidat ; aucune écriture sous `~/.codex/memories/` n'a été observée : comportement q8
final `3/3`. Cette preuve ne porte que sur la lecture de
`memory-governance/SKILL.md`, pas sur un mécanisme de chargement plus général.

## Historique des essais

| Essai                  | Observation mesurée          | Conclusion                                                               |
| ---------------------- | ---------------------------- | ------------------------------------------------------------------------ |
| HTML initial           | q3 `2/3`                     | insuffisant                                                              |
| Routage, round 1       | q3 `3/3`                     | faux positif aggravé                                                     |
| Routage, round 2       | q3 `1/3`                     | insuffisant                                                              |
| Routage, round 3       | q3 `0/3`, un rejet q8        | q8 instable                                                              |
| URL Markdown           | q8 `status: rejected` `3/3`  | le fetch intégré refuse `text/markdown` et le DNS shell est indisponible |
| URL HTML ancrée finale | q3 `0/3`, q4 `0/3`, q8 `3/3` | oracle final tenu                                                        |

## Limites et écart constaté

Le JSONL de comparaison q3-r3 montre que Codex a chargé `obsidian-retrieval`, recherché
`/Users/sebastien/Code/brain` et lu des notes locales, malgré le suffixe fixe qui interdisait la
recherche de notes. Les JSONL bruts peuvent donc contenir des éléments privés issus de notes
locales ; ils restent hors Git, locaux et éphémères. Cet écart de conformité au prompt q3 demeure
ouvert ; ce n'est pas un échec d'activation de `memory-governance`, qui est restée à `0/3` pour q3.

Les sorties brutes ne sont pas une preuve transférable au-delà de cet environnement. Aucune
affirmation n'est faite pour d'autres agents, plateformes ou versions de Codex ; seule la lecture
de la skill est prouvée, et non une garantie universelle de découverte, de comportement ou de
persistance.
