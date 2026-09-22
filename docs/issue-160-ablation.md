# #160 — Inventaire des commentaires conditionnel

La formulation C2 d’A044 a été validée par le mainteneur après présentation du diff et des
preuves. Elle omet l’inventaire lorsque rien n’a été commenté, conserve la justification de
chaque commentaire ajouté et rappelle de préserver les contrôles effectués et leurs limites.
L’admissibilité des commentaires reste inchangée. L’ADR-037 reprend cette formulation.

## Écart observé

Codex CLI 0.154.0, modèle demandé `gpt-5.6-sol`, effort `medium`, macOS 27 arm64. Sources figées
à `4d433cd` ; les skills de revue ont évolué depuis. Aucun résultat Claude ou Cursor n’est inféré.

| Facette et scénario                                                 | Actuel A | Retrait B | Variante C1 | Variante retenue C2 |
| ------------------------------------------------------------------- | -------: | --------: | ----------: | ------------------: |
| Inventaire vide dans les messages après correction sans commentaire |      3/3 |       0/3 |         1/3 |                 0/2 |
| Tests existants exécutés et réussis                                 |      3/3 |       3/3 |         3/3 |                 2/2 |
| OS/runtime attribués et Linux non exercé explicite                  |      3/3 |       3/3 |         2/3 |                 2/2 |
| Commentaire ajouté et fait externe rapportés, scénario distinct     |      3/3 |       3/3 |         3/3 |                 2/2 |

C1 n’est pas retenue : une réponse omet l’OS et la réserve Linux. C2 est une précision adaptative,
testée deux fois par scénario, sans placebo. Le scénario positif demande explicitement un
commentaire et son compte rendu ; sa justification spontanée n’est pas mesurée. Une réponse C2
ajoute aussi une réserve générique sur les autres plateformes : pas de gain global de concision
ni de non-régression générale revendiqué. Deux réponses A/B surestiment l’indisponibilité de
Node 24 ; trois autres réserves d’attestation sont conservées dans les scores.

Les contrôles indépendants confirment les périmètres des 43 exécutions comparables, les 11
corrections passant les trois tests existants sous Node 24.21.0 et les 11 fichiers commentés
acceptés par `node --check`. Ces contrôles ne prouvent rien pour Linux. Présence et empreintes
sont vérifiées ; injection globale exacte non attestée ; lectures de skills observables.
Notation indépendante partiellement masquée, modèle/effort du notateur non exposés.

Après application, huit tests existants d’assemblage et de liens passent sur macOS. Les helpers
natifs produisent les liens Claude et un assemblage Codex identique octet pour octet à C2 évaluée.
Idempotence, retour à l’ancien texte et réapplication sont vérifiés en fixture, sans déploiement
dans le home personnel. Ces contrôles structurels ne sont pas de nouveaux runs comportementaux.

## Décisions et couverture

- **A044 : rendre conditionnelle**, proposition C2 appliquée après validation ; résultat borné.
- **S010 : conserver provisoirement**. Sur 18 exécutions, les trois conditions explicitent
  l’hypothèse utile et évitent l’annonce superflue, 3/3 par situation. Effet marginal non établi.
- **A021/A022 : conserver provisoirement, non jugeables**. Les traces natives de délégation
  ne permettent pas d’attribuer création et résultat ; les premières notes sont trop répétitives.
- **A039.a : conservation provisoire historique**, résultat non concluant du précédent lot.

L’[inventaire](issue-160-inventory.md) contient 209 unités repérées, dont quatre ajouts mémoire
non évalués depuis les 205 unités initiales. Ce lot compare A044 et S010 ; il ne clôture pas #160.
#255 reste la capacité transversale, sans nouveau framework. #297, inspectée au head `d6efa710`,
ne modifie pas ces textes. Doctor #111 et les instructions projet sont hors périmètre.

## Preuves conservées

Les [rapports historiques](https://github.com/SebastienElet/dotfiles/tree/69035d09a248bf72cded2b6b9bf428439ae4707d/docs)
et l’[archive du nouveau lot](https://github.com/SebastienElet/dotfiles/blob/69035d09a248bf72cded2b6b9bf428439ae4707d/docs/evidence/issue-160-20260922.tar.gz)
sont conservés au commit `69035d0`, référencé par `codex/160-evidence`, hors du diff à intégrer.
L’archive contient prompts, fixtures, sources, variantes exactes, arguments natifs, événements,
diffs, scores, contrôles, répétitions, coûts et limites. `publication.json` associe les empreintes
originales et publiées : chemins opérateur et identifiants de threads natifs sont normalisés.
Les empreintes historiques concernent les originaux, pas automatiquement les copies normalisées.

Le lot compte 53 exécutions principales : 43 comparables, un contrôle technique et neuf tentatives
initiales exclues, dont une interrompue. Le profil initial exposait le catalogue des applications
du compte malgré le home isolé ; ses événements restent privés et ses limites sont conservées.
Le profil corrigé désactive explicitement applications et plugins avec les options natives.
Les anciens rapports ne constituent pas une attestation d’absence de ces capacités natives.

Compteurs disponibles : **7,60 USD d’équivalent API indicatif**, aux tarifs standard Sol du
22 septembre 2026 ; authentification ChatGPT Pro, aucune facture API. Interruption, coordination,
revue et éventuelle consommation enfant non exposée ne sont pas intégralement comptées.
Quota hebdomadaire partagé observé : 29 % → 32 %, sans attribution exclusive.
