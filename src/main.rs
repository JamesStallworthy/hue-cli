use clap::{Arg, App, Command};
use serde::{Serialize, Deserialize};
use serde_json::{Result, Value};
use std::collections::HashMap;
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
    username: String,
    aliases: HashMap::<String, String>
}

impl Default for Config{
    fn default() -> Config {
       Config {
           url: String::from("127.0.0.1"),
           username: String::new(),
           aliases: HashMap::new()
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
        .subcommand(
            Command::new("list")
                .long_flag("ls")
                .about("List all devices on the network")
        )
        .subcommand(
            Command::new("set")
                .long_flag("set")
                .about("Set a lights status")
                    .subcommand(Command::new("on")
                               .about("Set a light to on")
                               .arg(Arg::new("NAME")
                                    .required(true)
                                    )
                    )
                    .subcommand(Command::new("off")
                               .about("Set a light to off")
                               .arg(Arg::new("NAME")
                                    .required(true)
                                )
                    )
                    .subcommand(Command::new("bri")
                               .about("Set a brightness of a light between 0 and 100")
                               .arg(Arg::new("NAME")
                                    .required(true)
                                    )
                               .arg(Arg::new("BRI")
                                    .required(true)
                                    )
                    )
                    .subcommand(Command::new("alias")
                               .about("Set a alias name for a light so that it is easier to reference later")
                               .arg(Arg::new("NAME")
                                    .required(true)
                                    )
                               .arg(Arg::new("ALIAS")
                                    .required(true)
                                    )
                    )
        )
        .arg_required_else_help(true)
        .get_matches();

    let config = load_config();

    match matches.subcommand() {
        Some(("discover", _)) => discover(),
        Some(("test", _)) => {test(&config);}, 
        Some(("login", _)) => { let r = login(&config);
            if let Err(msg) = r{
                println!("{msg}");
            }
        },
        Some(("list", _)) => list(&config),
        Some(("set", sub)) => {
            match sub.subcommand() {
                Some(("on", args)) => set_state(State::On(true), String::from(args.value_of("NAME").unwrap()), &config),
                Some(("off", args)) => set_state(State::On(false), String::from(args.value_of("NAME").unwrap()), &config),
                Some(("bri", args)) => {
                    match validate_bri_value(args.value_of("BRI").unwrap()){
                        Ok(val) => set_state(State::Bri(val),String::from(args.value_of("NAME").unwrap()), &config),
                        Err(msg) => println!("Unable to set brightness: {}", msg)
                    }
                },
                Some(("alias", args)) => set_alias(String::from(args.value_of("NAME").unwrap()), String::from(args.value_of("ALIAS").unwrap()), &config),
                _ => unreachable!(),
            }
        },
        _ => unreachable!(),
    };
}

fn validate_bri_value(value: &str) -> std::result::Result<u8,Box<dyn std::error::Error>>{
   match value.parse::<u8>() {
       Ok(val) =>{ 
           if val <= 100{
               Ok(val)
           }
           else{
               Err("Brightness value not between 0 and 100".into())
           }
       },
       Err(msg) => Err(msg.into())
   }
}

fn test(config: &Config) -> bool{
    let testurl = format!("http://{}{}", config.url, HUE_BASE_PATH);
 
    let res = reqwest::blocking::get(&testurl).expect("Unable to connect to the hue bridge on {testurl}");
    match res.status(){
        reqwest::StatusCode::OK => {
            println!("Able to connect to the hue bridge on {testurl}");
            true
        },
        _ => {
            println!("Issue connecting to the hue bridge {testurl}");
            false
        }
    }
}

fn discover(){
    let res = reqwest::blocking::get(HUE_DISCOVER_URL).expect("Unable to connect to discover service");
    
    match res.status(){
        reqwest::StatusCode::OK => {
            let body = res.text().expect("Unable to read body");

            println!("{:?}", body);
            let value: Vec<DiscoverResponse> = serde_json::from_str(&body).unwrap();
            let ip = &value[0].internalipaddress;

            let url = String::from(ip);

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
       save_config(Config { ..Default::default() });
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

#[derive(Debug, Serialize, Deserialize)]
struct SuccessResponseModel<T> {
    success: T 
}

#[derive(Serialize, Deserialize)]
struct LoginSuccessModel {
    username: String
}

fn login(config :&Config) -> std::result::Result<(),String> {
    let devicetype;    
    {
        let devicename = whoami::devicename();

        devicetype = format!("{}#{}", APPLICATION_NAME, devicename);
    }

    println!("{devicetype}");

    println!("Press the link button on the hue bridge, then press any button to continue");
    let mut input_buffer = String::new(); 

    std::io::stdin()
        .read_line(&mut input_buffer)
        .expect("Failed to read line");

    let api_url = format!("http://{}{}", config.url, HUE_BASE_PATH);

    let reqbody = format!("{{\"devicetype\":\"{}\"}}", devicetype);
    let client = reqwest::blocking::Client::new();
    let res = client.post(api_url)
                        .body(reqbody)
                        .send().expect("Unable to post login request");

    let body = res.text().expect("Unable to read body");

    match serde_json::from_str::<Vec<SuccessResponseModel<LoginSuccessModel>>>(&body){
        Ok(val) => { 
            let new_config = Config{
                username: val[0].success.username.clone(),
                url: config.url.clone(),
                aliases: config.aliases.clone() 
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

fn list(config: &Config){
    print_lights(get_all_lights(config));
}

fn get_all_lights(config: &Config) -> Vec<LightModel>{
    let api_url = format!("http://{}{}/{}/lights", config.url,HUE_BASE_PATH, config.username);

    let res = reqwest::blocking::get(api_url).expect("Unable to connect to discover service");
    
    match res.status(){
        reqwest::StatusCode::OK => {
            let body = res.text().expect("Unable to read body");
                let v: Value = serde_json::from_str(&body).expect("");
                let mut list_of_lights : Vec<LightModel> = Vec::new();
                
                for (_ , value) in v.as_object().unwrap() {
                   list_of_lights.push(parse_light_json(value.to_string()));
                }

                list_of_lights
            },
        other => {
            println!("Failed to contact {HUE_DISCOVER_URL}. Status code: {other}");
            Vec::new()
        }
    }
}

fn parse_light_json(light_model_string: String) -> LightModel{
    let parse_light_json: LightModel = serde_json::from_str(&light_model_string).expect("Failed to parse light model");

    parse_light_json
}

fn print_lights(lights: Vec<LightModel>){
    for light in lights {
        if light.state.on {
            println!("{}: ON", light.name);
        }
        else{
            println!("{}: OFF", light.name);
        }
    }
}

#[derive(Serialize, Deserialize)]
struct LightModel {
    name: String,
    state: LightStateModel
}

#[derive(Serialize, Deserialize)]
struct LightStateModel {
    on: bool,
    bri: u8
}

enum State{
    On(bool),
    Bri(u8)
}

fn state_to_user_msg(s: State) -> String {
    match s {
        State::On(true) => String::from("Turned on"),
        State::On(false) => String::from("Turned off"),
        State::Bri(val) => format!("Set brightness to {} for", val)
    }
}

fn set_state(s: State, name: String, config: &Config){ 
    let light_model_state = match s{
        State::On(val) => format!("{{\"on\":{}}}", val),
        State::Bri(val) => {
            println!("{val}");
            format!("{{\"bri\":{}}}", (((val as f32)/100.0)*255.0).floor())
        }  
    };

    println!("{light_model_state}");

    let name = match config.aliases.get(&name) {
        Some(val) => val.clone(),
        None => name
    };

    //find the light
    let all_lights = get_all_lights(config);
    
    let mut index_of_light: i8 = -1;

    for (index,value) in all_lights.iter().enumerate() {
        if value.name == name {
            index_of_light = index as i8;
        }
    }

    if index_of_light == -1 {
        println!("Invalid light name {}", name);
        return;
    }
    //Hue does not use 0 index
    index_of_light += 1;

    let api_url = format!("http://{}{}/{}/lights/{}/state",config.url, HUE_BASE_PATH,config.username, index_of_light);

    let client = reqwest::blocking::Client::new();
    let res = client.put(api_url)
                        .body(light_model_state)
                        .send().expect("Unable to post login request");
    let body = res.text().unwrap();

    match serde_json::from_str::<Vec<SuccessResponseModel<Value>>>(&body){
        Ok(_) => println!("{} {} successfully",state_to_user_msg(s), name),
        Err(_) => {
            let error_msg: Vec<ErrorResponseModel> = serde_json::from_str(&body).unwrap();
            match error_msg[0].error.error_type {
                201 => println!("Cannot set value on a light that is not turned on"),
                _ => println!("Something went wrong when setting a state on the light: {}", error_msg[0].error.description)
            }
        }
    }
}

fn set_alias(name: String, a: String, config: &Config){
    if config.aliases.contains_key(&a) {
        println!("Alias {} has already been set", a);
        return
    }

    let lights = get_all_lights(config);

    let mut valid_light_name = false;
    for light in lights.iter() {
        if light.name == name{
            valid_light_name = true;
        }
    }

    if !valid_light_name{
        println!("Invalid light name: {}", name);
        return
    }

    let mut hm = config.aliases.clone();
    hm.insert(a, name);

    let new_config = Config{
        url : config.url.clone(),
        username : config.username.clone(),
        aliases: hm
    };

    save_config(new_config);
}
