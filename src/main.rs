#![warn(clippy::all)]

use std::{io, fs::File, collections::HashMap};
use ::stalch::{run_with_state, InOuter, State, Result as StalchResult};
use ::seximal::to_seximal_words;

use std::env;
use std::result::Result as StdResult;
use typemap::Key;

use rand::{Rng, thread_rng, rngs::ThreadRng};

// use hyper::rt::Future;

use serenity::prelude::*;
use serenity::framework::standard::{CommandResult, Args, macros::*};
use serenity::model::{
    channel::*,
    gateway::*,
    id::*,
    misc::EmojiIdentifier
};
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
const COUNCIL: ChannelId = ChannelId(588016489811017749);
#[allow(clippy::unreadable_literal)]
const COUNCIL_POLLS: ChannelId = ChannelId(588054919676952596);
#[allow(clippy::unreadable_literal)]
const COUNCIL_POLLS_RESULTS: ChannelId = ChannelId(588106268946858006);

#[allow(clippy::unreadable_literal)]
const VOTE_YES: EmojiId = EmojiId(588070595401482269);
#[allow(clippy::unreadable_literal)]
const VOTE_NO: EmojiId = EmojiId(588070628456660992);

#[allow(clippy::unreadable_literal)]
const COUNCILLOR_ROLE: RoleId = RoleId(588012792326520836);

#[allow(clippy::unreadable_literal)]
const FALCHATS: GuildId = GuildId(189120762659995648);

#[command]
fn ping(ctx: &mut Context, message: &Message, _args: Args) -> CommandResult {
    message.channel_id.say(ctx, "Pong!")?;

    Ok(())
}

#[command]
fn info(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    msg.channel_id.send_message(ctx, |cm| {
        cm.embed(|e| {
            e.title("falchbot")
             .colour(utils::Colour::BLUE)
             .description("(c) LFalch.com 2018")
             .footer(|f| f.text(serenity::constants::USER_AGENT))
        })
    }).unwrap();
    Ok(())
}

#[command]
fn setgame(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if msg.author.id == FALCH {
        ctx.set_activity(Activity::playing(args.message()));
    } else {
        msg.reply(ctx, "Unauthorised")?;
    }
    Ok(())
}

// fn seticon(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
//     let s = args.message();

//     if msg.author.id == FALCH {
//         let img = {
//             let client = hyper::Client::new();
//             let mut resp = client.get(hyper::Uri::from_static(&format!("http://dev.lfalch.com/{}.png", s))).wait()?;
//             if !resp.status().is_success() {
//                 msg.reply(ctx, "No success")?;
//                 return Ok(());
//             }

//             let mut v = Vec::new();

//             resp.into_body().for_each(|chunk| {
//                 v.extend(chunk);
//             });

//             let b64 = base64::encode(&v);
//             let ext = "png";

//             format!("data:image/{};base64,{}", ext, b64)
//         };

//         FALCHATS.edit(|e| e.icon(Some(&img)))?;
//     } else {
//         msg.reply("Unauthorised")?;
//     }
//     Ok(())
// }

#[command]
fn say(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    if msg.author.id == FALCH {
        msg.channel_id.say(ctx, args.message())?;
    } else {
        msg.reply(ctx, "Unauthorised")?;
    }
    Ok(())
}

#[command]
fn rpn(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    match calculate(args.iter::<String>().map(|s| s.unwrap()).by_ref()) {
        Ok(r) => msg.reply(ctx, &format!("Result: {}", r)),
        Err(e) => msg.reply(ctx, &format!("Error: {:?}", e))
    }?;
    Ok(())
}

#[command]
fn stalch(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    match stalch_run(&(args.message().to_owned() + "\n")) {
        Ok((r, s)) => {
            if r.is_empty() {
                msg.reply(ctx, &format!("Stack:\n```\n{}\n```", s))
            } else {
                msg.reply(ctx, &format!("Output:\n```\n{}\n```", r))
            }
        }
        Err(e) => msg.reply(ctx, &format!("Error: {:?}", e))
    }?;
    Ok(())
}

