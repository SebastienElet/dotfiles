# ADR-031 — Lecture web générique via Agent-Reach

- **Statut** : accepté
- **Date** : 2026-10

## Contexte

Le parcours précédent utilisait le fetch intégré, puis Scrapling et
CloakBrowser. La [décision révisée de l'issue #502](https://github.com/SebastienElet/dotfiles/issues/502#issuecomment-6171657742)
retient Agent-Reach à leur place et accepte les pertes de capacité explicitées
dans la [PR #506](https://github.com/SebastienElet/dotfiles/pull/506).

La recette web empaquetée d'Agent-Reach appelle le service hébergé Jina Reader
par `curl` ; elle ne fournit pas de navigateur générique. Les sondages publics
de la PR #506 ont obtenu du contenu statique, mais aussi un refus `403`, des
extractions JavaScript incomplètes et des CAPTCHA bloqués. Ils ne prouvent ni
une équivalence avec les anciens lecteurs, ni une disponibilité durable.

## Décision

Pour la lecture web générique, utiliser la recette Jina Reader du plugin
Agent-Reach. Retirer du parcours géré Scrapling, CloakBrowser et les lecteurs
web génériques redondants. Les recettes Agent-Reach propres à une plateforme
gardent leurs conditions d'accès distinctes ; elles ne sont pas des recours
génériques au navigateur.

N'admettre au Reader hébergé que les URL publiques non sensibles, après examen
du chemin et des paramètres. Ne pas lui transmettre implicitement des URL
privées, des liens d'accès, des données personnelles ou sensibles, ni des
ressources locales. Une exception pour ces contenus exige une autorisation
explicite nommant le service, les données et la finalité. Secrets, cookies,
jetons, profils et sessions ne sont pas transmis au Reader. La lecture web
générique privée ou authentifiée n'a pas de recours dans ce parcours.

Une lecture n'est réussie que si le contenu demandé est présent : un statut
HTTP `200` ou un texte partiel ne suffit pas. Signaler les réponses vides ou
incomplètes, les blocages et les indisponibilités. Aucun recours navigateur,
furtif ou CDP ne complète Jina dans le parcours générique.

## Conséquences

- Le dépôt ne déploie plus ses lanceurs et images Scrapling/CloakBrowser pour
  la lecture web générique. Cette décision ne préjuge pas des outils externes
  ou des configurations utilisateur hors du périmètre géré.
- Le parcours générique perd la lecture locale, privée ou authentifiée,
  l'interaction navigateur, l'accès au DOM complet, la continuité de session
  et le recours furtif ou CDP. Ces pertes ont été acceptées dans l'issue #502
  et livrées par la PR #506.
- Jina est un service tiers : fraîcheur, disponibilité, latence, quotas et
  tarifs dépendent de ce service. Les instructions limitent les données qu'on
  lui confie ; leur présence ne prouve pas que chaque agent les applique.

## Alternatives écartées

- Conserver l'escalade fetch intégré → Scrapling → CloakBrowser : la décision
  révisée de l'issue #502 retire ces lecteurs du parcours générique.
- Considérer Jina comme un remplacement fonctionnel équivalent d'un navigateur :
  les sondages de la PR #506 montrent du contenu manquant et des blocages.
