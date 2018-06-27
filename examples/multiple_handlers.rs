// This is not a hard dependency.
// Just used for loading the username, password and homeserverurl from a file.
extern crate config;
// Just used for rolling dice
extern crate rand;

extern crate matrix_bot_api;
use matrix_bot_api::{MatrixBot, MessageType};
use matrix_bot_api::handlers::{MessageHandler, StatelessHandler, extract_command, HandleResult};


fn main() {
    // ------- Getting the login-credentials from file -------
    // You can get them however you like: hard-code them here, env-variable,
    // tcp-connection, read from file, etc. Here, we use the config-crate to
    // load from botconfig.toml.
    // Change this file to your needs, if you want to use this example binary.
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("examples/botconfig")).unwrap();

    let user = settings.get_str("user").unwrap();
    let password  = settings.get_str("password").unwrap();
    let homeserver_url = settings.get_str("homeserver_url").unwrap();
    // -------------------------------------------------------

    // Here we want a handler with state (simple counter-variable).
    // So we had to implement our own MessageHandler.
    let counter = CounterHandler::new();

    // Give the first handler to your new bot (bot needs at least one handler)
    let mut bot = MatrixBot::new(counter);

    // Create another handler, and add it
    let mut who = StatelessHandler::new();
    who.register_handle("whoareyou", Box::new(whoareyou));

    bot.add_handler(who);

    let mut roll = StatelessHandler::new();
    roll.register_handle("roll", Box::new(roll_dice));
    roll.register_handle("help", Box::new(roll_help));

    bot.add_handler(roll);

    let mut shutdown = StatelessHandler::new();
    // Handlers can have different prefixes of course
    shutdown.set_cmd_prefix("BOT: ");

    shutdown.register_handle("leave", |bot: &MatrixBot, room: &str, _cmd: &str| {
        bot.send_message("Bye!", room, MessageType::RoomNotice);
        bot.leave_room(room);
        HandleResult::StopHandling
    });

    shutdown.register_handle("shutdown", |bot: &MatrixBot, _room: &str, _cmd: &str| {
        bot.shutdown();
        HandleResult::StopHandling
    });

    bot.add_handler(shutdown);

    // Blocking call (until shutdown). Handles all incoming messages and calls the associated functions.
    // The bot will automatically join room it is invited to.
    bot.run(&user, &password, &homeserver_url);
}


// We can register multiple handlers. Thus, we create some here.

// --------- Definition of 1. handler -----------
// Just copied from with_state.rs:
pub struct CounterHandler {
    counter: i32,
}

impl CounterHandler {
    fn new() -> CounterHandler {
        CounterHandler{counter: 0}
    }

    fn show_help(&mut self, bot: &MatrixBot, room: &str, _cmd: &str) -> HandleResult {
        let mut message = "Counter:\n".to_string();
        message += "!incr = Increases counter by one\n";
        message += "!decr = Decreases counter by one\n";
        message += "!show = Show current value of counter\n";
        bot.send_message(&message, room, MessageType::RoomNotice);
        HandleResult::ContinueHandling /* There might be more handlers that implement "help" */
    }
}

impl MessageHandler for CounterHandler {
    fn handle_message(&mut self, bot: &MatrixBot, room: &str, message: &str) -> HandleResult {
        let command = match extract_command(message, "!") {
            Some(x) => x,
            None => return HandleResult::ContinueHandling,
        };

        match command {
          "incr" => self.counter += 1,
          "decr" => self.counter -= 1,
          "show" => bot.send_message(&format!("Counter = {}", self.counter), room, MessageType::RoomNotice),
          "help" => return self.show_help(bot, room, message),
          _ => return HandleResult::ContinueHandling /* Not a known command */
        }
        HandleResult::StopHandling
    }
}

// --------- Definition for 2. handler -----------
// Copied from stateless.rs
fn whoareyou(bot: &MatrixBot, room: &str, _cmd: &str) -> HandleResult {
    bot.send_message("I'm a bot.", room, MessageType::RoomNotice);
    HandleResult::StopHandling
}

// --------- Definition for 3. handler -----------
fn roll_help(bot: &MatrixBot, room: &str, _cmd: &str) -> HandleResult {
    let mut message = "Roll dice:\n".to_string();
    message += "!roll X [X ..]\n";
    message += "with\n";
    message += "X = some number. Thats the number of eyes your die will have.\n";
    message += "If multpile numbers are given, multiple dice are rolled. The result as a sum is displayed as well.\n";
    message += "\nExample: !roll 6 12 => Rolls 2 dice, one with 6, the other with 12 eyes.\n";
    bot.send_message(&message, room, MessageType::RoomNotice);
    HandleResult::ContinueHandling /* There might be more handlers that implement "help" */
}

fn roll_dice(bot: &MatrixBot, room: &str, cmd: &str) -> HandleResult {
    let cmd_split = cmd.split_whitespace();

    let mut results: Vec<u32> = vec![];
    for dice in cmd_split {
        let sides = match dice.parse::<u32>() {
            Ok(x) => x,
            Err(_) => { bot.send_message(&format!("{} is not a number.", dice), room, MessageType::RoomNotice);
                        return HandleResult::StopHandling; }
        };
        results.push((rand::random::<u32>() % sides) + 1);
    }

    if results.len() == 0 {
        return roll_help(bot, room, cmd);
    }

    if results.len() == 1 {
        bot.send_message(&format!("{}", results[0]), room, MessageType::RoomNotice);
    } else {
       // make string from results:
       let str_res : Vec<String> = results.iter().map(|x| x.to_string()).collect();
       bot.send_message(&format!("{} = {}", str_res.join(" + "), results.iter().sum::<u32>()), room, MessageType::RoomNotice);
    }

    HandleResult::StopHandling
}

