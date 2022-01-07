use std::sync::Arc;
use twilight_model::application::interaction::application_command::{CommandData, CommandDataOption, CommandOptionValue};
use actix_web::HttpResponse;
use chrono::Utc;
use tracing::error;
use twilight_model::application::callback::InteractionResponse;
use crate::util::CommandError;
use twilight_model::application::interaction::ApplicationCommand;
use twilight_util::builder::CallbackDataBuilder;
use twilight_model::channel::message::MessageFlags;
use crate::State;

mod debug;
mod ping;

pub struct Reply {
    pub response: InteractionResponse,
    pub followup: bool,
}

pub type CommandResult = Result<Reply, CommandError>;

pub enum Commands {
    PING,
    DEBUG,
}

impl Commands {
    pub fn parse(data: &CommandData) -> Option<Self> {
        match data.name.as_str() {
            "ping" => Some(Self::PING),
            "debug" => Some(Self::DEBUG),
            _ => None
        }
    }

    fn has_subcommands(&self) -> bool {
        match self {
            _ => false
        }
    }

    fn parse_into_subcommand(&self, data: &CommandDataOption) -> Option<Commands> {
        unreachable!()
    }

    fn execute(&self, command: &Box<ApplicationCommand>, options: &Vec<CommandDataOption>, state: &Arc<State>) -> CommandResult {
        match self {
            Commands::PING => defer_async(false),
            Commands::DEBUG => defer_async(false)
        }
    }

    fn get_name(&self) -> &str {
        match self {
            Commands::PING => "ping",
            Commands::DEBUG => "debug"
        }
    }

    async fn async_followup(self, command: Box<ApplicationCommand>, options: Vec<CommandDataOption>, state: &Arc<State>) -> Result<(), CommandError> {
        match self {
            Commands::PING => ping::async_followup(command, state).await?,
            Commands::DEBUG => debug::async_followup(command, state).await?
        };
        Ok(())
    }
}

pub async fn handle_command(interaction: Box<ApplicationCommand>, state: Arc<State>) -> HttpResponse {
    // map to a command if possible, though in reality we should always have one
    // i just don't like crashes
    if let Some(command) = Commands::parse(&interaction.data) {
        let mut to_execute = command;
        let mut options = interaction.data.options.clone();
        // recurse down in subcommands
        while to_execute.has_subcommands() {
            if let Some(local_option) = options.pop() {
                if let Some(subcommand) = to_execute.parse_into_subcommand(&local_option) {
                    to_execute = subcommand;
                    // get the inner options, should be a correct one but guarded just in case
                    options = match local_option.value {
                        CommandOptionValue::SubCommand(options) => options,
                        CommandOptionValue::SubCommandGroup(options) => options,
                        _ => {
                            // if we get here some command is claiming it has subcommands when it really doesn't
                            error!("Command '{}' claimed to have subcommands but the first option was not a subcommand or subcommandgroup option!", to_execute.get_name());
                            return HttpResponse::BadRequest().body("");
                        }
                    };
                } else {
                    error!("Command '{}' claimed to have subcommands, but failed to parse into an actual subcommand to execute ({:?})", to_execute.get_name(), &local_option)
                }
            } else {
                error!("Command '{}' claimed to have subcommands but no options where received from discord!", to_execute.get_name());
                return HttpResponse::BadRequest().body("");
            }
        }

        let name = to_execute.get_name().to_string();

        let start = Utc::now();
        let result = to_execute.execute(&interaction, &options, &state);

        let duration = Utc::now() - start;
        let observation = (duration.num_microseconds().unwrap_or(i64::MAX) as f64) / 1_000_000f64;


        let (response, followup, status) = match result {
            Ok(reply) => {
                (reply.response, reply.followup, "COMPLETED")
            }
            Err(error) => {
                if !error.is_user_error() {
                    error!("Failed to handle interaction: {} (interaction data: {:?})", error.get_log_error(), &interaction);
                }
                (
                    InteractionResponse::ChannelMessageWithSource(
                        CallbackDataBuilder::new()
                            .content("Something went wrong!".to_string())
                            .flags(MessageFlags::EPHEMERAL)
                            .build()
                    ),
                    false,
                    if error.is_user_error() { "USER_ERROR" } else { "SYSTEM_ERROR" }
                )
            }
        };

        state.metrics.command_durations.get_metric_with_label_values(&[&name, status]).unwrap().observe(observation);
        state.metrics.command_totals.get_metric_with_label_values(&[&name, status]).unwrap().inc();


        if followup {
            let token = interaction.token.clone();
            actix_rt::spawn(async move {
                let start = Utc::now();
                let result = to_execute.async_followup(interaction, options, &state).await;

                let duration = Utc::now() - start;
                let observation = (duration.num_microseconds().unwrap_or(i64::MAX) as f64) / 1_000_000f64;
                let status = match result {
                    Ok(_) => "COMPLETED",
                    Err(e) => {
                        let metric_type = if e.is_user_error() {
                            "USER_ERROR"
                        } else {
                            error!("Error during followup for command '{}': {}", name, e.get_log_error());
                            "SYSTEM_ERROR"
                        };

                        // send an error followup to the requester
                        if let Err(e) = state.discord_client.create_followup_message(&token).unwrap()
                            //TODO: replace with actual lang once available
                            .content(&e.get_user_error(&state.translator, "en_us"))
                            .ephemeral(true)
                            .exec()
                            .await {
                            error!("Failed to inform that something went wrong: {}", e)
                        }

                        metric_type
                    }
                };

                state.metrics.command_followups_duration.get_metric_with_label_values(&[&name, status]).unwrap().observe(observation);
                state.metrics.command_followups_total.get_metric_with_label_values(&[&name, status]).unwrap().inc();
            });
        }

        let body = serde_json::to_string(&response).expect("InteractionResponse can't be converted to json anymore!");
        HttpResponse::Ok().body(body)
    } else {
        error!("Received a command to execute from discord that can't be mapped to an internal command handler! {}", interaction.data.name);
        HttpResponse::BadRequest().body("")
    }
}

fn defer_async(ephemeral: bool) -> CommandResult {
    let flags = if ephemeral {
        MessageFlags::EPHEMERAL
    } else {
        MessageFlags::empty()
    };
    Ok(
        Reply {
            response:
            InteractionResponse::DeferredChannelMessageWithSource(
                CallbackDataBuilder::new().flags(flags).build()
            ),
            followup: true,
        }
    )
}

pub fn get_required_string_value<'a>(name: &str, options: &'a Vec<CommandDataOption>) -> Result<&'a str, CommandError> {
    get_optional_string_value(name, options).ok_or_else(|| CommandError::MissingOption(name.to_string()))
}

pub fn get_optional_string_value<'a>(name: &str, options: &'a Vec<CommandDataOption>) -> Option<&'a str> {
    for option in options {
        if option.name == name {
            return match &option.value {
                CommandOptionValue::String(value) => {
                    Some(value)
                }
                _ => None
            }
        }
    }

    None
}