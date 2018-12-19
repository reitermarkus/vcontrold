use super::*;

use std::char;
use std::net::TcpStream;
use std::time::Duration;
use std::os::unix::io::FromRawFd;
use std::mem::ManuallyDrop;
use std::borrow::{Borrow, BorrowMut};

#[no_mangle]
pub unsafe extern fn connectServer(host: *const c_char, port: c_int) -> c_int {
  let host_string = CStr::from_ptr(host).to_str().unwrap();

  if host_string.chars().next() == Some('/') {
    log_it!(LOG_ERR, "Host format: IP|Name:Port");
    return -1
  }

  let sockfd = openCliSocket(host, port, 0);

  if sockfd == 0 {
    log_it!(LOG_INFO, "Setting up connection to {} port {} failed", host_string, port);
    return -1
  }

  log_it!(LOG_INFO, "Setup connection to {} port {}", host_string, port);
  sockfd
}

#[no_mangle]
pub unsafe extern fn recvSync(fd: c_int, wait: *mut c_char, recv: *mut *mut c_char) -> ssize_t {
  let wait = CStr::from_ptr(wait).to_str().unwrap();

  let mut stream = ManuallyDrop::new(TcpStream::from_raw_fd(fd));

  let prev_timeout = stream.borrow().read_timeout();

  stream.borrow_mut().set_read_timeout(Some(Duration::from_secs(CL_TIMEOUT as u64))).unwrap();

  let mut received_string = String::new();

  let mut c: c_char = mem::uninitialized();

  let mut count;

  loop {
    count = readn(fd, &mut c as *mut c_char as *mut c_void, 1);

    if count == -2 {
      log_it!(LOG_ERR, "timeout wait: {}", wait);
      if let Ok(prev_timeout) = prev_timeout {
        stream.borrow_mut().set_read_timeout(prev_timeout).unwrap();
      }
      return -1;
    }

    if count < 0 {
      continue
    }

    received_string.push(char::from_u32_unchecked(c as u32));

    if received_string.contains(wait) {
      log_it!(LOG_INFO, "recv: {}", received_string);
      break
    }
  }

  if count <= 0 {
    log_it!(LOG_ERR, "exit with count={}", count);
  }

  if let Ok(prev_timeout) = prev_timeout {
    stream.borrow_mut().set_read_timeout(prev_timeout).unwrap();
  }

  received_string = received_string.splitn(2, wait).next().unwrap().to_string();
  let len = received_string.len();
  *recv = calloc(len, mem::size_of::<c_char>()) as *mut c_char;
  let c_string = CString::new(received_string).unwrap();
  memcpy(*recv as *mut c_void, c_string.as_ptr() as *const _, len);

  count
}

#[no_mangle]
pub unsafe extern fn sendServer(fd: c_int, s_buf: *mut c_char, len: size_t) -> size_t {
  let mut stream = ManuallyDrop::new(TcpStream::from_raw_fd(fd));


  stream.borrow_mut().set_nonblocking(true).unwrap();

  let string = String::with_capacity(256);
  let c_string = CString::new(string).unwrap();

  // Empty buffer
  // As tcflush does not work correctly, we use nonblocking read
  while readn(fd, c_string.as_ptr() as *mut c_char as *mut c_void, 256) > 0 { }

  stream.borrow_mut().set_nonblocking(false).unwrap();

  Writen(fd, s_buf as *mut c_void, len as usize) as size_t
}

#[no_mangle]
pub unsafe extern fn disconnectServer(sockfd: c_int) {
  let string = "quit\n";
  let quit = CString::new(string).unwrap();
  let mut ptr: *mut c_char = mem::uninitialized();

  sendServer(sockfd, quit.as_ptr() as *mut c_char, string.len());
  recvSync(sockfd, BYE.as_ptr() as *mut c_char, &mut ptr);

  free(ptr as *mut c_void);
  close(sockfd);
}
