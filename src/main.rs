use clap::{Arg, App, Command};
use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};
use std::fs::{File, self};
use std::io::prelude::*;
use std::path::Path;

static HUE_DISCOVER_URL: &str = "https://discovery.meethue.com/";
static HUE_BASE_PATH: &str = "/api";
static CONFIG_FILE: &str = "config.json";
static APPLICATION_NAME: &str = "hue-cli";


#[derive(Serialize, Deserialize)]
struct Config {
    url: String,
    username: String
}

impl Default for Config{
    fn default() -> Config {
       Config {
           url: String::from("127.0.0.1"),
           username: String::new()
       } 
    }
}

#[derive(Deserialize)]
struct DiscoverResponse {
    id : String,
    internalipaddress : String,
    port : u64,
}

fn main() {
    let matches = App::new("Hue Cli Application")
        .version("0.1.0")
        .author("James Stallworthy <james@jamesstallworthy.com>")
        .about("Control your hue lights from the cli!")
        .subcommand(
            Command::new("discover")
                .short_flag('d')
                .long_flag("discover")
                .about("force discovery")
        )
        .subcommand(
            Command::new("test")
                .long_flag("test")
                .about("Test connection to hue bridge")
        )
        .subcommand(
            Command::new("login")
                .short_flag('l')
                .long_flag("login")
                .about("Test connection to hue bridge")
        )
        .arg_required_else_help(true)
        .get_matches();

    let config = load_config();

    match matches.subcommand() {
        Some(("discover", _)) => discover(),
        Some(("test", _)) => {test(&config);}, 
        Some(("login", _)) => { match login(&config) {
                Err(msg) => println!("{msg}"),
                _ => ()
            }
        },
        _ => unreachable!(),
    };
}

fn test(config: &Config) -> bool{
    let mut testurl = String::new();
 
    testurl.push_str("http://");
    testurl.push_str(&config.url);
    testurl.push_str(HUE_BASE_PATH);

    let res = reqwest::blocking::get(&testurl).expect("Unable to connect to the hue bridge on {testurl}");
    let result = match res.status(){
        reqwest::StatusCode::OK => {
            println!("Able to connect to the hue bridge on {testurl}");
            true
        },
        _ => {
            println!("Issue connecting to the hue bridge {testurl}");
            false
        }
    };

    result
}

fn discover(){
    let res = reqwest::blocking::get(HUE_DISCOVER_URL).expect("Unable to connect to discover service");
    
    match res.status(){
        reqwest::StatusCode::OK => {
            let body = res.text().expect("Unable to read body");

            println!("{:?}", body);
            let value: Vec<DiscoverResponse> = serde_json::from_str(&body).unwrap();
            let ip = &value[0].internalipaddress;

            let mut url = String::new();

            url.push_str(ip);
            url.push(':');

            println!("Hue bridge is located at {ip}");
            save_config(Config{
                url,
                ..Default::default()
            });
        },
        other => println!("Failed to contact {HUE_DISCOVER_URL}. Status code: {other}")
    }
}

fn save_config(new_config: Config){
    let mut file = File::create(CONFIG_FILE).expect("Unable to create config file");
    
    let new_config = serde_json::to_string(&new_config).unwrap();

    file.write_all(new_config.as_bytes()).unwrap();
}

fn load_config() -> Config {
   if !Path::new(CONFIG_FILE).exists(){
       save_config(Config { url: String::new(), ..Default::default() });
   }

   let config = fs::read_to_string(CONFIG_FILE).unwrap();
   let config: Config = serde_json::from_str(&config).unwrap();

   config
}

#[derive(Serialize, Deserialize)]
struct ErrorResponseModel {
    error: ErrorModel
}

#[derive(Serialize, Deserialize)]
struct ErrorModel {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    error_type: u64,
    address: String,
    description: String
}

#[derive(Serialize, Deserialize)]
struct SuccessResponseModel {
    success: LoginSuccessModel 
}

#[derive(Serialize, Deserialize)]
struct LoginSuccessModel {
    username: String
}

fn login(config :&Config) -> std::result::Result<(),String> {
    let mut devicetype = String::new();
    
    {
        let devicename = whoami::devicename();

        devicetype.push_str(APPLICATION_NAME);
        devicetype.push('#');
        devicetype.push_str(&devicename);
    }

    println!("Press the link button on the hue bridge, then press any button to continue");
    let mut input_buffer = String::new(); 

    std::io::stdin()
        .read_line(&mut input_buffer)
        .expect("Failed to read line");

    let mut api_url = String::new();

    api_url.push_str("http://");
    api_url.push_str(&config.url);
    api_url.push_str(HUE_BASE_PATH);

    let reqbody = format!("{{\"devicetype\":\"{}\"}}", devicetype);
    let client = reqwest::blocking::Client::new();
    let res = client.post(api_url)
                        .body(reqbody)
                        .send().expect("Unable to post login request");

    let body = res.text().expect("Unable to read body");

    match serde_json::from_str::<Vec<SuccessResponseModel>>(&body){
        Ok(val) => { 
            let new_config = Config{
                username: val[0].success.username.clone(),
                url: config.url.clone()
            };

            save_config(new_config);
        }
        Err(_) => {
            let error_response: Vec<ErrorResponseModel> = serde_json::from_str(&body).expect("Unable to read the response from the hue bridge");
            return Err(format!("Unable to login to the philips hue bridge for the following reason: {}", error_response[0].error.description));
        } 
    };
    
    Ok(())
}
