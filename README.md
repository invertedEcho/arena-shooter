# fun-shooter

## Compiling from source
1. If you're on linux, install `mold`, a linker, for faster iterative compile times
  - Ubuntu/Debian: `sudo apt install mold clang`
  - Fedora: `sudo dnf install mold clang`
  - Arch: `sudo pacman -S mold clang`
2. Bevy itself also needs a couple of dependency. You can find OS-specific installation instructions [here](https://bevy.org/learn/quick-start/getting-started/setup/#installing-os-dependencies)

## Running the server/client
- To run the server:
    - `cargo run -p server <headless|headful>`
    - You can specify either headless or headful. Headless is useful for running where a window cant be created, e.g. servers. Headful will spawn a window, which may be useful to see the map and the spawned players
    - You can also emit the argument, and the server will be started in headless mode.
- To run the client:
    - `cargo run -p client`

> [!WARNING]
> Please note that I've removed all assets from the repository as I've exceeded free LFS storage.
> As explained below, release builds will contain the assets.

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

### Multiplayer Setup
- Players are spawned on the server and replicated to all connected clients
- A client can then find its own Player
- Character controller is only running on the client
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

## Todo and feature list
- [x] User interface
- [x] Player movement
  - [x] Basic movement
  - [x] Write a proper kinematic character controller from scratch
    - [x] Climb slopes
    - [x] slide along walls when going into walls instead of zeroeing velocity
- [x] Different maps to play on
- [X] Game modes
  - [x] Free for all
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
- [x] Multiplayer
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
- skein for bevy <-> blender integration (work with bevy components in blender)
- lightyear for multiplayer

## Credits

### 3D Models
- fps/tps Map by theking1322 [CC-BY](https://creativecommons.org/licenses/by/3.0/) via Poly Pizza (https://poly.pizza/m/wna54gOjL7)
- SWAT by Quaternius (https://poly.pizza/m/Btfn3G5Xv4)
- "LOWPOLY | FPS | TDM | GAME | MAP by ResoForge" (https://skfb.ly/pxM87) by ResoForge (old profile) is licensed under Creative Commons Attribution (http://creativecommons.org/licenses/by/4.0/).

### Music & SFX
- Main Menu Theme by [juanjo_sound](https://juanjosound.itch.io/)
