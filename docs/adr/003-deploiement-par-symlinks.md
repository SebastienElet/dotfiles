# ADR-003 — Déploiement revalidé de la configuration

- **Statut** : accepté
- **Date** : 2026-08
- **Issue** : [#257](https://github.com/SebastienElet/dotfiles/issues/257)

## Contexte

Les fichiers versionnés sous `home/`, `harness/` et `tooling/` doivent rester liés à leurs
destinations. Une cible fichier Make ne revalidait toutefois pas un lien déjà présent : une
destination divergente pouvait donc satisfaire silencieusement le graphe.

## Décision

Chaque passage d’un profil revalide les destinations possédées. Une destination absente est créée ;
un lien vers la source exacte reste inchangé et silencieux ; tout autre fichier, répertoire ou lien
est préservé et provoque un échec explicite.

Les fichiers assemblés appliquent le même contrat sur leur contenu. `~/.codex/AGENTS.md` reste
assemblé parce que Codex ignore les directives `@import`; un contenu déjà exact conserve son
identité.

## Conséquences

- Une dérive locale n’est ni écrasée ni acceptée comme état convergé.
- Le second `make minimal` ne recrée aucun artefact conforme.
- Les recettes de probe sont masquées ; chaque mutation annonce son effet.

## Alternatives écartées

- Se fier uniquement aux dates Make : une destination étrangère peut paraître à jour.
- Réparer automatiquement une divergence : risque de perte de données locales.
- Copier les sources : divergence garantie entre dépôt et poste.