#[command]
fn pdgqz(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let mut pdqz = ctx.data.write();
    let pdqz = pdqz.get_mut::<PdgqzDisalloweds>().unwrap();
    
    if let Some(i) = pdqz.iter().position(|ch| ch == &msg.channel_id) {
        pdqz.remove(i);
    } else {
        pdqz.push(msg.channel_id);
    }
    Ok(())
}

#[command]
fn seximal(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let s: String = args.message().chars().filter(|c| !c.is_whitespace()).collect();
    match to_seximal_words(&s) {
        Ok(ref s) => msg.reply(ctx, s),
        Err(_) => msg.reply(ctx, "Malformed number")
    }?;
    Ok(())
}

#[command]
fn emojis(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild = FALCHATS.to_partial_guild(&ctx).unwrap();
    let mut emoji: Vec<_> = guild.emojis.values().map(|e| (e.id.0, e.name.to_owned())).collect();

    if let Some(emoji_name) = args.current() {
        emoji.retain(|e| e.1 == emoji_name);
    }

    for emoji in emoji {
        msg.channel_id.say(&ctx, format!("{}: {}", emoji.1, emoji.0))?;
    }

    Ok(())
}

#[command]
fn vote(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    if msg.channel(&ctx).unwrap().private().is_some() {
        let user_id = msg.author.id;


        let mut voters = load_voters()?;
        let voter;
        if let Some(v) = voters.get(&user_id) {
            voter = v.clone();
        } else {
            voter = uid();
            voters.insert(user_id, voter.clone());
        }
        save_voters(&voters)?;
        msg.reply(ctx, &voter)?;
    } else {
        msg.reply(ctx, "Please DM me instead with this.")?;
    }
    Ok(())
}

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

const NUM_CHARS: usize = 36;
const CHARACTERS: [char; NUM_CHARS] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
    'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

struct RandomCharacterIter {
    rng: ThreadRng,
}

impl Iterator for RandomCharacterIter {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        Some(CHARACTERS[self.rng.gen_range(0, NUM_CHARS)])
    }
}

fn uid() -> String {
    RandomCharacterIter { rng: rand::thread_rng() }.take(20).collect::<String>()
}

type Voters = HashMap<UserId, String>;

use std::io::{Write, BufRead};


fn load_voters() -> io::Result<Voters> {
    let file = io::BufReader::new(match File::open("./voters.data") {
        Ok(f) => f,
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(Voters::new()),
        Err(e) => return Err(e)
    });

    let mut ret = Voters::new();

    for line in file.lines() {
        let line = line?;
        let l = line.trim();
        let i = line.find(':').unwrap();
        let (usr, vote_id) = l.split_at(i);

        ret.insert(UserId(usr.trim_end().parse::<u64>().map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?), vote_id[1..].trim_start().to_owned());
    }
    Ok(ret)
}
fn save_voters(users: &Voters) -> io::Result<()> {
    let mut file = File::create("./voters.data")?;

    for (usr, vote_id) in users {
        file.write_fmt(format_args!("{}:{}\n", usr.0, vote_id))?;
    }
    Ok(())
}

group!({
    name: "general",
    commands: [ping, info, emojis, setgame, rpn, stalch, pdgqz, seximal, vote, say]
});

#[derive(Default)]
struct Handler;

fn main() {
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");
    let mut client = Client::new(&token, Handler::default()).unwrap();

    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix(PREFIX))
        .group(&GENERAL_GROUP)
    );

    {
        let mut data = client.data.write();
        data.insert::<BotUser>(String::default());
        data.insert::<PdgqzDisalloweds>(Vec::new());
    }

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

