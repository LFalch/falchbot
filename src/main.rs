#[macro_use]
extern crate serenity;

use serenity::client::Client;
use serenity::model;
use serenity::utils;
use std::env;

const PREFIX: &str = "f>";

command!(info(_ctx, msg, _args) {
    msg.channel_id.send_message(|cm| {
        cm.embed(|e| {
            e.title("lfalchbot")
             .colour(utils::Colour::blue())
             .description("(c) LFalch.com 2017")
             .footer(|f| f.text(serenity::constants::USER_AGENT))
        })
    }).unwrap();
});

command!(setgame(ctx, _msg, args) {
    ctx.set_game(model::Game::playing(&args.join(" ")));
});

fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token);

    client.with_framework(|f| f
        .configure(|c| c.prefix(PREFIX))
        .command("ping", |c| c.exec_str("Pong!"))
        .on("info", info)
        .on("setgame", setgame)
    );

    client.on_ready(|_context, ready| {
        println!("{} is connected!", ready.user.name);
        println!("Guilds:");
        for name in ready.guilds.iter().map(|g| g.id().get().unwrap().name) {
            println!("    {}", name);
        }
    });

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
