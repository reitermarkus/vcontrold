use std::mem::ManuallyDrop;
use std::net::TcpStream;
use std::io::{BufReader, BufRead};
use std::os::unix::io::FromRawFd;

use nix::{self, errno::Errno::*, errno::*};

use super::*;

const LISTEN_QUEUE: c_int = 128;

#[no_mangle]
pub unsafe extern fn writen(fd: c_int, vptr: *const c_void, n: size_t) -> ssize_t {
  let mut ptr: *const c_char = vptr as _;
  let mut nleft: size_t = n;

  while nleft > 0 {
    let slice = std::slice::from_raw_parts(ptr as *const _, nleft);

    let nwritten = match nix::unistd::write(fd, &slice) {
      Ok(nwritten) => nwritten,
      Err(err) => {
        if err.as_errno() == Some(Errno::EINTR) {
          0
        } else {
          return -1
        }
      },
    };

    nleft -= nwritten;
    ptr = ptr.add(nwritten);
  }

  n as _
}

#[no_mangle]
pub unsafe extern fn Writen(fd: c_int, ptr: *mut c_void, nbytes: size_t) -> ssize_t {
  if writen(fd, ptr, nbytes) != nbytes as _ {
    log_it!(LOG_ERR, "Error writing to socket");
    return 0
  }

  nbytes as _
}

#[no_mangle]
pub unsafe extern fn readn(fd: c_int, vptr: *mut c_void, n: size_t) -> ssize_t {
  let mut ptr: *mut c_char = vptr as _;

  let mut nleft: size_t = n;
  while nleft > 0 {
    let mut slice = std::slice::from_raw_parts_mut(ptr as *mut _, nleft);

    let nread = match nix::unistd::read(fd, &mut slice) {
      Ok(0) => break,
      Ok(nread) => nread,
      Err(err) => {
        if err.as_errno() == Some(EINTR) {
          0
          } else if err.as_errno() == Some(EWOULDBLOCK) {
            return -2

            }else {
          return -1
        }
      },
    };

    nleft -= nread;
    ptr = ptr.add(nread);
  }

  (n - nleft) as _
}

#[no_mangle]
pub unsafe extern fn Readn(fd: c_int, ptr: *mut c_void , nbytes: size_t) -> ssize_t {
  let n: ssize_t = readn(fd, ptr, nbytes);

  if n < 0 {
    log_it!(LOG_ERR, "Error reading from socket");
    return 0
  }

  n
}

unsafe extern fn readline(fd: c_int, vptr: *mut c_void, max_len: size_t) -> ssize_t {
  let mut stream = TcpStream::from_raw_fd(fd);

  let mut line = String::new();
  let res = BufReader::new(&mut stream).read_line(&mut line);

  let _ = ManuallyDrop::new(stream);

  let n: ssize_t = match res {
    Ok(n) => n as _,
    Err(_) => return -1,
  };

  let line = CString::new(line.into_bytes()).unwrap();

  if n > max_len as _ {
    log_it!(LOG_ERR, "Line was too long to read");
    memcpy(vptr, line.as_ptr() as *const _, max_len);
  } else {
    memcpy(vptr, line.as_ptr() as *const _, n as _);
  }

  n
}

#[no_mangle]
pub unsafe extern fn Readline(fd: c_int, ptr: *mut c_void, maxlen: size_t) -> ssize_t {
  let n: ssize_t = readline(fd, ptr, maxlen);

  if n < 0 {
    log_it!(LOG_ERR, "Error reading from socket");
    return 0
  }

  n
}

#[no_mangle]
pub unsafe extern fn openSocket(tcpport: c_int, inetversion: c_int) -> c_int {
  let mut hints: addrinfo = mem::zeroed();

  match inetversion {
    6 => { hints.ai_family = PF_INET6 as _; },
    _ => { hints.ai_family = PF_INET as _; },
  }

  hints.ai_socktype = SOCK_STREAM as _;
  hints.ai_protocol = IPPROTO_TCP as _;
  hints.ai_flags    = AI_PASSIVE;

  let port = CString::new(tcpport.to_string()).unwrap();

  let mut res: *mut addrinfo = ptr::null_mut();

  let n = getaddrinfo(ptr::null(), port.into_raw(), &hints as *const addrinfo, &mut res as *mut *mut addrinfo);

  if n < 0 {
    let error_message = CStr::from_ptr(gai_strerror(n)).to_str().unwrap();
    log_it!(LOG_ERR, "getaddrinfo error: [{}]\n", error_message);
    return -1
  }

  let ressave = res.clone();

  let mut listenfd = -1;

  while let Some(curr_res) = res.as_ref() {
    listenfd = socket(curr_res.ai_family, curr_res.ai_socktype, curr_res.ai_protocol);

    let optval: *const c_int = &1;

    if listenfd >= 0 {
      if setsockopt(listenfd, SOL_SOCKET as _, SO_REUSEADDR as _, optval as *const c_void, mem::size_of_val(&*optval) as u32) < 0 {
        log_it!(LOG_ERR, "setsockopt failed!");
        exit(1);
      }

      if bind(listenfd, curr_res.ai_addr as *mut _, curr_res.ai_addrlen) == 0 {
        break
      }

      close(listenfd);
      listenfd = -1;
    }

    res = curr_res.ai_next;
  }

  freeaddrinfo(ressave);

  if listenfd < 0 {
    eprintln!("socket error: could not open socket");
    exit(1);
  }

  listen(listenfd, LISTEN_QUEUE);

  log_it!(LOG_NOTICE, "TCP socket {} opened", tcpport);

  listenfd
}

#[no_mangle]
pub unsafe extern fn listenToSocket(listenfd: c_int, makeChild: c_int, checkP: extern fn(ip: *mut c_char) -> c_short) -> c_int {
  let _ = checkP;

  let mut cliaddr: sockaddr_storage = mem::zeroed();

  signal(SIGCHLD as _, SIG_IGN);

  let cliaddrlen: *const usize = &mem::size_of_val(&cliaddr);

  loop {
    let connfd = accept(listenfd, &mut cliaddr as *mut sockaddr_storage as *mut sockaddr, cliaddrlen as *mut u32);

    let mut client_host: [c_char; NI_MAXHOST as usize] = mem::zeroed();
    let mut client_service: [c_char; NI_MAXSERV as usize] = mem::zeroed();

    getnameinfo(&mut cliaddr as *mut _ as *mut _, *cliaddrlen as _, client_host.as_mut_ptr(), NI_MAXHOST, client_service.as_mut_ptr(), NI_MAXSERV, NI_NUMERICHOST);

    let client_host = CStr::from_ptr(client_host.as_ptr());
    let client_service = CStr::from_ptr(client_service.as_ptr());

    if connfd < 0 {
        log_it!(LOG_NOTICE, "accept on host {}: port {}", client_host.to_str().unwrap(), client_service.to_str().unwrap());
        close(connfd);
        continue
    }

    log_it!(LOG_NOTICE, "Client connected {}:{} (FD:{})", client_host.to_str().unwrap(), client_service.to_str().unwrap(), connfd);

    if makeChild == 0 {
      return connfd;
    } else {
      match fork() {
        0 => {
          close(listenfd);
          return connfd;
        },
        childpid => {
          log_it!(LOG_INFO, "Child process started with pid {}", childpid);
        }
      }
    }

    close(connfd);
  }
}

#[no_mangle]
pub unsafe extern fn closeSocket(sockfd: c_int) {
  log_it!(LOG_INFO, "Closed connection (fd:{})", sockfd);
  close(sockfd);
}
