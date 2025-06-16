//! Helix Ghost Text Plugin

mod data;
mod server;

/// Contains API loaded by the Helix config
mod api {
    use steel::{
        declare_module,
        steel_vm::ffi::{FFIModule, RegisterFFIFn as _},
    };

    use crate::server::{self, Server};

    declare_module!(ghost_text_steel_module);

    /// Declare the Steel module which will be dynamically loaded
    fn ghost_text_steel_module() -> FFIModule {
        let mut module = FFIModule::new("steel/ghost-text");

        module
            .register_fn("Server::new", Server::new)
            .register_fn("REGISTER_HELIX_BUFFER", server::register_helix_buffer)
            .register_fn("Server::init_logging", Server::init_logging)
            .register_fn("Server::start", Server::start)
            .register_fn("Server::stop", Server::stop)
            .register_fn("Server::update", Server::update);

        module
    }
}
