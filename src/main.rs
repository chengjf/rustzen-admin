use rustzen_admin::core::app::create_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load env
    dotenvy::dotenv().ok();

    // init log — level controlled by RUST_LOG env var (default: info)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .init();


    // create server
    create_server().await?;

    Ok(())
}
