//! General engine API bindings for Lua scripts.

use mlua::Lua;
use praxis_utils::Result;

/// Registers general engine functions with the Lua environment.
pub fn register_engine_api(lua: &Lua) -> Result<()> {
    let engine_table = lua.create_table()?;

    engine_table.set(
        "log_info",
        lua.create_function(|_, msg: String| {
            praxis_utils::info!("[Script] {}", msg);
            Ok(())
        })?,
    )?;

    engine_table.set(
        "log_debug",
        lua.create_function(|_, msg: String| {
            praxis_utils::debug!("[Script] {}", msg);
            Ok(())
        })?,
    )?;

    engine_table.set(
        "log_warn",
        lua.create_function(|_, msg: String| {
            praxis_utils::warn!("[Script] {}", msg);
            Ok(())
        })?,
    )?;

    engine_table.set(
        "log_error",
        lua.create_function(|_, msg: String| {
            praxis_utils::error!("[Script] {}", msg);
            Ok(())
        })?,
    )?;

    lua.globals().set("engine", engine_table)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_engine_api() {
        let lua = Lua::new();
        let result = register_engine_api(&lua);
        assert!(result.is_ok());

        let has_engine: bool = lua.load("return engine ~= nil").eval().unwrap();
        assert!(has_engine);
    }
}
