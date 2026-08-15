# USER.md — Biais et attentes de l'utilisateur

Préférences de travail, sans information personnelle. Identité de l'agent →
`SOUL.md`, règles techniques → `AGENTS.md`.

## Acquis

Shell, git et Unix ; web moderne (JS/TS, bundlers, npm) ; infra et conteneurs
(Docker, CI/CD, réseau) ; backend et bases de données (SQL, modélisation,
transactions). Aucun rappel de fondamentaux sur ces sujets : aller au fait.

## Biais techniques

- **Dépendance éprouvée > code maison**, dès que les cas limites sont réels.
  Une bibliothèque mature vaut mieux que quarante lignes à maintenir soi-même.
  Voir le point de vigilance correspondant : ce penchant déborde souvent.
- **Service avant pureté.** Sur incident : contournement immédiat, cause
  racine tracée dans un ticket et signalée par un `TODO` qui le référence.
- **Rien de cassé n'est toléré.** Un test intermittent est un bug jusqu'à
  preuve du contraire : ni `retry`, ni `skip`, on cherche la cause.
- **Revue non bloquante sur le style.** Signaler la sur-abstraction sans en
  faire un motif de blocage ; la décision reste à l'auteur.
- **Boy scout, jamais mimétisme.** Le voisinage fixe l'idiome, pas le niveau de
  qualité : ne jamais reproduire un défaut voisin, le corriger s'il est sur le
  chemin, dans un commit séparé.

## Mode de travail

- **Ampleur d'abord.** Petite tâche : agir directement. Plusieurs fichiers ou
  décision d'architecture : trois à cinq lignes de plan, validation, exécution.
- **Désaccord.** Livrer ce qui est demandé *et* l'alternative proposée, pour
  comparaison sur pièces. Ne pas se contenter d'objecter, ne pas bloquer.
- **Périmètre.** Nettoyage adjacent bienvenu, mais dans un commit séparé du
  commit fonctionnel, afin que chacun se révoque indépendamment.
- **Barre de vérification.** Lint et types verts suffisent par défaut ; les
  tests sont exécutés à la demande ou sur un chemin sensible.

## Points de vigilance

Signaler systématiquement, même sans être sollicité, les cas limites et les
conséquences non voulues d'un changement — et en priorité :

1. **Gestion d'erreur incomplète** — cas d'échec, délais d'attente, retours
   vides et nuls.
2. **Nommage et lisibilité** — proposer directement de meilleurs noms plutôt
   que de signaler le problème.
3. **Dépendance de trop** — le réflexe est d'installer une bibliothèque là où
   quelques lignes de TypeScript suffisent. Avant tout ajout : vérifier ce
   qui est déjà installé, puis la bibliothèque standard et les API natives.
   Si le besoin réel tient en quelques dizaines de lignes sans cas limite
   sérieux, proposer ces lignes et laisser la décision.
   Déclencheur typique : l'orchestration de promesses (`p-limit`, `p-map`,
   `p-queue` et apparentés). `Promise.all`, `Promise.allSettled` et une
   boucle avec compteur couvrent la quasi-totalité des besoins ; proposer
   ces quarante lignes avant toute installation.
