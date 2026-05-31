use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption};

pub fn register_server_command() -> CreateCommand {
    CreateCommand::new("server")
        .description("manage your minecraft server")
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "start",
            "start your minecraft server",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "stop",
            "stop your minecraft server",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "properties",
            "view and edit properties of your minecraft server",
        ))
}
