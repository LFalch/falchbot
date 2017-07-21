#[macro_use]
extern crate serenity;
extern crate rand;
extern crate typemap;

use typemap::Key;

use serenity::client::Client;
use serenity::model;
use serenity::utils;
use std::env;

use rand::Rng;
use rand::thread_rng;

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

const CSGO_MSGS: [&str; 4] = [
    "Vi varmer op med en comp!",
    "Jeg er på!",
    "Jeg er mere til Call of Duty ..",
    "Mongoskrald!"
];

const BEARTOOTH: [&str; 3] = [
    "I’m not useless! I’m just the king of excuses!",
    "One life and one decision! Make sure it ends with you still living!",
    "Lorte skat på T-shirts."
];

const RESPONSES: [&str; 4] = [
    "Undskyld, kan ikke snakke lige nu :(",
    "Hey, kan jeg ringe igen senere?",
    "Hva' så din noob!? :P",
    "Ad, hvem er du?"
];

struct BotUser;

impl Key for BotUser {
    type Value = String;
}

fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token);

    client.with_framework(|f| f
        .configure(|c| c.prefix(PREFIX))
        .command("ping", |c| c.exec_str("Pong!"))
        .on("info", info)
        .on("setgame", setgame)
    );

    {
        let mut data = client.data.lock().unwrap();
        data.insert::<BotUser>(String::default());
    }

    client.on_ready(|ctx, ready| {
        println!("{} is connected!", ready.user.name);
        println!("Guilds:");
        for name in ready.guilds.iter().map(|g| g.id().get().unwrap().name) {
            println!("    {}", name);
        }
        {
            let mut data = ctx.data.lock().unwrap();
            *data.get_mut::<BotUser>().unwrap() = ready.user.tag();
        }
    });

    client.on_message(|ctx, msg| {
        if msg.author.bot {
            return
        }
        let s: String = msg.content.chars()
            .filter(|c| c.is_alphabetic())
            .flat_map(|c|c.to_lowercase())
            .collect();

        if s.contains("csgo") {
            let i = thread_rng().gen_range(0, CSGO_MSGS.len());
            msg.channel_id.say(CSGO_MSGS[i]).unwrap();
        }
        if s.contains("mc") {
            msg.channel_id.say("MINECRAFT!").unwrap();
        }
        if s.contains("beartooth") {
            let i = thread_rng().gen_range(0, BEARTOOTH.len());
            msg.channel_id.say(BEARTOOTH[i]).unwrap();
        }
        if s.contains("rep") {
            msg.channel_id.say("Rep mig!").unwrap();
        }
        if s.contains("meme") {
            msg.channel_id.say("krydrede migmig'er").unwrap();
        }
        if s.contains("rusland") || s.contains("russia") || s.contains("росси") ||
            s.contains("russisk") || s.contains("russian") || s.contains("русск") {
            msg.channel_id.say("Communism is the ultimate goal of socialism!").unwrap();
        }
        let user = {
            ctx.data.lock().unwrap().get::<BotUser>().unwrap().clone()
        };
        if msg.mentions.iter().map(|u| u.tag()).any(|u| u == user) {
            let i = thread_rng().gen_range(0, RESPONSES.len());
            msg.channel_id.say(RESPONSES[i]).unwrap();
        }
    });


    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}
