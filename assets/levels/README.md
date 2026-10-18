# Niveaux

Un niveau = un fichier `.ron` dans ce dossier. Les fichiers sont chargés par
ordre alphabétique : préfixez-les (`01_`, `02_`…) pour fixer l'ordre du menu.

## Format

```ron
(
    name: "Mon niveau",        // nom affiché
    length: 120.0,             // position de la ligne d'arrivée (en tuiles)
    height: 10.0,              // hauteur du plafond (optionnel, 10 par défaut)
    start_speed: 1.0,          // multiplicateur de vitesse initial (optionnel)
    background: (0.05, 0.06, 0.16), // couleur de fond RGB (optionnel)
    objects: [
        (kind: Block, x: 20, y: 0),
        (kind: Spike, x: 30, y: 0),
    ],
)
```

* `x` : colonne (minimum 3, le cube démarre à 0). `y` : ligne, 0 = juste au-dessus du sol.
* Types : `Block`, `Platform`, `Spike`, `SpikeDown`, `Checkpoint`, `Portal(...)`.
* Portails : `GravityFlip`, `GravityNormal`, `GravityInverted`, `Speed(1.3)`, `Mini`, `Normal`.

Un niveau invalide est ignoré et l'erreur s'affiche dans la console.
Le test `cargo test` vérifie que les niveaux fournis sont **réalisables**.
