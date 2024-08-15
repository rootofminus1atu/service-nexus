use super::{Context, Error};




#[poise::command(slash_command)]
pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello, world!").await?;
    Ok(())
}
