# fun-shooter

## Setup

1. If you're on linux, install `mold`, a linker, for faster iterative compile times
  - Ubuntu/Debian: `sudo apt-get install mold clang`
  - Fedora: `sudo dnf install mold clang`
  - Arch: `sudo pacman -S mold clang`
2. This repository uses Git LFS. Follow the installation instructions [here](https://packagecloud.io/github/git-lfs/install). Afterwards, run `git lfs pull` to get all assets
3. Bevy itself also needs a couple of dependency. You can find installation instructions for Linux, Windows and MacOS [here](https://bevy.org/learn/quick-start/getting-started/setup/#installing-os-dependencies)
4. Install alternative codegen backend: `rustup component add rustc-codegen-cranelift-preview --toolchain-nightly`
5. Run the app
  - `cargo run`

> [!IMPORTANT]
> Some game assets are not included in this repository and are not covered by the GPL. These assets are proprietary and were obtained under separate licenses.
> Release builds of the game may include these assets bundled inside the game binary. The assets are provided only as part of the compiled game and are not redistributed as standalone files.

## Architecture & Project Structure

### Project Structure and Coding Guidelines

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

### Entity Initialization & Readiness

Entities in this project are initialized incrementally.
The presence of an entity does **not** mean it is fully ready.

We use marker components (e.g. `PlayerReady`) to explicitly signal
when an entity has all required components and can be used by
dependent systems (HUD, camera, input, etc).

- Systems must **not** assume ordering between other systems
- Game states gate *systems*, not entity readiness
- Systems that depend on fully initialized entities must query
  for the corresponding `*Ready` marker

### Multiplayer Setup
- Players are spawned on the server and replicated to all connected clients
- A client can then find its own Player
- Character controller is only run on the client
  - Client sends its new position to the server
  - The server validates whether this new position was even feasible by a distance check, comparing new position to old position (to be implemented)
  - The validated position is stored in `PlayerPositionServer`. This component gets replicated to all other clients
  - All clients can then update the `Transform` of that corresponding player
    - This is done via interpolation so it looks smooth. Without the intermediate component `PlayerPositionServer`, we wouldn't be able to add interpolation


For shooting:
- Note that the following below is not yet implemented, just client sends message -> raycast on server
- Server saves the position history of a player in a `VecDeque<(u32, Vec3)>`
  - Gets updated each tick and only last ~200ms are saved
- Clients send a ShootRequest to the server, that contains the necessary information together with a `client_tick`
- Server looks up the position for the given `client_tick`
- Server spawns temporary colliders to make the raycast
- If hit was sucessful, the `Health` component on the corresponding player is updated


#### Example
(Identifiers may be abbreviated for documentation clarity.)
```rust
// Attach equipment
fn add_player_weapon(
    mut commands: Commands,
    q: Query<Entity, Added<Player>>,
) {
    for e in &q {
        commands.entity(e).insert(PlayerWeapon);
    }
}

// Mark readiness once all requirements are present
fn mark_player_ready(
    mut commands: Commands,
    q: Query<Entity, (With<Player>, With<PlayerWeapon>), Without<PlayerReady>>,
) {
    for e in &q {
        commands.entity(e).insert(PlayerReady);
    }
}

// Spawn HUD only for ready players
fn spawn_player_hud(
    mut commands: Commands,
    q: Query<Entity, Added<PlayerReady>>,
) {
    for player in &q {
        commands.spawn(PlayerHud { player });
    }
}
```


## Todo and feature list
- [x] User interface
- [x] Player movement
  - [x] Basic movement
  - [x] Write a proper kinematic character controller from scratch
    - [x] Climb slopes
    - [x] slide along walls when going into walls instead of zeroeing velocity
- [x] Different maps to play on
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
- [x] Weapon animations
  - [x] Weapon sway
  - [x] Weapon recoil animation (e.g. when shooting kick back)
  - [x] Interpolate translation when switching aiming
- [ ] Audio Settings menu
  - [x] Global volume
  - [ ] Audio volume
  - [ ] Music volume
- [ ] Graphics settings menu 
  - [ ] Target FPS
- [ ] Input settings menu
  - [ ] Change keybinds of all inputs in the game
- and probably more stuff already implemented and coming soon..


## idk what to call this section
this project assumes 1 unit = 1m, e.g. a unit is like `Transform::from_xyz(1.0, 1.0, 1.0)`

## Libraries used
uses:
- bevy for game engine
- avian3d for physics
- lightyear for multiplayer

## Credits

### 3D Models
- fps/tps Map by theking1322 [CC-BY](https://creativecommons.org/licenses/by/3.0/) via Poly Pizza (https://poly.pizza/m/wna54gOjL7)
- SWAT by Quaternius (https://poly.pizza/m/Btfn3G5Xv4)
- "LOWPOLY | FPS | TDM | GAME | MAP by ResoForge" (https://skfb.ly/pxM87) by ResoForge (old profile) is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).

### Music & SFX
- Main Menu Theme by [juanjo_sound](https://juanjosound.itch.io/)
