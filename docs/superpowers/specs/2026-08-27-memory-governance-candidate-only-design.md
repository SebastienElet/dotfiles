# Gouvernance mémoire candidate-only

## Contexte

La PR 249 introduit une skill de gouvernance pour les invariants durables, mais son premier contrat
laisse entendre qu'un agent peut publier une entrée validée dans un store existant. Aucun adapter,
aucune identité stable, aucune opération atomique et aucun oracle de rejeu ne matérialisent cette
publication. Une source peut donc changer entre la validation et l'écriture, et un retry au résultat
inconnu peut dupliquer ou publier une entrée périmée.

La mémoire locale Codex ne fournit pas la surface d'écriture manquante. La documentation OpenAI la
décrit comme un état généré sous `~/.codex/memories/`, et demande de ne pas traiter l'édition manuelle
de ces fichiers comme surface de contrôle principale. Le pilote reste donc une politique d'admission
et de restitution de candidats, sans persistance par la skill.

La PR contient aussi des scénarios d'activation déclaratifs, mais aucune exécution enregistrée. Cela
ne prouve ni que Codex charge la skill, ni que ses refus tiennent sous pression. L'ADR-036 impose une
mesure comportementale avant de retenir une nouvelle règle d'agent.

## Objectif

Rendre le pilote honnête et vérifiable : la skill produit uniquement des candidats sourcés, refuse
toute écriture directe dans un état mémoire généré, et ne permet d'utiliser un candidat qu'après un
contrôle de fraîcheur de sa source au moment de la consommation. Des exécutions Codex fraîches
mesurent séparément l'activation et le comportement.

## Frontière de persistance

`memory-governance` ne possède aucun sink persistant. Elle ne crée, ne modifie et ne supprime aucun
fichier sous `~/.codex/memories/`, même si l'utilisateur le demande explicitement. Elle ne définit
pas non plus de contrat générique de store sans implémentation supportée.

Toute demande de mémorisation aboutit à l'un des résultats suivants :

- `candidate` si l'autorité primaire est actuelle, vérifiable et satisfait le contrat d'admission ;
- `rejected` si la source manque, si le contenu est dérivable, volatil, privé, ou constitue un
  contournement d'un défaut possédé ;
- `invalidated` pour un candidat antérieur dont la source, la révision, le statut, le scope ou la
  condition d'invalidation ne tient plus.

Une future persistance demandera une évolution séparée, seulement lorsqu'une API supportée expose
une identité stable, une publication atomique liée à la révision validée et un rejeu idempotent.

## Contrat du candidat

Le candidat conserve les champs existants et rend la révision obligatoire dans `source` : le
locator stable est suivi de la révision, de la version ou du statut effectivement contrôlé. Le
champ `validated_at` situe la preuve dans un environnement précis ; il ne constitue pas un TTL.
Le champ `invalidate_when` nomme un événement observable, pas une durée arbitraire.

Le statut `validated` disparaît du flux pilote : il suggère une publication ou une confiance
durable que la skill ne peut garantir. `candidate` signifie seulement que l'entrée a passé
l'admission au moment indiqué et peut être transmise à l'utilisateur ou à un workflow externe.

## Consommation et fraîcheur

Avant toute utilisation d'un candidat antérieur, l'agent relit la source primaire et vérifie son
statut, son scope, sa révision et `invalidate_when`. Si la source est indisponible, si sa révision a
changé ou si la vérification est ambiguë, l'agent refuse d'appliquer l'entrée et la retourne avec le
statut `invalidated`. Le contenu mémorisé ne sert jamais de preuve de sa propre fraîcheur.

Ce contrôle se trouve au sink réel du pilote : la décision d'utiliser l'invariant dans la tâche
active. Une vérification faite seulement lors de l'admission ne protège pas une session future.

## Preuve comportementale

Les prompts canoniques restent dans
`harness/skills/memory-governance/evals/trigger-queries.json`. Chaque run utilise un nouveau
processus `codex exec --ephemeral`, un dépôt temporaire propre et un mode non mutant. Les réponses
brutes restent des artefacts locaux ; un rapport versionné conserve l'environnement, le protocole,
les prompts, l'activation observée et le verdict comportemental.

Deux conditions exécutent les mêmes prompts au moins trois fois :

1. contrôle sans installation de `memory-governance` ;
2. comparaison avec la skill installée par son lien utilisateur Codex supporté.

L'activation Codex est prouvée uniquement par une lecture de
`~/.agents/skills/memory-governance/SKILL.md` dans le JSONL. Le verdict comportemental est noté
indépendamment et couvre : source absente, contournement d'un défaut possédé, données privées,
contrainte externe versionnée, demande de modification d'`AGENTS.md`, et réutilisation après
changement de révision.

Le lien temporaire de comparaison est créé seulement si la destination est absente, puis supprimé
après les runs. Les prompts interdisent toute mutation ; le sandbox est `read-only`. Une activation
manquée ne peut pas être masquée par un verdict comportemental correct par défaut.

## Fichiers

- `harness/skills/memory-governance/SKILL.md` porte le contrat candidate-only et le contrôle au
  moment de la consommation.
- `harness/skills/memory-governance/evals/trigger-queries.json` ajoute les scénarios de pression et
  de fraîcheur sans devenir une preuve à lui seul.
- `docs/memory-governance-validation.md` conserve la preuve normalisée des runs Codex réellement
  exécutés.

Le README dérivé ne change que si `sync-index` détecte une différence de description ; aucune règle
de routage externe ni aucun script d'eval permanent n'est ajouté.

## Vérification

Le doctor complet de `memory-governance` doit préserver tous ses PASS initiaux, avec
`skills-ref` explicitement signalé indisponible si absent. Les scénarios Codex doivent être
exécutés, le JSON d'eval validé, puis `sync-index` doit produire deux sorties byte-identiques.
Les tests de déploiement, le lint, le typecheck et la suite du dépôt doivent rester verts sur les
extensions touchées ; les limites de plateforme sont consignées avec l'environnement mesuré.

## Alternatives écartées

- Un contrat abstrait de store : il promettrait atomicité et idempotence sans sink ni oracle.
- Un store propriétaire : il élargirait la PR, dupliquerait l'état généré de Codex et demanderait
  sa propre architecture de cycle de vie.
- L'édition directe de `~/.codex/memories/` : elle dépend d'un format généré non contractuel et
  contourne la surface de contrôle supportée.
- Une vérification périodique : un TTL ne détecte ni un changement immédiat d'ADR ni un changement
  de scope ; la consommation doit vérifier l'événement observable.
