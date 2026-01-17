# create_colliders_json

This needs some work so it can be run as standalone binary.
I just put the code here so i can re-use it later.

This binary is supposed to generate colliders for a given map, and export them as a json file
`.json` is definitely not the right file format for this, but it's fine for now.

How?:
- Spawn map from gltf model
- Insert ColliderConstructorHierarchy from avian, which generates colliders from mesh data from the gltf model
- When avian is done, signalled by `ColliderConstructorHierarchyReady`, we can query for all colliders and export them into a `.json` file
