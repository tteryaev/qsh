use mlua::{Function, Lua, RegistryKey};

pub struct Events {
    pub on_start: Vec<RegistryKey>,

    pub before_command: Vec<RegistryKey>,

    pub after_command: Vec<RegistryKey>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            on_start: Vec::new(),

            before_command: Vec::new(),

            after_command: Vec::new(),
        }
    }

    pub fn run_on_start(&self, lua: &Lua) {
        for key in &self.on_start {
            let callback = match lua.registry_value::<Function>(key) {
                Ok(callback) => callback,

                Err(error) => {
                    eprintln!("plugin error: {}", error);

                    continue;
                }
            };

            if let Err(error) = callback.call::<()>(()) {
                eprintln!("plugin error: {}", error);
            }
        }
    }

    pub fn run_before_command(&self, lua: &Lua, command: String) -> bool {
        for key in &self.before_command {
            let callback = match lua.registry_value::<Function>(key) {
                Ok(callback) => callback,

                Err(error) => {
                    eprintln!("plugin error: {}", error);

                    continue;
                }
            };

            match callback.call::<bool>(command.clone()) {
                Ok(result) => {
                    if !result {
                        return false;
                    }
                }

                Err(error) => {
                    eprintln!("plugin error: {}", error);
                }
            }
        }

        true
    }

    pub fn run_after_command(&self, lua: &Lua, command: String, success: bool) {
        for key in &self.after_command {
            if let Ok(callback) = lua.registry_value::<Function>(key) {
                let _ = callback.call::<()>((command.clone(), success));
            }
        }
    }
}
