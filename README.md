```markdown
# Cube Dash

<p align="center">

A fast-paced 2D rhythm platformer inspired by Geometry Dash, built entirely with Rust and Bevy.

</p>

<p align="center">

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Engine-Bevy-blue)
![Docker](https://img.shields.io/badge/Docker-Ready-2496ED?logo=docker)
![License](https://img.shields.io/badge/License-MIT-green)

</p>


## Overview

Cube Dash is a 2D rhythm-based platform game developed in Rust using the Bevy game engine.

The objective is simple: control a cube, avoid obstacles, jump over traps, and complete levels synchronized with music.

The project focuses on:

- Rust game development
- Entity Component System architecture
- Real-time physics
- Collision detection
- Game state management
- Performance optimization
- Containerized development


## Features

### Gameplay

- Smooth cube movement
- Gravity and jumping mechanics
- Automatic level scrolling
- Platform collision system
- Obstacles and traps
- Death and restart system
- Progress tracking
- Multiple levels


### Visual Effects

- Cube animations
- Particle effects
- Camera effects
- Dynamic backgrounds
- Smooth transitions


### Audio

- Background music support
- Jump sounds
- Collision sounds
- Event-based audio system


### Game Systems

- Main menu
- Pause menu
- Level selection
- Score system
- Save system
- Checkpoints
- Settings management


## Technologies

| Technology | Usage |
|------------|-------|
| Rust | Main programming language |
| Bevy | Game engine |
| Cargo | Dependency management |
| Docker | Development environment |
| GitHub Actions | CI/CD |


## Architecture

The project follows a modular ECS architecture:

```
src/
│
├── main.rs
├── app.rs
├── game.rs
│
├── player/
│   ├── movement.rs
│   ├── physics.rs
│   └── animation.rs
│
├── obstacles/
│
├── collision/
│
├── camera/
│
├── audio/
│
├── particles/
│
├── levels/
│
├── ui/
│
└── save/
```


## Installation

### Requirements

Install:

- Rust
- Cargo
- Docker (optional)


Check Rust installation:

```bash
rustc --version
cargo --version
```


## Run locally

Clone the repository:

```bash
git clone https://github.com/YOUR_USERNAME/cube-dash.git

cd cube-dash
```


Install dependencies:

```bash
cargo build
```


Launch the game:

```bash
cargo run
```


## Docker

Build the image:

```bash
docker build -t cube-dash .
```


Run the container:

```bash
docker run cube-dash
```


Using Docker Compose:

```bash
docker compose up
```


## Controls

| Action | Key |
|--------|-----|
| Jump | Space |
| Restart | R |
| Pause | ESC |


## Development

Format the code:

```bash
cargo fmt
```


Run code analysis:

```bash
cargo clippy
```


Run tests:

```bash
cargo test
```


## Roadmap

Future improvements:

- [x] Basic player movement
- [x] Physics system
- [x] Collision detection
- [ ] Level editor
- [ ] Online leaderboard
- [ ] More game modes
- [ ] Custom skins
- [ ] Shader effects
- [ ] Mobile support


## Screenshots

Add screenshots here:

```
assets/screenshots/
```

Example:

![Gameplay](assets/screenshots/gameplay.png)


## Contributing

Contributions are welcome.

Steps:

1. Fork the repository
2. Create a branch

```bash
git checkout -b feature/new-feature
```

3. Commit changes

```bash
git commit -m "Add new feature"
```

4. Push your branch

```bash
git push origin feature/new-feature
```

5. Open a Pull Request


## License

This project is licensed under the MIT License.

See the `LICENSE` file for more information.


## Author

**Melissa Boulsan**

Rust Developer | Software Engineering | Game Development

GitHub:
https://github.com/melbouls
```

Petit conseil pour GitHub : ajoute ensuite un dossier :

```
assets/screenshots/
```

avec une capture du jeu ou un GIF. Le README paraîtra beaucoup plus professionnel sur ton portfolio.
