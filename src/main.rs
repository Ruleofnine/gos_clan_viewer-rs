use chrono::{Date, Duration, FixedOffset, Local, TimeZone};
use clap::Parser;
use owo_colors::{AnsiColors, OwoColorize};
use regex::Regex;
use soup::prelude::*;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    ///Path to the file
    #[arg(required = false,default_value_t=String::from("example.html"))]
    path: String,
    ///Days to be considered inactive
    #[arg(short, long, default_value_t = 30)]
    days: i64,
    ///To show only active memebers
    #[arg(short, long, default_value_t = false)]
    active: bool,
    ///To show only inactive memebers
    #[arg(short, long, default_value_t = false)]
    inactive: bool,
}
#[derive(PartialEq, Debug)]
enum DateStatus {
    Date(Date<FixedOffset>),
    NONE(),
}
impl fmt::Display for DateStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DateStatus::Date(i) => write!(f, "{}", i.format("%m/%d/%Y")),
            DateStatus::NONE() => write!(f, "{}", Status::NONE.color(AnsiColors::Magenta)),
        }
    }
}

struct Player {
    username: String,
    user_id: usize,
    //chat rank: usize,
    last_login: DateStatus,
    last_action: DateStatus,
    login_status: Status,
    action_status: Status, 
    login_color: AnsiColors,
    action_color: AnsiColors,
}
fn regex_out_date(text: &str, pattern: &str, offset: &FixedOffset) -> DateStatus {
    let pattern = Regex::new(pattern).unwrap();
    let string = match pattern.captures(&text) {
        Some(caps) => Some(caps.get(1).unwrap().as_str().to_owned()),
        _ => None,
    };
    let datetime = match string {
        Some(i) => {
            let validated_date = validate_date(i);
            let ymd = validated_date
                .split("/")
               .map(|a| a.parse::<u32>().unwrap())
                .collect::<Vec<u32>>();
            DateStatus::Date(offset.ymd(ymd[2] as i32, ymd[0], ymd[1]))
        }
        None => DateStatus::NONE(),
    };
    datetime
}
fn validate_date(text: String) -> String {
    let mut date_str = Local::today().to_string();
    let len = date_str.len();
    let _ = date_str.drain(2..len);
    let year_regex = Regex::new(r"(\d{2}/\d{2}/)(\d{2}$)").unwrap();
    let year_string = match year_regex.captures(&text) {
        Some(i) => {
            let year = i.get(2).unwrap().as_str().to_owned();
            let md = i.get(1).unwrap().as_str().to_owned();
            md + &date_str + &year
        }
        _ => text,
    };
    year_string
}

#[derive(PartialEq, Debug)]
enum Status {
    ACTIVE,
    INACTIVE,
    NONE,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let to_write = match self {
            Status::INACTIVE => "INACTIVE",
            Status::ACTIVE => "ACTIVE",
            Status::NONE => "NONE",
        };
        write!(f, "{}", to_write)
    }
}

impl Player {
    fn new(
        username: String,
        user_id: usize,
        last_login: DateStatus,
        last_action: DateStatus,
    ) -> Player {
        Player {
            username,
            user_id,
            last_login,
            last_action,
            login_status: Status::NONE,
            action_status: Status::NONE,
            login_color: AnsiColors::Red,
            action_color: AnsiColors::Red, // chat_rank,
        }
    }
    fn status(&mut self, days: Duration, offset: &FixedOffset) {
        self.login_status = match self.last_login {
            DateStatus::Date(login) => {
                if login + days > Local::today().with_timezone(offset) {
                    self.login_color = AnsiColors::Green;
                    Status::ACTIVE
                } else {
                    self.login_color = AnsiColors::Red;
                    Status::INACTIVE
                }
            }
            DateStatus::NONE() => Status::NONE,
        };
        self.action_status = match self.last_action {
            DateStatus::Date(action) => {
                if action + days > Local::today().with_timezone(offset) {
                    self.action_color = AnsiColors::Green;
                    Status::ACTIVE
                } else {
                    self.action_color = AnsiColors::Red;
                    Status::INACTIVE
                }
            }
            DateStatus::NONE() => {
                self.action_color = AnsiColors::Red;
                Status::NONE
            }
        }
    }
    fn print(&self) {
        println!(
            "{}: ID {} Login: {}[{}] Action {}[{}]",
            // self.chat_rank.blue(),
            self.username.yellow(),
            self.user_id.cyan(),
            self.last_login.color(self.login_color),
            self.login_status.color(self.login_color),
            self.last_action.color(self.action_color),
            self.action_status.color(self.action_color)
        );
    }
}
fn main() {
    let local = Local::now();
    let offset = local.offset();
    let args = Args::parse();
    let path = Path::new(&args.path);
    let days = Duration::days(args.days);
    let display = path.display();
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't read {}:, {}", display, why),
        Ok(file) => file,
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}:, {}", display, why),
        Ok(_) => (),
    };
    let pattern = Regex::new(r"myForm").unwrap();
    let soup = Soup::new(&s);
    let player_forms = soup.tag("form").attr("id", pattern).find_all();
    for x in player_forms {
        let username = x
            .tag("input")
            .attr("type", "hidden")
            .attr("name", "user")
            .find()
            .expect("failed username")
            .get("value")
            .unwrap();
        let id = x
            .tag("input")
            .attr("type", "hidden")
            .attr("name", "id")
            .find()
            .expect("failed id")
            .get("value")
            .unwrap();
        // let chat_rank = x
        //     .tag("img")
        //     .find()
        //     .expect("failed to find chat rank")
        //     .get("src")
        //     .unwrap()
        //     .split("_")
        //     .collect::<Vec<_>>()
        //     .last()
        //     .unwrap()
        //     .replace(&['.', 'p', 'n', 'g'][..], "");
        let login_text = x.text().trim().to_owned();
        let login_date = regex_out_date(&login_text, r"Last Login: (\d{2}/\d{2}/\d{2,4})", offset);
        let action_date = regex_out_date(
            &login_text,
            r"Last Action: (\d{1,2}/\d{1,2}/\d{2,4})",
            offset,
        );
        let mut player = Player::new(
            username,
            id.parse::<usize>().unwrap(),
            login_date,
            action_date,
            // chat_rank.parse::<usize>().unwrap(),
        );
        player.status(days, offset);
        if args.active
            && (player.login_status == Status::ACTIVE || player.action_status == Status::ACTIVE)
        {
            player.print();
        } else if args.inactive
            && (player.login_status == Status::INACTIVE || player.action_status == Status::INACTIVE)
        {
            player.print();
        } else if !args.inactive && !args.active {
            player.print()
        }
    }
}
