use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

pub fn register_chat_command() -> CreateCommand {
    let message = CreateCommandOption::new(
        CommandOptionType::String,
        "message",
        "The message you're sending to minecraft chat",
    )
    .required(true);
    let command = CreateCommandOption::new(
        CommandOptionType::String,
        "command",
        "The command you're sending to chat",
    )
    .required(true);

    CreateCommand::new("chat")
        .description("manage chat to and from your minecraft server")
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "set",
            "set this channel as your minecraft chat channel (muting this channel in Discord is recommended)",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "say",
            "send a message in your minecraft chat",
        ).add_sub_option(message))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "command",
            "execute a command in your minecraft server",
        ).add_sub_option(command))
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "start", "start streaming chat from minecraft"))
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "stop", "stop streaming chat from your minecraft server"))
}
