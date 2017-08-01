#[macro_use]
extern crate serenity;
extern crate rand;
extern crate typemap;

use std::env;
use std::result::Result as StdResult;
use typemap::Key;

use rand::Rng;
use rand::thread_rng;

use serenity::Result;
use serenity::client::{Client, Context};
use serenity::model;
use serenity::utils;

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

command!(rpn(_ctx, msg, args) {
    match calculate(&args) {
        Ok(r) => msg.reply(&format!("Result: {}", r)),
        Err(e) => msg.reply(&format!("Error: {:?}", e))
    }.unwrap();
});

const CSGO_MSGS: [&str; 6] = [
    "Vi varmer op med en comp!",
    "Jeg er på!",
    "Jeg er mere til Call of Duty ..",
    "Сука блядь!",
    "-skyder en bot for at få dens Bizon-",
    "Mongoskrald!"
];

const BEARTOOTH: [&str; 3] = [
    "I’m not useless! I’m just the king of excuses!",
    "One life and one decision! Make sure it ends with you still living!",
    "Lorteskat på T-shirts."
];

const RESPONSES: [&str; 4] = [
    "Undskyld, kan ikke snakke lige nu :(",
    "Hey, kan jeg ringe igen senere?",
    "Hva' så, din noob!? :P",
    "Ad, hvem er du?"
];

const REDALERT: [&str; 4] = [
    "Your base is under a salt!",
    "Det gamle lortespil?",
    "Jeg er mere til CSGO.",
    "\"Mine depleted\" ..."
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
        .on("rpn", rpn)
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

    client.on_message(on_message);

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

fn send_random(chl: model::ChannelId, list: &[&str]) -> Result<model::Message> {
    let i = thread_rng().gen_range(0, list.len());
    chl.say(list[i])
}

macro_rules! joke {
    ($s:expr; $($trigger:expr),+; $bl:block) => (
        if $($s.contains($trigger))||* $bl
    );
    ($s:expr, $channel_id:expr; $($trigger:expr),+;; $joke:expr) => (
        joke!($s; $($trigger),*; {
            $channel_id.say($joke).unwrap();
        })
    );
    ($s:expr, $channel_id:expr; $($trigger:expr),+; $jokes:expr) => (
        joke!($s; $($trigger),*; {
            send_random($channel_id, &$jokes).unwrap();
        })
    );
}

fn on_message(ctx: Context, msg: model::Message) {
    if msg.author.bot {
        return
    }
    let s: String = msg.content.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c|c.to_lowercase())
        .collect();

    joke!(s, msg.channel_id; "css", "source";; "Hvor er mine skins!?");
    joke!(s, msg.channel_id; "csgo", "counterstrike", "globaloffensive"; CSGO_MSGS);
    joke!(s, msg.channel_id; "mc", "minecraft";; "MINECRAFT!");
    joke!(s, msg.channel_id; "beartooth"; BEARTOOTH);
    joke!(s, msg.channel_id; "rep";; "Rep mig!");
    joke!(s, msg.channel_id; "ftl";; "Zoltan shield OP");
    joke!(s, msg.channel_id; "bindingofisaac";; "Mom OP");
    joke!(s, msg.channel_id; "meme";; "krydrede migmig'er");
    joke!(s, msg.channel_id; "gunsoficarus";; "Spillere online: 85");
    joke!(s, msg.channel_id; "doom";; "Rip and tear!");
    joke!(s, msg.channel_id; "dyinglight";; "Det dér Left 4 Dead-spil?");
    joke!(s; "english"; {
        msg.channel_id.send_message(|cm| {
            cm.embed(|e| {
                e.image("http://dev.lfalch.com/english.jpg")
            })
        }).unwrap();
    });
    joke!(s, msg.channel_id; "warthunder", "wt", "thunder", "tankspil";; "Jeg hader World of Tanks!");
    joke!(s, msg.channel_id; "ra3", "redalert"; REDALERT);
    joke!(s, msg.channel_id; "rusland", "russia", "росси", "russisk",
        "russian", "русск", "russer";; "Communism is the ultimate goal of socialism.");

    let user = {
        ctx.data.lock().unwrap().get::<BotUser>().unwrap().clone()
    };
    if msg.mentions.iter().map(|u| u.tag()).any(|u| u == user) {
        send_random(msg.channel_id, &RESPONSES).unwrap();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RpnError<'a> {
    StackTooSmall,
    UnknownOperator(&'a str)
}

use RpnError::*;

pub fn calculate<'a, T: IntoIterator<Item=&'a String>>(operations: T) -> StdResult<f64, RpnError<'a>> {
    let mut stack = Vec::new();

    for operation in operations {
        if let Some(d) = operation.parse::<f64>().ok() {
            stack.push(d)
        }else{
            calc(operation, &mut stack)?
        }
    }

    stack.pop().ok_or(StackTooSmall)
}

fn calc<'a>(op: &'a str, stack: &mut Vec<f64>) -> StdResult<(), RpnError<'a>>{
    let res = match (stack.pop(), stack.pop()){
        (Some(op1), Some(op2)) => match op{
            "+"|"add" => op2 + op1,
            "-"|"sub" => op2 - op1,
            "/"|"div" => op2 / op1,
            "*"|"mul" => op2 * op1,
            "^"|"pow" => op2.powf(op1),
            "log" => op2.log(op1),
            "hypot" => op2.hypot(op1),
            "%"|"rem" => op2 % op1,
            "|"|"or" => (op2 as i64 | op1 as i64) as f64,
            "&"|"and" => (op2 as i64 & op1 as i64) as f64,
            "xor" => (op2  as i64 ^ op1 as i64) as f64,
            _ => return Err(UnknownOperator(op))
        },
        _ => return Err(StackTooSmall)
    };
    Ok(stack.push(res))
}
