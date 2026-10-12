# Lua reacts to events

Gameplay logic (quests, dialogue, encounters, item effects) lives in Lua through mlua, with LuaJIT where supported; the engine stays in Rust. Lua responds to engine events and runs coroutines but never authors per-frame systems, which is what lets scripts hot-reload and the engine hold its frame budget. Planned; sequenced by the milestone ladder.
