# Documentation

Le [README racine](../README.md) présente le dépôt et ses commandes courantes ;
les [ADR](adr/README.md) consignent les décisions structurantes en vigueur.
Un guide n'est conservé que s'il est hautement qualitatif, concis, à jour et
directement exploitable par un humain comme par un agent.

## Guides

| Sujet                                                | Guide                         |
| ---------------------------------------------------- | ----------------------------- |
| Diagnostic, synchronisation et mesure du harnais     | [Arnes](arnes.md)             |
| Installation optionnelle et connexion des canaux web | [Agent-Reach](agent-reach.md) |
| Évaluations déterministes et essais live manuels     | [Évaluations](arnes-eval.md)  |

## Structure

Les guides vivent uniquement dans `docs/<sujet-kebab-case>.md`, sans numéro
d'issue, date ni suffixe d'audit ou de validation. Un sujet a un seul guide,
indexé ci-dessus. Les instructions, ressources de skills et fixtures exécutées
restent dans leur emplacement canonique ; ce ne sont pas des guides annexes.

Gabarit unique, à remplir sans recopier le code, l'aide CLI ou les inventaires :

```markdown
# Sujet

## Usage

Quand utiliser ce guide et prérequis indispensables.

## Procédure

Commandes nécessaires et résultat à examiner.

## Limites et vérification

Refus, effets sensibles, limites d'observation et contrôle existant pertinent.
```

Corriger ou supprimer un guide devenu faux ou redondant dans le même changement.
Les notes d'issue, audits, validations ponctuelles et récits de livraison vont
dans la restitution ou la PR, pas dans des fichiers versionnés. Extraire uniquement
une décision structurante encore vivante vers l'ADR concernée. Ne créer aucune
gate pour imposer cette structure.
