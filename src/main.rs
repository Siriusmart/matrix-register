use std::{collections::HashSet, env, fs, sync::OnceLock};

use matrix_sdk::ruma::api::client::{account::register, uiaa};
use serde::Deserialize;
use serenity::{
    Client,
    all::{
        ActionRowComponent, Colour, ComponentInteractionDataKind, Context, CreateActionRow,
        CreateButton, CreateCommand, CreateEmbed, CreateEmbedFooter, CreateInputText,
        CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateModal,
        EventHandler, GatewayIntents, GuildId, InputTextStyle, Interaction, Member, Ready,
    },
    async_trait,
};

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "discord-token")]
    discord_token: String,
    #[serde(rename = "homeserver-url")]
    homeserver_url: String,
    #[serde(rename = "homeserver-domain")]
    homeserver_domain: String,
    #[serde(rename = "registration-token")]
    registration_token: Option<String>,
    #[serde(rename = "guild-ids")]
    guilds_ids: HashSet<GuildId>,
    message: String,
}

struct Handler;

static CONFIG: OnceLock<Config> = OnceLock::new();

struct Success {
    pub username: String,
    pub homeserver: String,
    pub password: String,
}

async fn register(username: String, password: String) -> Result<Success, String> {
    let client = matrix_sdk::Client::builder()
        .homeserver_url(CONFIG.get().unwrap().homeserver_url.clone())
        .build()
        .await
        .unwrap();

    let mut request = register::v3::Request::new();
    if !password.is_empty() {
        request.password = Some(password.clone());
    }
    request.username = Some(username.clone());
    request.kind = register::RegistrationKind::User;
    request.auth = CONFIG
        .get()
        .unwrap()
        .registration_token
        .as_ref()
        .map(|token| {
            uiaa::AuthData::RegistrationToken(uiaa::RegistrationToken::new(token.clone()))
        });

    let _ = client
        .matrix_auth()
        .register(request)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Success {
        username,
        homeserver: CONFIG.get().unwrap().homeserver_domain.clone(),
        password,
    })
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        if !CONFIG
            .get()
            .unwrap()
            .guilds_ids
            .contains(&new_member.guild_id)
        {
            return;
        }

        let builder = CreateMessage::new()
            .embed(
                CreateEmbed::new()
                    .colour(Colour::from_rgb(0, 25, 255))
                    .title("You are invited to join our Matrix homeserver!")
                    .description(CONFIG.get().unwrap().message.clone()),
            )
            .button(CreateButton::new("accept_invite").label("Accept invite"));

        let _ = new_member.user.direct_message(&ctx.http, builder).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = &interaction {
            if command.data.name.as_str() != "matrix" {
                return;
            }

            let builder = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(
                        CreateEmbed::new()
                            .colour(Colour::from_rgb(0, 25, 255))
                            .title("You are invited to join our Matrix homeserver!")
                            .description(CONFIG.get().unwrap().message.clone()),
                    )
                    .button(CreateButton::new("accept_invite").label("Accept invite")),
            );
            if let Err(why) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {why}");
            }
        }

        if let Interaction::Component(interaction) = &interaction {
            if !matches!(interaction.data.kind, ComponentInteractionDataKind::Button) {
                return;
            }

            if interaction.data.custom_id.as_str() != "accept_invite" {
                return;
            }

            let builder = CreateInteractionResponse::Modal(
                CreateModal::new("register", "Create your account (ignore the warning pls)")
                    .components(vec![
                        CreateActionRow::InputText(CreateInputText::new(
                            InputTextStyle::Short,
                            "Choose your username, this cannot be changed",
                            "username",
                        )),
                        CreateActionRow::InputText(CreateInputText::new(
                            InputTextStyle::Short,
                            "Choose a secure password, this can be changed",
                            "password",
                        )),
                    ]),
            );

            if let Err(why) = interaction.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {why}");
            }
        }

        if let Interaction::Modal(modal) = interaction {
            if modal.data.custom_id.as_str() != "register" {
                return;
            }

            let _ = modal
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .embed(
                                CreateEmbed::new()
                                    .title("Processing your request")
                                    .colour(Colour::from_rgb(210, 252, 22))
                                    .description("Waiting for Matrix homeserver to respond..."),
                            )
                            .ephemeral(true),
                    ),
                )
                .await;

            let ActionRowComponent::InputText(username) = &modal.data.components[0].components[0]
            else {
                return;
            };

            let ActionRowComponent::InputText(password) = &modal.data.components[1].components[0]
            else {
                return;
            };

            let res = register(
                username.value.clone().unwrap(),
                password.value.clone().unwrap(),
            )
            .await;

            match res {
                Ok(Success {
                    username,
                    homeserver,
                    password,
                }) => {
                    let _ = modal.user.direct_message(
                        ctx,
                        CreateMessage::new().embed(
                            CreateEmbed::new()
                                .title("Account Created")
                                .colour(Colour::from_rgb(22, 252, 80))
                                .description(format!(
                                    "Homeserver: `{homeserver}`\nUsername: `{username}`\nPassword: ||`{password}`||",
                                )).field("What's next?", format!("Login to your account on [Cinny](https://app.cinny.in/login/{homeserver}), have fun!") , false),
                        ),
                    ).await;
                }
                Err(err) => {
                    let _ = modal
                        .user
                        .direct_message(
                            ctx,
                            CreateMessage::new().embed(
                                CreateEmbed::new()
                                    .colour(Colour::from_rgb(237, 18, 18))
                                    .title("Registration Failed")
                                    .field("Reason", err.to_string(), true)
                                    .footer(CreateEmbedFooter::new(
                                        "Contact an admin if this persists",
                                    )),
                            ),
                        )
                        .await;
                }
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        for guild_id in CONFIG.get().unwrap().guilds_ids.iter().copied() {
            if env::var("UNREGISTER_COMMANDS").is_ok() {
                guild_id.set_commands(&ctx.http, vec![]).await.unwrap();
            } else {
                guild_id
                    .set_commands(
                        &ctx.http,
                        vec![
                            CreateCommand::new("matrix")
                                .description("Sends the Matrix invite message."),
                        ],
                    )
                    .await
                    .unwrap();
            }
        }

        if env::var("UNREGISTER_COMMANDS").is_ok() {
            println!("Commands unregistered, please stop the bot")
        } else {
            println!("Commands registered")
        }
    }
}

#[tokio::main]
async fn main() {
    let config_path = env::var("CONFIG").expect("missing env CONFIG");
    let _ = CONFIG.set(toml::from_slice(&fs::read(config_path).unwrap()).unwrap());

    // Build our client.
    let mut client = Client::builder(
        CONFIG.get().unwrap().discord_token.clone(),
        GatewayIntents::all(),
    )
    .event_handler(Handler)
    .await
    .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
