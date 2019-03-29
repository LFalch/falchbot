#![warn(clippy::all)]

#[macro_use]
extern crate serenity;

use std::io;
use stalch::{run_with_state, InOuter, State, Result as StalchResult};

use std::env;
use std::io::Read;
use std::result::Result as StdResult;
use typemap::Key;

use rand::Rng;
use rand::thread_rng;

use serenity::prelude::*;
use serenity::model::*;
use serenity::Result;
use serenity::utils;

use serenity::framework::StandardFramework;

const PREFIX: &str = "]";
#[allow(clippy::unreadable_literal)]
const WESTMANN: UserId = UserId(229154015626264577);
#[allow(clippy::unreadable_literal)]
const FALCH: UserId = UserId(165877785544491008);
#[allow(clippy::unreadable_literal)]
const MEMES: ChannelId = ChannelId(306454829738491904);
#[allow(clippy::unreadable_literal)]
const FALCHATS: GuildId = GuildId(189120762659995648);

command!(info(_ctx, msg, _args) {
    msg.channel_id.send_message(|cm| {
        cm.embed(|e| {
            e.title("falchbot")
             .colour(utils::Colour::blue())
             .description("(c) LFalch.com 2018")
             .footer(|f| f.text(serenity::constants::USER_AGENT))
        })
    }).unwrap();
});

command!(setgame(ctx, msg, args) {
    if msg.author.id == FALCH {
        ctx.set_game(Game::playing(&args.join(" ")));
    } else {
        msg.reply("Unauthorised")?;
    }
});

command!(seticon(_ctx, msg, args) {
    let s = args.join("/");

    if msg.author.id == FALCH {
        let img = {
            let client = hyper::Client::new();
            let mut resp = client.get(&format!("http://dev.lfalch.com/{}.png", s)).send()?;
            if !resp.status.is_success() {
                msg.reply("No success")?;
                return Ok(());
            }

            let mut v = Vec::new();

            resp.read_to_end(&mut v)?;

            let b64 = base64::encode(&v);
            let ext = "png";

            format!("data:image/{};base64,{}", ext, b64)
        };

        FALCHATS.edit(|e| e.icon(Some(&img)))?;
    } else {
        msg.reply("Unauthorised")?;
    }
});

command!(rpn(_ctx, msg, args) {
    match calculate(&*args) {
        Ok(r) => msg.reply(&format!("Result: {}", r)),
        Err(e) => msg.reply(&format!("Error: {:?}", e))
    }?;
});

command!(stalch(_ctx, msg, args) {
    match stalch_run(&(args.join(" ") + "\n")) {
        Ok((r, s)) => {
            if r.is_empty() {
                msg.reply(&format!("Stack:\n```\n{}\n```", s))
            } else {
                msg.reply(&format!("Output:\n```\n{}\n```", r))
            }
        }
        Err(e) => msg.reply(&format!("Error: {:?}", e))
    }?;
});

command!(pdgqz(ctx, msg, _args) {
    let mut pdqz = ctx.data.lock();
    let pdqz = pdqz.get_mut::<PdgqzDisalloweds>().unwrap();
    
    if let Some(i) = pdqz.iter().position(|ch| ch == &msg.channel_id) {
        pdqz.remove(i);
    } else {
        pdqz.push(msg.channel_id);
    }
});

command!(seximal(_ctx, msg, args) {
    let s = args.join("");
    match seximal::to_seximal_words(&s) {
        Ok(ref s) => msg.reply(s),
        Err(_) => msg.reply("Malformed number")
    }?;
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

struct PdgqzDisalloweds;

impl Key for PdgqzDisalloweds {
    type Value = Vec<ChannelId>;
}

struct BotUser;

impl Key for BotUser {
    type Value = String;
}

#[derive(Default)]
struct Handler;

fn main() {
    println!("{} in {}", WESTMANN, MEMES);

    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler::default());

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix(PREFIX))
        .command("ping", |c| c.exec_str("Pong!"))
        .on("info", info)
        .on("setgame", setgame)
        .on("rpn", rpn)
        .on("stalch", stalch)
        .on("pdgqz", pdgqz)
        .on("seximal", seximal)
        .on("seticon", seticon)
    );

    {
        let mut data = client.data.lock();
        data.insert::<BotUser>(String::default());
        data.insert::<PdgqzDisalloweds>(Vec::new());
    }

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
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

impl EventHandler for Handler {
    fn on_ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("Guilds:");
        for name in ready.guilds.iter().map(|g| g.id().get().unwrap().name) {
            println!("    {}", name);
        }
        {
            let mut data = ctx.data.lock();
            *data.get_mut::<BotUser>().unwrap() = ready.user.tag();
        }
    }

    fn on_message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return
        }
        {
            if ctx.data.lock().get::<PdgqzDisalloweds>().unwrap().contains(&msg.channel_id) {
                let filter = |c: char| {
                    let c = c.to_lowercase().next().unwrap();
                     c == 'p' || c == 'd' || c == 'g' || c == 'q' || c == 'z'
                };
                if msg.content.chars().any(filter) {
                    let s = msg.content.chars().filter(|c| !filter(*c)).collect::<String>();
                    if !s.is_empty() {
                        msg.channel_id.say(s).unwrap();
                    }
                }
            }
        }
        if msg.channel_id == MEMES && msg.author.id == WESTMANN && msg.attachments.iter().any(|a| a.width.is_some()) {
            msg.channel_id.say("Den er gammel!").unwrap();
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
        joke!(s, msg.channel_id; "report";; "ReviewBrah");
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
            ctx.data.lock().get::<BotUser>().unwrap().clone()
        };
        if msg.mentions.iter().map(|u| u.tag()).any(|u| u == user) {
            send_random(msg.channel_id, &RESPONSES).unwrap();
        }
    }
}

fn send_random(chl: ChannelId, list: &[&str]) -> Result<Message> {
    let i = thread_rng().gen_range(0, list.len());
    chl.say(list[i])
}

#[derive(Debug, Copy, Clone)]
pub enum RpnError<'a> {
    StackTooSmall,
    UnknownOperator(&'a str)
}

use crate::RpnError::*;

pub fn calculate<'a, T: IntoIterator<Item=&'a String>>(operations: T) -> StdResult<f64, RpnError<'a>> {
    let mut stack = Vec::new();

    for operation in operations {
        if let Ok(d) = operation.parse::<f64>() {
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
    stack.push(res);
    Ok(())
}

fn stalch_run(s: &str) -> StalchResult<(String, String)> {
    let mut state = State::new();
    let mut io = InOuter::new(Vec::new(), io::repeat(b'\n'));

    run_with_state(s.as_bytes(), &mut state, &mut io)?;

    let (output, _) = io.extract();

    let s = String::from_utf8(output).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    Ok((s, format!("{:?}", state.stack())))
}
