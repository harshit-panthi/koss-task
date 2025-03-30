use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
  GET,
  HEAD,
  POST,
  PUT,
  DELETE,
  CONNECT,
  OPTIONS,
  TRACE,
  PATCH,
}

impl FromStr for RequestType {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
      match s {
        "GET" => Ok(Self::GET),
        "HEAD" => Ok(Self::HEAD),
        "POST" => Ok(Self::POST),
        "PUT" => Ok(Self::PUT),
        "DELETE" => Ok(Self::DELETE),
        "CONNECT" => Ok(Self::CONNECT),
        "OPTIONS" => Ok(Self::OPTIONS),
        "TRACE" => Ok(Self::TRACE),
        "PATCH" => Ok(Self::PATCH),
        _ => Err(()),
      }
  }
}