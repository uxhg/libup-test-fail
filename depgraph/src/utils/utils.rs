use env_logger::Env;

pub fn init_log() {
    let env = Env::default()
        .filter_or("RUST_LOG", "warn")
        .write_style_or("LOG_STYLE", "always");
    env_logger::init_from_env(env);
}

