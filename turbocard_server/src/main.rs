use anyhow::Result;
use futures_util::SinkExt;
use futures_util::StreamExt;
use std::io::Write;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

#[tokio::main]
async fn main() -> Result<()> {
 eprintln!("HULLO");
 run_ws_server().await
 // run_completion().await
}

async fn run_ws_server() -> Result<()> {
 let addr = "127.0.0.1:8080";
 let try_socket = TcpListener::bind(addr).await;
 let listener = try_socket.expect("Failed to bind");
 eprintln!("Listening on: {}", addr);

 while let Ok((stream, _)) = listener.accept().await {
  tokio::spawn(accept_connection(stream));
 }

 Ok(())
}

async fn accept_connection(stream: TcpStream) -> Result<()> {
 let addr = stream.peer_addr().expect("connected streams should have a peer address");
 eprintln!("Peer address: {}", addr);

 let ws_stream = tokio_tungstenite::accept_async(stream)
  .await
  .expect("Error during the websocket handshake occurred");

 eprintln!("New WebSocket connection: {}", addr);

 let (mut sender, receiver) = ws_stream.split();
 sender.send(Message::Text("tick".to_owned())).await?;

 // read.forward(write).await.expect("Failed to forward message")
 Ok(())
}

async fn run_completion() -> Result<()> {
 let client = reqwest::Client::new();
 let mut res = client
  .post("https://api.openai.com/v1/engines/davinci/completions")
  .header(reqwest::header::CONTENT_TYPE, "application/json")
  .header(reqwest::header::AUTHORIZATION, include_str!("../../credentials/openai"))
  .body(
   r#"{
  "prompt": "hello",
  "temperature": 0.7,
  "max_tokens": 64,
  "top_p": 1,
  "frequency_penalty": 0.1,
  "presence_penalty": 0.1,
  "stream": true
}"#,
  )
  .send()
  .await?;

 while let Some(chunk) = res.chunk().await? {
  let str = String::from_utf8(chunk.to_vec())?;
  if str == "data: [DONE]\n\n" {
   break;
  }
  let data = str.trim_start_matches("data: ").trim_end_matches('\n');
  let v: serde_json::Value = serde_json::from_str(&data)?;

  let s = match v["choices"][0]["text"].clone() {
   serde_json::Value::String(s) => s,
   _ => panic!("Aaaa"),
  };
  print!("{}", s);
  std::io::stdout().flush()?;
 }
 println!();

 Ok(())
}
