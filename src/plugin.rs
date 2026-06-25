//! Lua plugin system via mlua.
//! Plugins live in ~/.config/tde/plugins/*.lua
use anyhow::Result;
use mlua::prelude::*;
use std::path::PathBuf;

fn lua_result<T>(result: mlua::Result<T>) -> Result<T> {
    result.map_err(|e| anyhow::anyhow!(e.to_string()))
}

pub struct PluginManager {
    lua: Lua,
}

impl PluginManager {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();
        Ok(Self { lua })
    }

    /// Register the TDE Lua API so plugins can call tde.notify(), etc.
    pub fn register_api(&self, notif_tx: std::sync::mpsc::Sender<String>) -> Result<()> {
        let tde = lua_result(self.lua.create_table())?;

        // tde.notify("message")
        let tx = notif_tx.clone();
        let notify_fn = lua_result(self.lua.create_function(move |_, msg: String| {
            tx.send(msg).ok();
            Ok(())
        }))?;
        lua_result(tde.set("notify", notify_fn))?;

        // tde.version
        lua_result(tde.set("version", "0.1.1"))?;

        lua_result(self.lua.globals().set("tde", tde))?;
        Ok(())
    }

    /// Load and execute all plugins from ~/.config/tde/plugins/
    pub fn load_all(&self) -> Result<()> {
        let dir = plugin_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
            return Ok(());
        }
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                let code = std::fs::read_to_string(&path)?;
                if let Err(e) = self.lua.load(&code).exec() {
                    eprintln!("Plugin error {:?}: {e}", path.file_name().unwrap());
                }
            }
        }
        Ok(())
    }

    /// Call an event hook in all plugins, e.g. on_startup, on_pane_focus.
    pub fn call_hook(&self, hook: &str) -> Result<()> {
        if let Ok(f) = self.lua.globals().get::<LuaFunction>(hook) {
            lua_result(f.call::<()>(()))?;
        }
        Ok(())
    }
}

pub fn plugin_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tde")
        .join("plugins")
}