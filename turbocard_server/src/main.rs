use std::io::Write;

#[tokio::main]
async fn main() {
 eprintln!("HULLO");

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
  .await
  .unwrap();

 while let Some(chunk) = res.chunk().await.unwrap() {
  let str = String::from_utf8(chunk.to_vec()).unwrap();
  if str == "data: [DONE]\n\n" {
   break;
  }
  let data = str.trim_start_matches("data: ").trim_end_matches('\n');
  let v: serde_json::Value = serde_json::from_str(&data).unwrap();

  let s = match v["choices"][0]["text"].clone() {
   serde_json::Value::String(s) => s,
   _ => panic!("Aaaa"),
  };
  print!("{}", s);
  std::io::stdout().flush().unwrap();
 }
 println!();
}
