use super::*;

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
