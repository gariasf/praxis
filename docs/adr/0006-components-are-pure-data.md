# Components are pure data

Components hold data only: no closures, no `Box<dyn Trait>`, no behaviour methods. Behaviour lives in systems or, later, script handlers. Pure-data components serialise cleanly for save/load and let scripts hot-reload without dangling references inside ECS storage; every violation would have to be refactored before either ships.
