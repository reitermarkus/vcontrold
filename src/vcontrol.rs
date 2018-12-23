use std::io::{self, Read, Write};
use std::process::exit;

use clap::{crate_version, Arg, App, SubCommand, AppSettings::ArgRequiredElseHelp};

use vcontrol::{Configuration, PreparedProtocolCommand, OptoLink};

fn pretty_bytes(bytes: &[u8]) -> String {
  bytes.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<_>>().join(" ")
}

trait ReadWrite: Read + Write {}
impl<T> ReadWrite for T where T: Read + Write {}

fn execute_command(socket: &mut ReadWrite, commands: &[PreparedProtocolCommand]) -> Result<Option<Box<std::fmt::Display>>, io::Error> {
  // socket.set_nonblocking(true)?;
  //
  // let mut vec = Vec::new();
  // let _ = socket.read_to_end(&mut vec);
  //
  // socket.set_nonblocking(false)?;

  let mut res = None;

  for command in commands {
    match command {
      PreparedProtocolCommand::Send(bytes) => {
        socket.write(&bytes)?;
      },
      PreparedProtocolCommand::Wait(bytes) => {
        let mut buf = vec![0; bytes.len()];
        socket.read_exact(&mut buf)?;

        if &buf != bytes {
          return Err(io::Error::new(io::ErrorKind::Other, format!("failed to wait for {}, received {}", pretty_bytes(bytes), pretty_bytes(&buf))))
        }
      },
      PreparedProtocolCommand::Recv(unit) => {
        let len = unit.size();
        socket.write(&[len as u8])?;
        socket.flush().unwrap();

        let mut buf = vec![0; len];
        socket.read_exact(&mut buf)?;

        println!("Received: {}", pretty_bytes(&buf));

        res = Some(unit.bytes_to_output(&buf));
      },
    }
  }

  Ok(res)
}

fn main() {
  let config = Configuration::default();

  let app = App::new("vcontrol")
              .version(crate_version!())
              .setting(ArgRequiredElseHelp)
              .help_short("?")
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


  println!("Connecting ...");

  let mut device: Box<ReadWrite> = if let Some(device) = matches.value_of("device") {
    Box::new(OptoLink::open(device).unwrap())
  } else {
    let host = matches.value_of("host").unwrap_or("localhost");
    let port = matches.value_of("port")
                 .map(|port| {
                   port.parse().unwrap_or_else(|_| {
                     eprintln!("Error: Could not parse port from “{}”.", port);
                     exit(1);
                   })
                 })
                 .unwrap();

    Box::new(OptoLink::connect((host, port)).unwrap())
  };

  if let Some(matches) = matches.subcommand_matches("get") {
    let command = matches.value_of("command").unwrap();
    let command = config.prepare_command("KW2", command, "get", &[]);

    match execute_command(&mut device, &command) {
      Ok(res) => {
        if let Some(output) = res {
          println!("{}", output);
        };
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
    let command = config.prepare_command("KW2", command, "set", &[]);

    match execute_command(&mut device, &command) {
      Ok(res) => {
        if let Some(output) = res {
          println!("{}", output);
        };
      },
      Err(err) => {
        eprintln!("Error: {}", err);
        exit(1);
      }
    }
  }
}
