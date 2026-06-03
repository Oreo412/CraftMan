use tracing_subscriber::{EnvFilter, fmt};

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    #[cfg(debug_assertions)]
    {
        let file_appender = tracing_appender::rolling::daily("logs", "craftman-server.log");

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // Keep the guard alive for the whole program.
        Box::leak(Box::new(guard));

        fmt()
            .with_ansi(false)
            .with_writer(non_blocking)
            .with_env_filter(env_filter)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        fmt().with_ansi(false).with_env_filter(env_filter).init();
    }
}
