Correction appliquée dans [src/trim-label.js](/synthetic/issue160-calibration/run-01/workspace/src/trim-label.js:1) : `trim()` conserve désormais `""`.

- `npm test` : 3/3 tests réussis
- `git diff --check` : réussi
- Environnement testé : Node.js 26.8.1
- Commentaires ajoutés : aucun

Changement livrable localement. Le projet déclarant Node.js 24, la CI doit encore valider cette version avant fusion.