macro_rules! joke {
    ($ctx:expr, $s:expr; $($trigger:expr),+; $bl:block) => (
        if $($s.contains($trigger))||* $bl
    );
    ($ctx:expr, $s:expr, $channel_id:expr; $($trigger:expr),+;; $joke:expr) => (
        joke!($ctx, $s; $($trigger),*; {
            $channel_id.say($ctx, $joke).unwrap();
        })
    );
    ($ctx:expr, $s:expr, $channel_id:expr; $($trigger:expr),+; $jokes:expr) => (
        joke!($ctx, $s; $($trigger),*; {
            send_random($ctx, $channel_id, &$jokes).unwrap();
        })
    );
}

impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        println!("Guilds:");
        for name in ready.guilds.iter().map(|g| g.id().to_partial_guild(&ctx).unwrap().name) {
            println!("    {}", name);
        }
        {
            let mut data = ctx.data.write();
            *data.get_mut::<BotUser>().unwrap() = ready.user.tag();
        }
    }

    fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return
        }
        {
            if ctx.data.read().get::<PdgqzDisalloweds>().unwrap().contains(&msg.channel_id) {
                let filter = |c: char| {
                    let c = c.to_lowercase().next().unwrap();
                     c == 'p' || c == 'd' || c == 'g' || c == 'q' || c == 'z'
                };
                if msg.content.chars().any(filter) {
                    let s = msg.content.chars().filter(|c| !filter(*c)).collect::<String>();
                    if !s.is_empty() {
                        msg.channel_id.say(&ctx, s).unwrap();
                    }
                }
            }
        }
        if msg.channel_id == COUNCIL {
            let start = msg.content.get(..6).unwrap_or("");
            if start == "poll: " || start == "Poll: " || start == "POLL: " {
                let poll = &msg.content[6..];

                let msg = COUNCIL_POLLS.say(&ctx, format!("{}: {}", msg.author.mention(), poll)).unwrap();

                let yes = EmojiIdentifier{id: VOTE_YES, name: "yes".to_owned()};
                let no = EmojiIdentifier{id: VOTE_NO, name: "no".to_owned()};

                msg.react(&ctx, yes).unwrap();
                msg.react(&ctx, no).unwrap();
            }
        }
        if msg.channel_id == MEMES && msg.author.id == WESTMANN && msg.attachments.iter().any(|a| a.width.is_some()) {
            msg.channel_id.say(&ctx, "Den er gammel!").unwrap();
        }
        let s: String = msg.content.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c|c.to_lowercase())
        .collect();

        joke!(&ctx, s, msg.channel_id; "css", "source";; "Hvor er mine skins!?");
        joke!(&ctx, s, msg.channel_id; "csgo", "counterstrike", "globaloffensive"; CSGO_MSGS);
        joke!(&ctx, s, msg.channel_id; "minecraft";; "MINECRAFT!");
        joke!(&ctx, s, msg.channel_id; "beartooth"; BEARTOOTH);
        joke!(&ctx, s, msg.channel_id; "rep";; "Rep mig!");
        joke!(&ctx, s, msg.channel_id; "ftl";; "Zoltan shield OP");
        joke!(&ctx, s, msg.channel_id; "bindingofisaac";; "Mom OP");
        joke!(&ctx, s, msg.channel_id; "meme";; "krydrede migmig'er");
        joke!(&ctx, s, msg.channel_id; "gunsoficarus";; "Spillere online: 8");
        joke!(&ctx, s, msg.channel_id; "doom";; "Rip and tear!");
        joke!(&ctx, s, msg.channel_id; "dyinglight";; "Det dér Left 4 Dead-spil?");
        joke!(&ctx, s, msg.channel_id; "report";; "ReviewBrah");
        joke!(&ctx, s; "english"; {
            msg.channel_id.send_message(&ctx, |cm| {
                cm.embed(|e| {
                    e.image("http://dev.lfalch.com/english.jpg")
                })
            }).unwrap();
        });
        joke!(&ctx, s, msg.channel_id; "warthunder", "thunder", "tankspil";; "Jeg elsker World of Tanks!");
        joke!(&ctx, s, msg.channel_id; "ra3", "redalert"; REDALERT);
        joke!(&ctx, s, msg.channel_id; "rusland", "russia", "росси", "russisk",
        "russian", "русск", "russer";; "Союз нерушимый республик свободных!");

        let user = {
            ctx.data.read().get::<BotUser>().unwrap().clone()
        };
        if msg.mentions.iter().map(|u| u.tag()).any(|u| u == user) {
            send_random(&ctx, msg.channel_id, &RESPONSES).unwrap();
        }
    }

    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        if add_reaction.channel_id == COUNCIL_POLLS {
            let message = add_reaction.message(&ctx).unwrap();

            if message.reactions.iter().any(|r| if let ReactionType::Unicode(ref s) = r.reaction_type {
                s == "❎" || s == "✅"
            } else { false }) {
                // Has already been decided
                return;
            }

            let mut aye_sayers = message.reaction_users(&ctx, EmojiIdentifier{id: VOTE_YES, name: "ja".to_owned()}, None, None).unwrap();
            let mut nay_sayers = message.reaction_users(&ctx, EmojiIdentifier{id: VOTE_NO, name: "nej".to_owned()}, None, None).unwrap();

            aye_sayers.retain(|u| !u.bot && u.has_role(&ctx, FALCHATS, COUNCILLOR_ROLE).unwrap());
            nay_sayers.retain(|u| !u.bot && u.has_role(&ctx, FALCHATS, COUNCILLOR_ROLE).unwrap());
            let abstainers: Vec<_> = aye_sayers.iter().filter(|u| nay_sayers.contains(&u)).cloned().collect();

            aye_sayers.retain(|u| !abstainers.contains(u));
            nay_sayers.retain(|u| !abstainers.contains(u));

            let pass_limit = (FALCHATS
                .members(&ctx, Some(1000), None::<UserId>)
                .unwrap()
                .iter()
                .filter(|member| member.roles.contains(&COUNCILLOR_ROLE))
                .count() - abstainers.len()) / 2;

            let (ayes, noes) = (aye_sayers.len(), nay_sayers.len());

            if ayes > pass_limit || noes > pass_limit {
                use std::cmp::Ordering::*;
                let verdict = match ayes.cmp(&noes) {
                    Greater => ("vedtaget ", "✅", aye_sayers),
                    Less => ("afslået ", "❎", nay_sayers),
                    Equal => return,
                };
                let mut list_of_people = String::with_capacity(37*verdict.2.len());
                for person in &verdict.2 {
                    list_of_people.push_str(", ");
                    list_of_people.push_str(&person.mention());
                }
                message.react(&ctx, verdict.1).unwrap();
                COUNCIL_POLLS_RESULTS.say(&ctx, format!("Følg. forslag er blevet **{}{}** {}-{} ({} abstained) af {}: \n{}", verdict.0, verdict.1, ayes, noes, abstainers.len(), &list_of_people[2..], message.content)).unwrap();
            }
        }
    }
}

fn send_random(ctx: &Context, chl: ChannelId, list: &[&str]) -> Result<Message> {
    let i = thread_rng().gen_range(0, list.len());
    chl.say(ctx, list[i])
}

#[derive(Debug, Clone)]
pub enum RpnError {
    StackTooSmall,
    UnknownOperator(String)
}

use crate::RpnError::*;

pub fn calculate<T: IntoIterator<Item=String>>(operations: T) -> StdResult<f64, RpnError> {
    let mut stack = Vec::new();

    for operation in operations {
        if let Ok(d) = operation.parse::<f64>() {
            stack.push(d)
        }else{
            calc(&operation, &mut stack)?
        }
    }

    stack.pop().ok_or(StackTooSmall)
}

fn calc(op: &str, stack: &mut Vec<f64>) -> StdResult<(), RpnError> {
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
            _ => return Err(UnknownOperator(op.to_owned()))
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
