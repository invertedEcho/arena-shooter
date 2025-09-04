# fun-shooter

## Project Structure and Coding Guidelines

- Each part of the game is a seperate module
- Every module of the game has its own plugin, e.g.:
  - Everything player related goes into the PlayerPlugin
  - World generation goes into the WorldPlugin
  - Plugin declarations must always live in the `mod.rs` of the given module
- If a module has sub modules, like for example Player has the submodule movement, it will be in `movement/mod.rs`
  - Note that a submodule always has its own plugin.
  - Every module, no matter if submodule or not, needs to be split up into seperate files, always following ECS structure, e.g. one file `components.rs` for all components of the given module, or a `systems.rs` for all systems of the given module.
  - This makes it very easy to navigate the codebase

## idk what to call this section
this project assumes 1 unit = 1m, e.g. a unit is like `Transform::from_xyz(1.0, 1.0, 1.0)`

## Libraries used
uses:
- avian3d for physics
- skein for bevy <-> blender integration (work with bevy components in blender)

## Attributions

- WA 2000 by abc08002 [CC-BY](https://creativecommons.org/licenses/by/3.0/) via Poly Pizza (https://poly.pizza/m/wns2a88122)
- fps/tps Map by theking1322 [CC-BY](https://creativecommons.org/licenses/by/3.0/) via Poly Pizza (https://poly.pizza/m/wna54gOjL7)
