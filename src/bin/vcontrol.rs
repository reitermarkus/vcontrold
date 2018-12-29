use std::process::exit;

use clap::{crate_version, Arg, App, SubCommand, AppSettings::ArgRequiredElseHelp};
use serde_json;

use vcontrol::{Configuration, Optolink, VControl};

fn main() {
  let app = App::new("vcontrol")
              .version(crate_version!())
              .setting(ArgRequiredElseHelp)
              .help_short("?")
              .arg(Arg::with_name("config")
                .short("c")
                .long("config")
                .takes_value(true)
                .required(true)
                .help("path to the configuration file"))
              .arg(Arg::with_name("device")
                .short("d")
                .long("device")
                .takes_value(true)
                .conflicts_with_all(&["host", "port"])
                .help("path of the device"))
              .arg(Arg::with_name("host")
                .short("h")
                .long("host")
                .takes_value(true)
                .conflicts_with("device")
                .requires("port")
                .help("hostname or IP address of the device (default: localhost)"))
              .arg(Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .conflicts_with("device")
                .help("port of the device"))
              .subcommand(SubCommand::with_name("get")
                .about("get value")
                .arg(Arg::with_name("command")
                  .help("name of the command")
                  .required(true)))
              .subcommand(SubCommand::with_name("set")
                .about("set value")
                .arg(Arg::with_name("command")
                  .help("name of the command")
                  .required(true))
                .arg(Arg::with_name("value")
                  .help("value")
                  .required(true)));

  let matches = app.get_matches();

  let config = Configuration::open(matches.value_of("config").unwrap()).unwrap_or_else(|err| {
    eprintln!("Error: {}", err);
    exit(1);
  });

  let mut vcontrol = if let Some(device) = matches.value_of("device") {
    Optolink::open(device)
      .map(|device| VControl::new(device, config.commands()))
  } else if let Some(port) = matches.value_of("port") {
    let host = matches.value_of("host").unwrap_or("localhost");
    let port = port.parse().unwrap_or_else(|_| {
      eprintln!("Error: Could not parse port from “{}”.", port);
      exit(1);
    });

    Optolink::connect((host, port))
      .map(|device| VControl::new(device, config.commands()))
  } else {
    VControl::from_config(config)
  }.unwrap_or_else(|err| {
    eprintln!("Error: {}", err);
    exit(1);
  });

  if let Some(matches) = matches.subcommand_matches("get") {
    let command = matches.value_of("command").unwrap();

    match vcontrol.get(command) {
      Ok(output) => {
        println!("{}", serde_json::to_string(&output).unwrap());
      },
      Err(err) => {
        eprintln!("Error: {}", err);
        exit(1);
      }
    }
  }

  if let Some(matches) = matches.subcommand_matches("set") {
    let command = matches.value_of("command").unwrap();
    let value = matches.value_of("value").unwrap();

    match vcontrol.set(command, value) {
      Ok(()) => {},
      Err(err) => {
        eprintln!("Error: {}", err);
        exit(1);
      }
    }
  }
}
