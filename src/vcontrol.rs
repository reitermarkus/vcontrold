 use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use clap::{crate_version, Arg, App, ArgGroup, AppSettings::ArgRequiredElseHelp};

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
        let output: Box<std::fmt::Display> = unit.bytes_to_output(&buf);
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
                 .help("path of the device"))
              .arg(Arg::with_name("host")
                 .short("h")
                 .long("host")
                 .takes_value(true)
                 .help("hostname or IP address of the device"))
              .arg(Arg::with_name("port")
                 .short("p")
                 .long("port")
                 .takes_value(true)
                 .help("port of the device"));

  let matches = app.get_matches();

  let host = matches.value_of("host").unwrap_or("localhost");
  let port = matches.value_of("port").map(|port| port.parse().unwrap()).unwrap_or(3002);

  let mut socket = TcpStream::connect((host, port)).unwrap();

  socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();


  let temp_command = config.prepare_command("KW2", "outside_temp_actual", "get", &[]);
  println!("\"temp_command\": {:#?}", temp_command);

  let res = execute_command(&mut socket, &temp_command).unwrap().unwrap();
}
