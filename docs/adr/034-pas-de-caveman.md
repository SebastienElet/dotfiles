# ADR-034 — Ne pas utiliser les skills Caveman

- **Statut** : accepté
- **Date** : 2026-08
- **Commits** : `9e42a94` (adoption), `e5fcd90`, `39530c3` (retrait)

## Contexte

Les skills Caveman, installées en 2026-05, promettaient de comprimer les
réponses de l'agent (annonce : −65 % de tokens de sortie). Le retrait
(`39530c3`) constate trois choses :

- **Elles ne s'activaient pas.** Installées telles quelles sous
  `~/.agents/skills` sans hook `SessionStart`, elles n'ont jamais pu
  s'auto-activer : l'activation automatique que promet leur README provient du
  plugin, non des fichiers de skills.
- **Le coût était payé quand même** : 2,5 Ko de descriptions, soit environ
  620 tokens injectés dans la liste des skills à chaque démarrage de session.
- **Le gain serait négatif ici de toute façon.** `SOUL.md` plafonne déjà les
  réponses à cinq phrases, ce qui place l'usage dans le régime que l'auteur
  documente lui-même dans `docs/HONEST-NUMBERS.md` : en dessous d'environ
  1 500 tokens de sortie par réponse, les règles injectées coûtent plus que la
  compression ne rapporte. Une mesure indépendante plafonne par ailleurs le
  gain réel à −8,5 %, contre les −65 % annoncés.

## Décision

Retirer les cibles Caveman du `Makefile`. La concision reste obtenue par la
contrainte de registre inscrite dans `SOUL.md`
([ADR-027](027-soul-et-user-separes.md)).

## Conséquences

- 620 tokens récupérés à chaque démarrage de session.
- La concision dépend d'une règle lisible et modifiable plutôt que d'un paquet
  tiers.
- Confirme la règle générale
  ([ADR-029](029-pas-de-skills-tierces.md)) : une skill tierce doit prouver son
  déclenchement autant que son gain.

## Alternatives écartées

- Installer le plugin complet pour obtenir l'auto-activation : ferait payer le
  coût réel d'un gain mesuré négatif dans ce régime d'usage.
- Conserver les skills en activation manuelle : le coût de description est dû à
  chaque session, quel que soit l'usage.
