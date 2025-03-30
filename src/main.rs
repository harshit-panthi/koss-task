mod threadpool;
mod httpreq;

use std::{
  fs::{self, File},
  io::{prelude::*, BufReader},
  net::{TcpListener, TcpStream}, path::Path,
};

use threadpool::*;
use httpreq::*;

fn main() {
  let listener = TcpListener::bind("127.0.0.1:1560").unwrap();
  let tp = ThreadPoolHandle::new(4);

  for stream in listener.incoming() {
      let stream = stream.unwrap();

      tp.queue(|| handle_connection(stream));
  }
}

fn handle_connection(mut stream: TcpStream) {
  let buf_reader = BufReader::new(&stream);
  let request: Vec<_> =  buf_reader
  .lines()
  .map(|result| result.unwrap())
  .take_while(|line| !line.is_empty())
  .collect();
  let request_line: Vec<_> = request.get(0).unwrap().split(' ').collect();
  let len_req_line = request_line.len();
  if len_req_line != 3 {
    println!("Only GET and HEAD methods are supported");
    return;
  }

  if *request_line.get(2).unwrap() != "HTTP/1.1" {
    println!("Only HTTP/1.1 is supported");
    return;
  }

  let method = match request_line.get(0).unwrap().parse::<RequestType>() {
    Ok(m) => m,
    Err(_) => {
      println!("invalid method");
      return;
    },
  };
  
  let mut path = String::from(*request_line.get(1).unwrap());

  // route / to hello.html
  if path == "/" {
    path = String::from("/hello.html");
  }

  let mut temp_path = String::from("./web");
  temp_path.push_str(&mut path);

  let path = temp_path;

  match method {
    RequestType::GET => {
      let (status_line, mut file) = match File::open(&path) {
        Err(_) => ("HTTP/1.1 404 NOT FOUND", File::open("./web/404.html").expect("404.html not found")),
        Ok(file) => ("HTTP/1.1 200 OK",file),
      };
      
      let mut contents = String::from("");
      file.read_to_string(&mut contents);
      let contents = contents;

      let content_length = contents.len();

      let response =
        format!("{status_line}\r\nContent-Length: {content_length}\r\n\r\n{contents}");
      stream.write_all(response.as_bytes()).unwrap();
    },

    RequestType::HEAD => {
      let (status_line, mut file) = match File::open(&path) {
        Err(_) => ("HTTP/1.1 404 NOT FOUND", File::open("./web/404.html").expect("404.html not found")),
        Ok(file) => ("HTTP/1.1 200 OK",file),
      };

      let mut contents = String::from("");
      file.read_to_string(&mut contents);
      let contents = contents;

      let content_length = contents.len();

      let response =
        format!("{status_line}\r\nContent-Length: {content_length}\r\n\r\n");
      stream.write_all(response.as_bytes()).unwrap();
    },
    _ => {
      println!("Only GET and HEAD methods are supported");
      return;
    },
  };
}