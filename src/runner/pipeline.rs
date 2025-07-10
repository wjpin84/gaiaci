use std::fs;
use std::path::Path;
use mlua::{Lua, Table};
use crate::core::lua::register_gaia_lib;


pub fn load_pipeline(path: &Path) -> Result<(Lua, Table), Box<dyn std::error::Error>> {
    let lua: Lua = Lua::new();
    let code = fs::read_to_string(path)?;
    register_gaia_lib(&lua)?; // register the gaia lua library

    // Setup Lua paths for luarocks
    lua.load(r#"
        package.path = ".gaiaci/?.lua;.gaiaci/?/init.lua;.gaiaci/.luarocks/share/lua/5.4/?.lua;" .. package.path
        package.cpath = ".gaiaci/.luarocks/lib/lua/5.4/?.so;" .. package.cpath
    "#).exec()?;

     let pipeline: Table = lua.load(&code).eval()?;
    Ok((lua, pipeline))
}