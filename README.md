# fun-shooter

## Setup

1. If you're on linux, install `mold`, a linker, for faster iterative compile times
  - Ubuntu/Debian: `sudo apt-get install mold clang`
  - Fedora: `sudo dnf install mold clang`
  - Arch: `sudo pacman -S mold clang`
2. This repository uses Git LFS. Follow the installation instructions [here](https://packagecloud.io/github/git-lfs/install). Afterwards, run `git lfs pull` to get all assets
3. Please note that shipping the binary requires disabling dynamic linking. This can be achieved by removing the `dynamic_linking` feature from the dependency `bevy` in `Cargo.toml`
4. Bevy itself also needs a couple of dependency. You can find installation instructions for Linux, Windows and MacOS [here](https://bevy.org/learn/quick-start/getting-started/setup/#installing-os-dependencies)
5. Install alternative codegen backend: `rustup component add rustc-codegen-cranelift-preview --toolchain-nightly`
5. Run the app
  - `cargo run`

## Features
- [x] User interface
- [x] Realistic kinematic player movement
  - [ ] Using "collide and slide" algorithm: https://www.peroxide.dk/papers/collision/collision.pdf
- [X] Game modes
  - [x] Wave mode (the game gets more difficult each round, e.g. more enemies are spawned)
  - [ ] Capture the flag
  - [ ] Deathmatch
- [x] Particle effects
  - [x] Wall/ground bullet impact
  - [x] Blood splatter when hitting enemies
- [x] Enemy AI
  - [x] Enemies check if they can see the player and shoot them
  - [x] Chasing the player via pathfinding
  - [ ] Going to locations the player made noises
- [ ] Multiplayer
- and probably more stuff already implemented and coming soon..

## Project Structure and Coding Guidelines

- This project relies heavily on modularization, to be specific, rust modules
- Every module has its own plugin, e.g.
  - Everything player related goes into the PlayerPlugin
  - Everything related to the user interface goes into the WorldPlugin
  - Plugin declarations must always live in the `mod.rs` of the given module
- Every module is split up into seperate files, following ECS structure, e.g.
  - Components of a module go into `./components.rs`
  - Systems of a module go into `./systems.rs`
  - Messages of a module go into `./messages.rs`
  - and so on
  - This makes it very easy to navigate the codebase and scales well
- If a module has sub modules, like for example movement logic of the player, it will be in `movement/mod.rs`
  - Note that a submodule may have its own plugin.
    - If the submodule doesnt have lots of logic, all code may be located in its `mod.rs` and then be used in the root plugin of given module

## idk what to call this section
this project assumes 1 unit = 1m, e.g. a unit is like `Transform::from_xyz(1.0, 1.0, 1.0)`

## Libraries used
uses:
- bevy for game engine
- avian3d for physics
- skein for bevy <-> blender integration (work with bevy components in blender)

## Credits

### 3D Models
- fps/tps Map by theking1322 [CC-BY](https://creativecommons.org/licenses/by/3.0/) via Poly Pizza (https://poly.pizza/m/wna54gOjL7)
- SWAT by Quaternius (https://poly.pizza/m/Btfn3G5Xv4)
- "LOWPOLY | FPS | TDM | GAME | MAP by ResoForge" (https://skfb.ly/pxM87) by ResoForge (old profile) is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).

### Music & SFX
- Main Menu Theme by [juanjo_sound](https://juanjosound.itch.io/)
