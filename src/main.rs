extern crate sonr;

fn main() {
}
 
//  use std::io::{Result as IoResult, Read, Write};
//  use std::thread;
//  use std::fs::remove_file;
//  
//  use sonr::{SmallBuffer, Buffer};
//  use sonr::errors::Result;
//  // use sonr::{Chain, Spawn};
//  // use sonr::{Session, SessionResult};
//  // use sonr::Service;
//  
//  
//  static RESP: &'static [u8] = b"HTTP/1.1 200 OK\nContent-Type: text/html; charset=UTF-8\nContent-Encoding: UTF-8\nContent-Length: 126\nServer: Narwhal example http server\nAccept-Ranges: bytes\nConnection: close\n\n<html> <head> <title>An Example Page</title> </head> <body> Hello World, this is a very simple HTML document.  </body> </html>";
//  
//  // -----------------------------------------------------------------------------
//  // 		- Server -
//  // -----------------------------------------------------------------------------
//  fn serve(server_id: usize) -> Result<()> {
//      let uds_path = "/tmp/foo.sock";
//      remove_file(uds_path);
//  
//      const MAX_CON: usize = 10_000;
//  
//      // let mut tcp_1 = sonr::Server::<_, HttpSession, _>::new(
//      //     tcp_listener("0.0.0.0", 5000).unwrap(),
//      //     ServerConfig::new(0, MAX_CON - 1, 1),
//      // );
//  
//      // let mut tcp_2 = sonr::Server::<_, HttpSession, _>::new(
//      //     tcp_listener("0.0.0.0", 5000).unwrap(),
//      //     ServerConfig::new(0, MAX_CON - 1, 1),
//      // );
//  
//      // let mut tcp_3 = sonr::Server::<_, HttpSession, _>::new(
//      //     tcp_listener("0.0.0.0", 5000).unwrap(),
//      //     ServerConfig::new(0, MAX_CON - 1, 1),
//      // );
//  
//      // let http = sonr::Server::<_, TcpConnection, _>::new(
//      //     tcp_listener("0.0.0.0", 8000).unwrap(),
//      //     SillyService::new(MAX_CON, MAX_CON + 1),
//      //     ServerConfig::new(MAX_CON, MAX_CON - 1, MAX_CON + 1),
//      // );
//  
//      // let uds = server::Server::<_, UdsConnection, _>::new(
//      //     uds_listener(uds_path).unwrap(),
//      //     SillyService::new(MAX_CON, 1 + MAX_CON * 2),
//      //     ServerConfig::new(MAX_CON * 2, MAX_CON - 1, 1 + MAX_CON * 2),
//      // );
//  
//      // thread::spawn(move || {
//      //     let res = tcp_2.run(1024);
//      // });
//      // thread::spawn(move || {
//      //     let res = tcp_3.run(1024);
//      // });
//      // let res = tcp_1.run(1024);
//      Ok(())
//  }
//  
//  fn main() -> Result<()> {
//      let res = serve(0);
//      println!("{:#?}", res);
//      Ok(())
//  }
//  
//  // fn temp() {
//  //     println!("");
//  //     println!("");
//  //     use common::config::*;
//  //     use std::net::{SocketAddr, ToSocketAddrs};
//  // 
//  //     let mut hm = HashMap::new();
//  // 
//  //     use std::path::PathBuf;
//  //     let pb = PathBuf::from("/test/bar");
//  // 
//  //     hm.insert("unixy".to_string(), Command {
//  //         keep_alive: true,
//  //         addr: Some(SourceAddr::Uds(pb))
//  //     });
//  // 
//  //     let sa = "127.0.0.1:5000".parse().unwrap();
//  //     hm.insert("netty".to_string(), Command {
//  //         keep_alive: false,
//  //         addr: Some(SourceAddr::Tcp(sa))
//  //     });
//  // 
//  // 
//  //     let cfg = Config {
//  //         host: None,
//  //         port: None,
//  //         uds_path: None,
//  //         commands: hm,
//  //     };
//  // 
//  //     println!("{}", toml::to_string(&cfg).unwrap());
//  // 
//  //     let raw = r#"
//  // [commands.unixy]
//  // keep_alive = true
//  // addr = "/test/bar"
//  // 
//  // [commands.netty]
//  // keep_alive = false
//  // addr = "127.0.0.1:5000"
//  //         "#;
//  // }
