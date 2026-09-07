# ADR-003 — Déploiement des configurations et rejeu

- **Statut** : accepté
- **Date** : 2026-08
- **Révision** : 2026-09-07

## Contexte

La décision du 30 août décrivait uniquement une installation depuis un état propre, alors
que les recettes conservaient des contrôles de liens, une migration des includes Git et une
restauration des fichiers Fisher. La migration Moon conserve ces comportements explicitement,
plutôt que supprimer les protections exercées par les tests.

## Décision

Moon porte les déploiements migrés ; Make reste transitoire pour les opérations restantes.
Pour un lien géré, une destination absente est créée, un lien vers la source attendue reste
inchangé et silencieux, et une autre destination est conservée avec un échec explicite.
Ce contrôle compare le lien attendu ; il ne certifie pas le contenu de toute sa cible.

Les instructions Codex restent assemblées depuis les sources du harnais, car Codex ignore
les directives `@import`. Le résultat identique n'est pas réécrit ; un nouveau résultat
est écrit dans un fichier temporaire puis renommé, sans écrire à travers un lien existant.
La définition d'agent Codex garde son déploiement par copie.

Les comportements de configuration spécifiques restent locaux à leurs utilitaires :

- Git ajoute l'include XDG avant de retirer l'ancien include connu, conserve les autres valeurs,
  et propage les échecs de lecture, d'ajout ou de suppression ;
- Fisher conserve son installation des fichiers fzf et la restauration des fichiers sauvegardés
  en cas d'échec ; cette restauration n'est pas une transaction sur tous les fichiers du plugin.

Les destinations et les sources restent celles de l'ADR-038. Une suppression ou reconstruction
plus large est une action explicite ; `make clean` conserve sa portée actuelle limitée et
ne constitue pas une remise à zéro générale du poste.

## Conséquences

- Le second passage peut être silencieux sans réécrire les artefacts observés.
- Les tests de déploiement exercent les collisions, le rejeu et les erreurs des utilitaires.
- Les oracles Arnes évaluent séparément leurs ressources ; l'installation ne remplace pas
  leurs diagnostics.
- Aucune réparation générale des destinations divergentes n'est ajoutée à l'installateur.

## Alternatives écartées

- Supprimer les contrôles existants pendant la migration : modifie le comportement du poste.
- Écraser un lien divergent pour faire réussir le profil : détruit une configuration non attendue.
- Confondre rejeu silencieux et conformité de tout le poste : dépasse les artefacts observés.
