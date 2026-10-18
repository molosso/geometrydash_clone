<div align="center">
Cube Dash

Un jeu de plateforme rythmique open source, écrit entièrement en Rust avec Bevy.

Saute, esquive les pics, traverse les portails… et ne meurs pas.

Rust Bevy Licence Plateformes

Fonctionnalités • Installation • Contrôles • Créer un niveau • Architecture • Roadmap

</div>
Aperçu
<!-- Ajoute tes captures dans docs/ puis décommente : <p align="center"><img src="docs/menu.png" width="45%"> <img src="docs/gameplay.gif" width="45%"></p> -->

Captures d'écran et GIF à venir dans docs/.

Fonctionnalités
	
Gameplay	Cube à saut automatique, gravité, double saut optionnel, caméra à défilement automatique
Mort & progression	Réapparition, checkpoints, pourcentage, compteur de tentatives, chronomètre, score
Portails	Inversion de gravité, gravité normale/inversée, vitesse variable, mini, taille normale
Obstacles	Blocs, plateformes, pics (sol et plafond), ligne d'arrivée
Niveaux en RON	Ajoute un niveau en déposant un fichier dans assets/levels/
Physique robuste	Simulation par sous-pas de 1/240 s : le cube ne traverse jamais un obstacle
Niveaux garantis réalisables	Un robot de test vérifie que chaque niveau fourni peut être terminé
Installation
Prérequis
Rust (dernière version stable)
Linux uniquement, dépendances système de Bevy :
bash
# Debian / Ubuntu
sudo apt install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev \
    libwayland-dev libxkbcommon-dev

Sous Windows et macOS, rien d'autre n'est nécessaire.

Lancer le jeu
bash
git clone <url-du-depot>
cd cube_dash

# Développement : compilation incrémentale rapide
cargo run --features dev

# Version optimisée
cargo run --release

La première compilation de Bevy prend plusieurs minutes. Les suivantes sont rapides.

Contrôles
Action	Touches
Sauter (maintenir = saut en boucle)	Espace · ↑ · W · clic gauche
Pause	Échap
Recommencer le niveau	R
Menu : choisir un niveau	↑ / ↓
Menu : lancer	Entrée
Menu : double saut ON/OFF	D
Pause : quitter vers le menu	Q
Victoire : niveau suivant / rejouer / menu	Entrée / R / Échap
Créer un niveau

Un niveau est un fichier .ron dans assets/levels/. Les fichiers sont chargés dans l'ordre alphabétique : préfixe-les (01_, 02_…) pour fixer l'ordre du menu.

ron
(
    name: "Mon niveau",
    length: 120.0,                    // ligne d'arrivée, en tuiles
    objects: [
        (kind: Spike, x: 20, y: 0),
        (kind: Block, x: 30, y: 0),
        (kind: Portal(Speed(1.4)), x: 50, y: 0),
        (kind: Checkpoint, x: 60, y: 0),
    ],
)
x : colonne (minimum 3), y : ligne (0 = juste au-dessus du sol).
Objets : Block, Platform, Spike, SpikeDown, Checkpoint, Portal(...).
Portails : GravityFlip, GravityNormal, GravityInverted, Speed(1.3), Mini, Normal.

Un niveau invalide est ignoré et l'erreur s'affiche dans la console. Détails complets dans assets/levels/README.md.

Architecture

Chaque domaine est un module indépendant exposant un Plugin Bevy.

src/
├── main.rs         point d'entrée (une seule ligne utile)
├── lib.rs          racine de la bibliothèque
├── app.rs          assemblage des plugins
├── game.rs         états globaux et ordre des systèmes
├── gameplay/       mise en place d'une partie, pause, victoire, statistiques
├── physics/        simulation du cube, 100 % pure et testée
├── collision/      index spatial des obstacles (recherche binaire)
├── levels/         format RON, chargement, niveaux intégrés
├── player/         entrées, sous-pas de simulation, rendu du cube
├── obstacles/      entités visuelles : sol, blocs, pics, portails
├── portals/        apparence des portails
├── camera/         caméra à défilement automatique
├── menu/           menu principal
├── ui/             ancrage à l'écran et HUD
└── utils/          constantes et conversions de coordonnées

assets/
├── levels/         niveaux .ron
└── audio/ fonts/ textures/ shaders/ ui/

tests/              tests d'intégration

Choix de conception

La physique ne dépend pas de Bevy : c'est une fonction pure step(état, entrées, niveau, dt), donc rapide à tester et déterministe.
Les collisions viennent des données du niveau, pas des entités affichées : le rendu ne ralentit pas la simulation.
Chaque image est découpée en sous-pas de 1/240 s pour éviter qu'un obstacle fin soit traversé.
Tests et qualité
bash
cargo test                                   # unitaires + intégration
cargo clippy --all-targets -- -D warnings    # analyse statique
cargo fmt --all                              # formatage

Les tests couvrent la physique (saut, atterrissage, mort, portails, checkpoints, double saut), le format des niveaux, le score, et vérifient que chaque niveau fourni est réalisable.

Roadmap
 Fondations : états, caméra, plugins
 Joueur, physique, collisions
 Niveaux RON, obstacles, portails, checkpoints
 Effets : particules, animation de mort, shake caméra, parallax, background animé
 Interface complète : sélection de niveau, paramètres, records, police avec accents
 Sauvegarde : progression, meilleurs scores, niveaux débloqués, paramètres
 Audio : musique, sons de saut et de collision, volume réglable
 Éditeur de niveaux intégré
 Docker (développement et production)
 CI/CD : GitHub Actions, releases automatiques, image Docker
Contribuer

Les contributions sont les bienvenues : ouvre une issue pour discuter d'une idée, puis une pull request. Avant de proposer ton code, vérifie que cargo fmt, cargo clippy et cargo test passent.

Licence

Distribué sous licence MIT.
