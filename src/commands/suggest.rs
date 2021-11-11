use serenity::async_trait;
use serenity::client::Context;
use serenity::model::id::ChannelId;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::interactions::application_command::{
    ApplicationCommandInteraction, ApplicationCommandOptionType,
};
use serenity::model::interactions::InteractionResponseType;
use serenity::model::prelude::InteractionApplicationCommandCallbackDataFlags;
use serenity::utils::Color;

use crate::commands::Command;

// Just put any channel ID here or a 0, I'll put the correct values in when I compile it for the server.
const SUGGESTIONS_CHANNEL: ChannelId = ChannelId(0);

pub struct Suggestion;

#[async_trait]
#[allow(unused_must_use)]
impl Command for Suggestion {
    fn name(&self) -> String {
        "suggestion".to_string()
    }

    async fn register(&self, ctx: &Context) -> crate::Result<()> {
        crate::GUILD
            .create_application_command(&ctx, |c| {
                c.name(self.name())
                    .description("Create a suggestion")
                    .create_option(|o| {
                        o.name("suggestion")
                            .description("Put your suggestion here")
                            .required(true)
                            .kind(ApplicationCommandOptionType::String)
                    })
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

        // Embed

        command
            .edit_original_interaction_response(&ctx, |r| {
                r.create_embed(|e| {
                    e.title("Suggestions")
                        .description("Your suggestion has been delivered successfully".to_string())
                        .color(Color::new(0x74a8ee))
                })
            })
            .await?;
        let x = &command.user;
        let s = match &command.data.options[0].resolved {
            Some(ApplicationCommandInteractionDataOptionValue::String(s)) => s.clone(),
            _ => panic!("expected a string value"),
        };
        //        SUGGESTIONS_CHANNEL.say(ctx, format!("received suggestion: {}", s)).await?;
        SUGGESTIONS_CHANNEL
            .send_message(&ctx.http, |m| {
                m.content("_ _");
                m.embed(|e| {
                    e.title("Recieved a new suggestion! from: ");
                    e.description(x.to_string());
                    e.field("Suggestion:", s.to_string(), false);
                    e.color(Color::new(0x74a8ee));
                    e.footer(|f| {
                        f.text("Bridge Scrims");
                        f
                    })
                })
            })
            .await;

        Ok(())
    }

    fn new() -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(Suggestion)
    }
}
