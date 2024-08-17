use axum::Router;
use bot::setup_discord_bot;
use shuttle_runtime::SecretStore;
use axum::ServiceExt;
use web::setup_web_server;

pub mod web;
pub mod bot;
pub mod helpers;

pub struct CustomService {
    // discord_bot: serenity::Client,
    router: Router,
}


#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CustomService {
    async fn bind(
        mut self,
        addr: std::net::SocketAddr,
    ) -> Result<(), shuttle_runtime::Error> {
        let router = self.router.into_service();

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        
        let serve_router = async move {
            axum::serve(listener, router.into_make_service())
                .await
                .unwrap();
        };

        // let discord_bot_future = async {
        //     self.discord_bot.start().await.unwrap();
        // };

        tokio::select! {
            // _ = discord_bot_future => {},
            _ = serve_router => {}
        };

        Ok(())
    }
}




#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> Result<CustomService, shuttle_runtime::Error> {
    let _discord_bot = setup_discord_bot(&secret_store).await?;
    let router = setup_web_server(&secret_store).await?;

    Ok(CustomService {
        // discord_bot: discord_bot,
        router,
    })
}