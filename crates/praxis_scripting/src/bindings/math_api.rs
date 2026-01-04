//! Math API bindings for Lua scripts.

use mlua::Lua;
use praxis_utils::Result;

/// Registers math types and functions with the Lua environment.
pub fn register_math_api(lua: &Lua) -> Result<()> {
    let math_table = lua.create_table()?;
    
    // Vec3 constructor
    math_table.set("Vec3", lua.create_function(|lua, (x, y, z): (f32, f32, f32)| {
        let table = lua.create_table()?;
        table.set("x", x)?;
        table.set("y", y)?;
        table.set("z", z)?;
        Ok(table)
    })?)?;
    
    // Quat constructor
    math_table.set("Quat", lua.create_function(|lua, (x, y, z, w): (f32, f32, f32, f32)| {
        let table = lua.create_table()?;
        table.set("x", x)?;
        table.set("y", y)?;
        table.set("z", z)?;
        table.set("w", w)?;
        Ok(table)
    })?)?;
    
    // Constants
    math_table.set("pi", std::f32::consts::PI)?;
    math_table.set("tau", std::f32::consts::TAU)?;
    
    // Helper functions
    math_table.set("sqrt", lua.create_function(|_, x: f32| Ok(x.sqrt()))?)?;
    math_table.set("sin", lua.create_function(|_, x: f32| Ok(x.sin()))?)?;
    math_table.set("cos", lua.create_function(|_, x: f32| Ok(x.cos()))?)?;
    math_table.set("tan", lua.create_function(|_, x: f32| Ok(x.tan()))?)?;
    math_table.set("abs", lua.create_function(|_, x: f32| Ok(x.abs()))?)?;
    
    lua.globals().set("math", math_table)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_math_api() {
        let lua = Lua::new();
        let result = register_math_api(&lua);
        assert!(result.is_ok());
        
        let has_math: bool = lua.load("return math ~= nil").eval().unwrap();
        assert!(has_math);
    }
    
    #[test]
    fn test_vec3_constructor() {
        let lua = Lua::new();
        register_math_api(&lua).unwrap();
        
        let result: f32 = lua.load(r#"
            local v = math.Vec3(1, 2, 3)
            return v.x + v.y + v.z
        "#).eval().unwrap();
        
        assert_eq!(result, 6.0);
    }
}
