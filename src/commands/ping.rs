use serenity::{
    async_trait,
    model::{
        id::RoleId,
        prelude::{
            application_command::{
                ApplicationCommandInteraction, ApplicationCommandOptionType,
                ApplicationCommandPermissionType,
            },
            InteractionApplicationCommandCallbackDataFlags,
        },
    },
    prelude::Context,
};
use std::time::Duration;

use crate::{commands::Command, consts::CONFIG};
use bridge_scrims::cooldown::Cooldowns;
use bridge_scrims::interact_opts::InteractOpts;

pub struct Ping {
    cooldowns: Cooldowns,
}

#[async_trait]
impl Command for Ping {
    fn name(&self) -> String {
        "ping".to_string()
    }
    async fn register(&self, ctx: &Context) -> crate::Result<()> {
        for option in &CONFIG.pings {
            let cmd = CONFIG
                .guild
                .create_application_command(&ctx.http, |cmd| {
                    cmd.name(option.name.clone())
                        .description("Ping a desired role uppon request")
                        .default_permission(false)
                        .create_option(|m| {
                            m.name("role")
                                .kind(ApplicationCommandOptionType::String)
                                .description("The role you would like to mention")
                                .required(true);
                            for (name, role) in &option.options {
                                m.add_string_choice(name, role.0);
                            }
                            m
                        })
                        .create_option(|x| {
                            x.name("text")
                                .kind(ApplicationCommandOptionType::String)
                                .required(false)
                                .description("An optional additional text to put in the message")
                        });
                    cmd
                })
                .await?;
            CONFIG
                .guild
                .create_application_command_permission(&ctx.http, cmd.id, |perms| {
                    for r in &option.required_roles {
                        perms.create_permission(|p| {
                            p.kind(ApplicationCommandPermissionType::Role)
                                .id(r.0)
                                .permission(true)
                        });
                    }
                    perms.create_permission(|p| {
                        p.kind(ApplicationCommandPermissionType::Role)
                            .id(CONFIG.staff.0)
                            .permission(true)
                    })
                })
                .await?;
        }
        Ok(())
    }
    fn is_command(&self, name: String) -> bool {
        CONFIG.pings.iter().any(|opt| opt.name == name)
    }
    async fn run(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> crate::Result<()> {
        let role = RoleId(command.get_str("role").unwrap().parse().unwrap());
        if let Some(opt) = CONFIG
            .pings
            .iter()
            .find(|opt| opt.name == command.data.name)
        {
            if let Some(channels) = &opt.allowed_channels {
                let cat = command
                    .channel_id
                    .to_channel(&ctx)
                    .await?
                    .guild()
                    .unwrap()
                    .category_id;
                if !channels
                    .iter()
                    .any(|c| c == &command.channel_id || Some(*c) == cat)
                {
                    command
                        .create_interaction_response(&ctx.http, |r| {
                            r.interaction_response_data(|d| {
                                d.content("This command is disabled in this channel.")
                                    .flags(
                                        InteractionApplicationCommandCallbackDataFlags::EPHEMERAL,
                                    )
                            })
                        })
                        .await?;
                    return Ok(());
                }
            }
        }
        let cid = format!("{}", role.0);
        let cooldown = self
            .cooldowns
            .check_cooldown_key(command.user.id, cid.clone())
            .await;
        if let Some(t) = cooldown {
            command
                .create_interaction_response(&ctx.http, |r| {
                    r.interaction_response_data(|d| {
                        d.content(format!(
                            "You are on a cooldown. Please wait {:.2} seconds.",
                            t.as_secs_f32()
                        ))
                        .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                    })
                })
                .await?;
            return Ok(());
        }
        self.cooldowns
            .add_global_cooldown_key(cid.clone(), Duration::from_secs(60))
            .await;
        self.cooldowns
            .add_user_cooldown_key(cid.clone(), Duration::from_secs(60 * 5), command.user.id)
            .await;
        let text = command.get_str("text").unwrap_or_else(|| "".to_string());
        command
            .channel_id
            .send_message(&ctx.http, |r| {
                r.content(format!("<@!{}>: <@&{}> {}", command.user.id, role.0, text))
                    .allowed_mentions(|m| m.roles(vec![role]))
            })
            .await?;
        command
            .create_interaction_response(&ctx.http, |r| {
                r.interaction_response_data(|d| {
                    d.content("Ping sent!")
                        .flags(InteractionApplicationCommandCallbackDataFlags::EPHEMERAL)
                })
            })
            .await?;
        Ok(())
    }
    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(Ping {
            cooldowns: Cooldowns::new(),
        })
    }
}
