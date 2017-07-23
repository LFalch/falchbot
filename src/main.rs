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
    match calculate(&args.join(" ")) {
        Ok(r) => msg.reply(&format!("Result: {}", r)),
        Err(e) => msg.reply(&format!("Error: {:?}", e))
    }.unwrap();
});

const CSGO_MSGS: [&str; 5] = [
    "Vi varmer op med en comp!",
    "Jeg er på!",
    "Jeg er mere til Call of Duty ..",
    "Сука блядь!",
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
    "Hva' så din noob!? :P",
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

fn on_message(ctx: Context, msg: model::Message) {
    if msg.author.bot {
        return
    }
    let s: String = msg.content.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c|c.to_lowercase())
        .collect();

    if s.contains("css") || s.contains("source") {
        msg.channel_id.say("Hvor er mine skins!?").unwrap();
    }
    if s.contains("csgo") || s.contains("counterstrike") || s.contains("globaloffensive") {
        send_random(msg.channel_id, &CSGO_MSGS).unwrap();
    }
    if s.contains("mc") || s.contains("minecraft") {
        msg.channel_id.say("MINECRAFT!").unwrap();
    }
    if s.contains("beartooth") {
        send_random(msg.channel_id, &BEARTOOTH).unwrap();
    }
    if s.contains("rep") {
        msg.channel_id.say("Rep mig!").unwrap();
    }
    if s.contains("ftl") {
        msg.channel_id.say("Zoltan shield OP").unwrap();
    }
    if s.contains("bindingofisaac") {
        msg.channel_id.say("Mom OP").unwrap();
    }
    if s.contains("meme") {
        msg.channel_id.say("krydrede migmig'er").unwrap();
    }
    if s.contains("gunsoficarus") {
        msg.channel_id.say("Spillere online: 85").unwrap();
    }
    if s.contains("doom") {
        msg.channel_id.say("Rip and tear!").unwrap();
    }
    if s.contains("dyinglight") {
        msg.channel_id.say("Left 4 Dead?").unwrap();
    }
    if s.contains("english") {
        msg.channel_id.send_message(|cm| {
            cm.embed(|e| {
                e.image("http://dev.lfalch.com/english.jpg")
            })
        }).unwrap();
    }
    if s.contains("ra3") || s.contains("redalert") {
        send_random(msg.channel_id, &REDALERT).unwrap();
    }
    if s.contains("rusland") || s.contains("russia") || s.contains("росси") ||
        s.contains("russisk") || s.contains("russian") || s.contains("русск") || s.contains("russer"){
        msg.channel_id.say("Communism is the ultimate goal of socialism.").unwrap();
    }
    let user = {
        ctx.data.lock().unwrap().get::<BotUser>().unwrap().clone()
    };
    if msg.mentions.iter().map(|u| u.tag()).any(|u| u == user) {
        send_random(msg.channel_id, &RESPONSES).unwrap();
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RpnError {
    StackTooSmall,
    UnknownOperator(char),
    NoOperands
}

pub fn calculate(calculation: &str) -> StdResult<f64, RpnError>{
    let mut stack = Vec::new();

    let operations = calculation.trim().split(' ');

    for operation in operations {
        let d = operation.parse::<f64>().ok();
        if let Some(d) = d {
            stack.push(d);
        }else{
            calc(operation.chars().next().unwrap(), &mut stack)?
        }
    }

    stack.pop().ok_or(RpnError::NoOperands)
}

fn calc(op: char, stack: &mut Vec<f64>) -> StdResult<(), RpnError>{
    match (stack.pop(), stack.pop()){
        (Some(op1), Some(op2)) => match op{
            '+' => stack.push(op2 + op1),
            '-' => stack.push(op2 - op1),
            '/' => stack.push(op2 / op1),
            '*' => stack.push(op2 * op1),
            '^' => stack.push(op2.powf(op1)),
            _ => return Err(RpnError::UnknownOperator(op))
        },
        _ => return Err(RpnError::StackTooSmall)
    }
    Ok(())
}
