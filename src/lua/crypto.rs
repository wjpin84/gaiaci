use mlua::{Lua, Table, Result as LuaResult};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use base64::{engine::general_purpose, Engine as _};
use super::GaiaModule;

pub struct Crypto;

impl GaiaModule for Crypto {
    fn create(lua: &Lua) -> LuaResult<Table> {
        let crypto = lua.create_table()?;

        crypto.set("sha256", lua.create_function(sha256)?)?;
        crypto.set("uuid", lua.create_function(uuid)?)?;
        crypto.set("base64_encode", lua.create_function(base64_encode)?)?;
        crypto.set("base64_decode", lua.create_function(base64_decode)?)?;

        Ok(crypto)
    }
}

fn sha256(_: &Lua, input: String) -> LuaResult<String> {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

fn uuid(_: &Lua, _: ()) -> LuaResult<String> {
    Ok(Uuid::new_v4().to_string())
}

fn base64_encode(_: &Lua, input: String) -> LuaResult<String> {
    Ok(general_purpose::STANDARD.encode(input))
}

fn base64_decode(_: &Lua, input: String) -> LuaResult<String> {
    let decoded = general_purpose::STANDARD
        .decode(input)
        .map_err(|e| mlua::Error::RuntimeError(format!("Base64 decode error: {e}")))?;
    Ok(String::from_utf8_lossy(&decoded).to_string())
}
