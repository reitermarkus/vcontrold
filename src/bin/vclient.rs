use std::net::TcpStream;
use std::io::{self, Read, Write, BufReader, BufRead};
use std::fs::File;
use std::time::Duration;
use std::mem::drop;

use clap::{crate_version, Arg, App, ArgGroup, AppSettings::ArgRequiredElseHelp};

const PROMPT: &str = "vctrld>";
const SOCKET_TIMEOUT: Duration = Duration::from_secs(25);

pub fn recv_sync(socket: &mut TcpStream, string: &str) -> Result<String, io::Error> {
  let prev_timeout = socket.read_timeout()?;

  socket.set_read_timeout(Some(SOCKET_TIMEOUT))?;

  let mut response = String::new();

  loop {
    let mut buf: [u8; 1] = Default::default();
    socket.read_exact(&mut buf)?;
    response.push(buf[0] as char);

    if response.ends_with(string) {
      response.split_off(response.len() - string.len());
      break;
    }
  }

  socket.set_read_timeout(prev_timeout)?;

  Ok(response)
}

pub fn send_server(socket: &mut TcpStream, string: &str) -> Result<(), io::Error> {
  socket.set_nonblocking(true)?;

  let mut vec = Vec::new();
  let _ = socket.read_to_end(&mut vec);

  socket.set_nonblocking(false)?;

  socket.write(string.as_bytes())?;
  socket.flush()?;

  Ok(())
}

fn main() {
  let app = App::new("vclient")
              .version(crate_version!())
              .setting(ArgRequiredElseHelp)
              .help_short("?")
              .arg(Arg::with_name("host")
                 .short("h")
                 .long("host")
                 .takes_value(true)
                 .help("hostname or IP address of vcontrold (default: localhost)"))
              .arg(Arg::with_name("port")
                 .short("p")
                 .long("port")
                 .takes_value(true)
                 .help("port of vcontrold (default: 3002)"))
              .arg(Arg::with_name("command")
                 .short("c")
                 .long("command")
                 .multiple(true)
                 .takes_value(true)
                 .help("list of commands to be executed, sparated by commas"))
              .arg(Arg::with_name("commandfile")
                 .short("f")
                 .long("commandfile")
                 .takes_value(true)
                 .help("optional command file, one command per line"))
              .group(ArgGroup::with_name("commands")
                 .args(&["command", "commandfile"])
                 .required(true))
              .arg(Arg::with_name("output")
                 .short("o")
                 .long("output")
                 .takes_value(true)
                 .help("write to given file instead of STDOUT"))
              // .arg(Arg::with_name("csvfile")
              //    .short("s")
              //    .long("csvfile")
              //    .takes_value(true)
              //    .help("format output in CSV for further processing"))
              // .arg(Arg::with_name("template")
              //    .short("t")
              //    .long("template")
              //    .help("template, variables are substituted with acquired values"))
              // .arg(Arg::with_name("execute")
              //    .short("x")
              //    .long("execute")
              //    .help("the converted template (cf. -t) is written to the given file and executed subsequently"))
              // .arg(Arg::with_name("munin")
              //    .short("m")
              //    .long("munin")
              //    .conflicts_with("cacti")
              //    .help("output a Munin data logger compatible format (units and error details are discarded)"))
              // .arg(Arg::with_name("cacti")
              //    .short("k")
              //    .long("cacti")
              //    .conflicts_with("munin")
              //    .help("output a Cacti data logger compatible format (units and error details are discarded)"))
              // .arg(Arg::with_name("verbose")
              //    .short("v")
              //    .long("verbose")
              //    .help("be verbose (for testing purposes)"))
              // .arg(Arg::with_name("inet4")
              //    .short("4")
              //    .long("inet4")
              //    .help("force IPv4"))
              // .arg(Arg::with_name("inet6")
              //    .short("6")
              //    .long("inet6")
              //    .help("force IPv6"))
              ;

  let matches = app.get_matches();

  let host = matches.value_of("host").unwrap_or("localhost");
  let port = matches.value_of("port").map(|port| port.parse().unwrap()).unwrap_or(3002);
  // let csvfile = matches.value_of("csvfile").map(|csvfile| {
  //   File::create(csvfile).expect(&format!("could not create CSV file '{}'", csvfile))
  // });
  // let template = matches.value_of("template").map(|template| {
  //   File::open(template).expect(&format!("could not open template file '{}'", template))
  // });
  // let munin = matches.is_present("munin");
  // let cacti = matches.is_present("cacti");
  // let verbose = matches.is_present("verbose");
  // let force_ipv4 = matches.is_present("inet4");
  // let force_ipv6 = matches.is_present("inet6");

  let commands: Vec<String> = if let Some(commands) = matches.values_of("command") {
    commands.flat_map(|command| command.split(",")).map(|s| s.to_owned()).collect()
  } else if let Some(commandfile) = matches.value_of("commandfile") {
    let file = File::open(commandfile).expect(&format!("could not open command file '{}'", commandfile));
    BufReader::new(file).lines().collect::<Result<_, _>>().expect(&format!("error reading command file '{}'", commandfile))
  } else {
    Vec::new()
  };

  let mut out: Box<Write> = if let Some(outfile) = matches.value_of("outfile") {
    Box::new(File::create(outfile).expect(&format!("could not create output file '{}'", outfile)))
  } else {
    Box::new(io::stdout())
  };

  let mut socket = TcpStream::connect((host, port)).unwrap();

  recv_sync(&mut socket, PROMPT).unwrap();

  let responses: Vec<(String, String)> = commands.iter().map(|command| {
    send_server(&mut socket, &format!("{}\n", command)).unwrap();

    let res = recv_sync(&mut socket, PROMPT).unwrap();
    (command.to_owned(), res.trim_end().to_owned())
  }).collect();

  drop(socket);

  for (command, response) in responses {
    writeln!(&mut out, "{}:\n{}", command, response).expect("error writing output");
  }
}
