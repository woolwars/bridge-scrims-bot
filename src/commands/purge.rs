use std::collections::HashMap;

use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::futures::StreamExt;
use serenity::model::channel::Message;

use serenity::model::interactions::application_command::{
    ApplicationCommandInteraction, ApplicationCommandOptionType, ApplicationCommandPermissionType,
};
use serenity::model::interactions::InteractionResponseType;
use serenity::model::prelude::InteractionApplicationCommandCallbackDataFlags;

use bridge_scrims::interact_opts::InteractOpts;

use crate::commands::Command;
use crate::consts::CONFIG;

pub enum PurgeOption {
    All,
    FromUser,
    Embeds,
    Images,
    Attachments,
    Contains,
    Bots,
    Links,
}

impl PurgeOption {
    fn name(&self) -> String {
        match self {
            PurgeOption::All => "all",
            PurgeOption::FromUser => "from_user",
            PurgeOption::Embeds => "embeds",
            PurgeOption::Images => "images",
            PurgeOption::Attachments => "attachments",
            PurgeOption::Contains => "contains",
            PurgeOption::Bots => "bots",
            PurgeOption::Links => "links",
        }
        .to_string()
    }
    fn register_options(&self, cmd: &mut CreateApplicationCommand) {
        // Add more sub options if neccessary
        match self {
            PurgeOption::FromUser => {
                cmd.create_option(|user| {
                    user.name("user")
                        .kind(ApplicationCommandOptionType::User)
                        .description("The user who's messages are to be purged (if the from_user option is selected)")
                        .required(false)
                });
            }
            PurgeOption::Contains => {
                cmd.create_option(|user| {
                    user.name("text")
                        .kind(ApplicationCommandOptionType::String)
                        .description("The text to search for in purging messages (if the contains option is selected)")
                        .required(false)
                });
            }
            _ => {}
        }
    }

    async fn check(&self, cmd: &ApplicationCommandInteraction, msg: Message) -> bool {
        match self {
            PurgeOption::All => true,
            PurgeOption::FromUser => {
                let x: u64 = cmd
                    .get_str("user")
                    .unwrap_or_else(|| "0".to_string())
                    .parse()
                    .unwrap();
                msg.author.id.0 == x
            }
            PurgeOption::Embeds => !msg.embeds.is_empty(),
            PurgeOption::Images => {
                !msg.attachments.is_empty() && msg.attachments[0].height.is_some()
                // Height is some if its an imag
            }
            PurgeOption::Attachments => !msg.attachments.is_empty(),
            PurgeOption::Contains => {
                let x = cmd.get_str("text").unwrap_or_default();
                msg.content
                    .to_ascii_lowercase()
                    .contains(&x.to_ascii_lowercase())
            }
            PurgeOption::Bots => msg.author.bot,
            PurgeOption::Links => {
                msg.content.to_ascii_lowercase().contains("https://")
                    || msg.content.to_ascii_lowercase().contains("http://")
            }
        }
    }
}

pub struct Purge {
    options: HashMap<String, PurgeOption>,
}

#[async_trait]
impl Command for Purge {
    fn name(&self) -> String {
        "purge".to_string()
    }

    async fn register(&self, ctx: &Context) -> crate::Result<()> {
        let cmd = CONFIG
            .guild
            .create_application_command(&ctx, |c| {
                c.name(self.name())
                    .description("Purges a specific amount of messages from the channel")
                    .default_permission(false);
                c.create_option(|opt| {
                    opt.name("filter")
                        .kind(ApplicationCommandOptionType::String)
                        .description("The specific type of messages to purge.")
                });
                c.create_option(|amount| {
                    amount
                        .name("amount")
                        .kind(ApplicationCommandOptionType::Integer)
                        .description(
                            "The amount of messages to go through to purge (total messages)",
                        )
                        .required(true)
                });
                for option in self.options.values() {
                    option.register_options(c);
                }
                c
            })
            .await?;
        CONFIG
            .guild
            .create_application_command_permission(&ctx, cmd.id, |p| {
                for role in &[CONFIG.support, CONFIG.trial_support, CONFIG.staff] {
                    p.create_permission(|perm| {
                        perm.kind(ApplicationCommandPermissionType::Role)
                            .id(role.0)
                            .permission(true)
                    });
                }
                p
            })
            .await?;
        Ok(())
    }

    async fn run(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> crate::Result<()> {
        command
            .create_interaction_response(&ctx, |r| {
                r.interaction_response_data(|d| {
                    d.flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
                .kind(InteractionResponseType::DeferredChannelMessageWithSource)
            })
            .await?;

        let channel = command.channel_id;
        let mut messages = channel.messages_iter(&ctx.http).boxed();
        let filter = command.get_str("filter").unwrap();
        let max_purge = command.get_i64("amount").unwrap_or(50);

        let option = self.options.get(&filter).unwrap();
        let mut i = 0;
        while let Some(Ok(message)) = messages.next().await {
            i += 1;

            if i > max_purge {
                break;
            }

            if option.check(command, message.clone()).await {
                // ignore errors here since it doesn't matter if we can't delete
                let _ = message.delete(&ctx.http).await;
            }
        }
        command
            .edit_original_interaction_response(&ctx.http, |r| {
                r.content("Purge Successful!".to_string())
            })
            .await?;
        Ok(())
    }

    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        let options: Vec<PurgeOption> = vec![
            PurgeOption::All,
            PurgeOption::FromUser,
            PurgeOption::Embeds,
            PurgeOption::Images,
            PurgeOption::Attachments,
            PurgeOption::Contains,
            PurgeOption::Bots,
            PurgeOption::Links,
        ];
        let options = options.into_iter().fold(HashMap::new(), |mut map, opt| {
            map.insert(opt.name(), opt);
            map
        });

        Box::new(Purge { options })
    }
}
