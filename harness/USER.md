# USER.md — Biais et attentes de l'utilisateur

Préférences de travail, sans information personnelle. Identité de l'agent →
`SOUL.md`, règles techniques → `AGENTS.md`.

## Acquis

Shell, git et Unix ; web moderne (JS/TS, bundlers, npm) ; infra et conteneurs
(Docker, CI/CD, réseau) ; backend et bases de données (SQL, modélisation,
transactions). Aucun rappel de fondamentaux sur ces sujets : aller au fait.

## Biais techniques

- **Simplicité et explicabilité avant élégance.** À qualité comparable, préférer
  la solution concise, testable et facile à reconstruire par un humain ou un agent.
- **Dépendance éprouvée > code maison**, dès que les cas limites sont réels.
  Avant tout ajout, vérifier l'existant, la bibliothèque standard et les API natives.
- **Service avant pureté.** Sur incident : restaurer le service rapidement et tracer
  la cause racine dans une issue ; ne pas pérenniser un workaround d'un défaut possédé.
- **Rien de cassé n'est toléré.** Un test intermittent est un bug jusqu'à
  preuve du contraire : ni `retry`, ni `skip`, on cherche la cause.
- **Refactorer pour simplifier la feature.** Refactorer le chemin directement concerné
  si cela simplifie réellement la feature ; demander avant d'élargir aux abstractions voisines.
- **Revue non bloquante sur le style.** Signaler la sur-abstraction sans en
  faire un motif de blocage ; la décision reste à l'auteur.

## Mode de travail

- **Lecture autonome, écriture contrôlée.** Explorer, rechercher et vérifier librement.
  Une tâche évidente peut être implémentée directement ; sinon, demander validation avant
  toute écriture persistante dès que périmètre, architecture ou hypothèse sont incertains.
- **Validation concise.** Donner le périmètre, les changements principaux, hypothèses,
  incertitudes et une recommandation. Pour un changement complexe, préférer une vue du
  diff conceptuel : arborescence, schéma ASCII, maquette CLI/UI ou équivalent.
- **Plan seulement quand il aide.** Sur une tâche simple et stable, plan court si utile.
  Sur une tâche complexe, explorer sans figer tôt la solution et proposer des issues pour
  les blocages découverts.
- **Rester focalisé.** Prioriser blocages, réduction d'incertitude, fonctionnel, puis dette.
  Une dette non bloquante ne mérite une proposition d'issue que si elle touche directement
  la tâche ; ne jamais l'implémenter sans décision explicite.
- **Continuer ce qui est indépendant.** Un blocage local n'arrête pas un travail sans
  dépendance de code ni dépendance architecturale avec lui.
- **Relais sans ancrage.** Pour un nouvel agent, transmettre faits, contraintes et pistes
  invalidées ; ne transmettre des sources précises que si leur pertinence est certaine,
  et ne pas pré-écrire la solution.
- **Contexte comme ressource.** Si la session devient trop chargée pour un nouveau correctif,
  préférer une issue et un prompt concis pour une nouvelle session. ~60 % de contexte visible
  est un signal d'alerte, pas une limite absolue.
- **Barre de vérification.** Lint, types, tests et CI verts restent la barre par défaut, mais ne
  constituent pas une revue. Avant de demander une revue sur une PR que vous ouvrez, passez
  `merge-verdict` sur votre propre PR et corrigez ses constats bloquants avant de solliciter quiconque.

## Review

- Garder les retours courts, hiérarchisés et centrés sur la PR.
- Un constat important peut devenir une issue. S'il est simple et déjà compris, le reviewer
  peut le corriger immédiatement ; sinon, proposer une issue et un prompt pour un agent frais.

## Points de vigilance

Signaler systématiquement les cas limites et conséquences non voulues du changement :

1. **Gestion d'erreur incomplète** — échecs, timeouts, retours vides/nuls. Sur une écriture
   externe : idempotence nommée, ordre attendu, écriture atomique des valeurs dérivées ; une
   entrée inconnue reste brute, quarantinée et rejouable.
2. **Nommage et lisibilité** — proposer directement de meilleurs noms. Une forte densité de
   commentaires sur le chemin de la tâche est un signal de conception ; proposer une issue,
   sans élargir automatiquement le travail.
3. **Dépendance de trop** — si quelques dizaines de lignes sans cas limites sérieux suffisent,
   proposer d'abord cette option. Pour l'orchestration de promesses, vérifier si les primitives
   natives suffisent avant `p-limit`, `p-map`, `p-queue` ou équivalent.

## Expériences en cours

- Sur le code existant, ~500 lignes modifiées/supprimées = alerte de dérive ; ~800 = forte
  réévaluation : justifier la suite ou proposer un découpage. Exclure code généré, snapshots,
  lockfiles, fixtures et nouveaux tests ; compter les modifications de tests existants.
- Avant une modification non triviale, annoncer les zones prévues et signaler une expansion
  matérielle du périmètre.
