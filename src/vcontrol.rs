 use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use clap::{crate_version, Arg, App, ArgGroup, SubCommand, AppSettings::ArgRequiredElseHelp};

use vcontrol::{DEFAULT_CONFIG, Configuration, PreparedProtocolCommand, FromBytes, SysTime, CycleTime, ErrState};

fn execute_command(socket: &mut TcpStream, commands: &[PreparedProtocolCommand]) -> Result<Option<Vec<u8>>, io::Error> {
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
        assert_eq!(buf, *bytes);
      },
      PreparedProtocolCommand::Recv(unit) => {
        let len = unit.size();
        socket.write(&[len as u8])?;
        socket.flush().unwrap();

        let mut buf = vec![0; len];
        socket.read_exact(&mut buf)?;

        println!("Received: {}", buf.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<String>>().join(" "));
        let output = unit.bytes_to_output(&buf);
        println!("Output: {}", output.to_string());

        res = Some(buf);
      },
    }
  }

  Ok(res)
}

fn main() {
  let mut config = Configuration::default();

  println!("{:#?}", config);

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
                .help("hostname or IP address of the device"))
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

  let host = matches.value_of("host").unwrap_or("localhost");
  let port = matches.value_of("port").map(|port| port.parse().unwrap()).unwrap_or(3002);

  let mut socket = TcpStream::connect((host, port)).unwrap();

  socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();

  if let Some(matches) = matches.subcommand_matches("get") {
    let command = matches.value_of("command").unwrap();
    let command = config.prepare_command("KW2", command, "get", &[]);
    execute_command(&mut socket, &command).unwrap().unwrap();
  }

  if let Some(matches) = matches.subcommand_matches("set") {
    let command = matches.value_of("command").unwrap();
    let value = matches.value_of("value").unwrap();
    let command = config.prepare_command("KW2", command, "set", &[]);
    execute_command(&mut socket, &command).unwrap().unwrap();
  }
}
