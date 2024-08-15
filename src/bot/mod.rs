use poise::serenity_prelude::{self as serenity, ClientBuilder};
use shuttle_runtime::SecretStore;
use shuttle_runtime::__internals::Context as ShuttleContext;

mod commands;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn setup_discord_bot(secret_store: &SecretStore) -> Result<serenity::Client, shuttle_runtime::Error> {
    let discord_token = secret_store
        .get("DISCORD_TOKEN")
        .context("discord token not found")?;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![self::commands::hello()],
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                mention_as_prefix: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .build();

    let client = ClientBuilder::new(discord_token, serenity::GatewayIntents::non_privileged() 
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_PRESENCES)
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client)
}