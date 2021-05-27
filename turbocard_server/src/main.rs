use anyhow::Result;
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde::Serialize;
use std::io::Write;
use std::time::Duration;
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

 let (mut sender, mut receiver) = ws_stream.split();

 // let mut interval = tokio::time::interval(Duration::from_millis(1000));

 // Echo incoming WebSocket messages and send a message periodically every second.

 loop {
  tokio::select! {
   msg = receiver.next() => {
    match msg {
     Some(msg) => {

      let msg = msg?;
      if msg.is_text() {  //  || msg.is_binary()
       eprintln!("{}", msg.to_string());
       run_completion(msg.to_string(), &mut sender).await?;
       // sender.send(msg).await?;
      } else if msg.is_close() {
       break;
      }
     }
     None => break,
    }
   }
   // _ = interval.tick() => {
    // sender.send(Message::Text("tick".to_owned())).await?;
   // }
  }
 }

 // read.forward(write).await.expect("Failed to forward message")
 Ok(())
}

#[derive(Default, Serialize)]
struct CompletionRequest {
 prompt: Option<String>,
 max_tokens: Option<u16>,
 temperature: Option<f32>,
 top_p: Option<f32>,
 n: Option<u8>,
 stream: Option<bool>,
 logprobs: Option<u16>,
 presence_penalty: Option<f32>,
 frequency_penalty: Option<f32>,
}

async fn run_completion<S>(prompt: String, sender: &mut S) -> Result<()>
where
 S: SinkExt<Message> + Unpin,
{
 let req = CompletionRequest {
  prompt: Some(prompt),
  temperature: Some(0.7),
  max_tokens: Some(64),
  top_p: Some(1.0),
  frequency_penalty: Some(0.1),
  presence_penalty: Some(0.1),
  stream: Some(true),
  ..Default::default()
 };

 let client = reqwest::Client::new();
 let mut res = client
  .post("https://api.openai.com/v1/engines/davinci/completions")
  .header(reqwest::header::CONTENT_TYPE, "application/json")
  .header(reqwest::header::AUTHORIZATION, include_str!("../../credentials/openai"))
  .body(serde_json::to_string(&req).unwrap())
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
  sender.send(Message::Text(s)).await.unwrap_or_else(|_| panic!("couldn't send"));
  std::io::stdout().flush()?;
 }
 println!();

 Ok(())
}
