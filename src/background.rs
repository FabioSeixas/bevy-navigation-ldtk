use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam_channel::{Receiver, Sender};
use libsql::Builder;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup_channel,
                // Ensure the channel is set up before spawning tasks.
                spawn_task.after(setup_channel),
            ),
        );
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub id: usize,
    pub content: String,
}

#[derive(Resource)]
pub struct MessageChannel {
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>,
}

fn setup_channel(mut commands: Commands) {
    let (sender, receiver) = crossbeam_channel::unbounded();
    commands.insert_resource(MessageChannel { sender, receiver });
}

const LLM_MODEL: &str = "phi3:3.8b";
const PROMPT: &str = "
<|system|>
SYSTEM: You are an NPC in a medieval fantasy RPG. You always speak in character. 

NPC PROFILE: 
Name: Eldrin 
Role: Blacksmith 
Personality: Cynical, blunt, tired 
Loyalty: Iron Guild 
Mood: Irritated 

WORLD FACTS: 
- Baron raised taxes yesterday 
- Bandits near the north road 
- You are out of ores and ingots

RULES: 
- Speak in first person 
- Medieval tone 
- No modern words 
- No explanations 
- Max 2 sentences 

AVOID: 
- 'I can help' 
- 'Perhaps' 
- 'It seems' 
- 'As a' 

IMPORTANT: 
Respond with ONLY the NPC's spoken dialogue. 
Do NOT describe actions. 
Do NOT explain your reasoning.
<|end|>

<|user|>
Greetings Blacksmith! I have an urgent order of Iron tools for the bridge building.
<|end|>

<|assistant|>
";

#[derive(Serialize)]
struct GenerationRequest {
    model: String,
    prompt: String,
    stream: bool,
    temperature: f32,
    top_p: f32,
    num_predict: usize,
    repeat_penalty: f32,
    stop: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct GenerationResponse {
    response: String,
    context: Option<Vec<u32>>,
    done: bool,
}

fn spawn_task(channel: Res<MessageChannel>) {
    let pool = IoTaskPool::get();

    let _sender = channel.sender.clone();
    let msg_id = 1;
    // Spawn a task on the async compute pool
    pool.spawn(async move {
        let db = Builder::new_local("local.db")
            .build()
            .await
            .expect("build new local.db failed");
        let conn = db.connect().expect("connect failed");

        println!("conn: {:?}", conn);

        let testing = conn
            .execute(
                "
             CREATE TABLE IF NOT EXISTS chat_context (
                id INTEGER PRIMARY KEY,
                context TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
              )
            ",
                (),
            )
            .await
            .expect("error running query");

        println!("result: {:?}", testing);

        let req = GenerationRequest {
            model: LLM_MODEL.to_string(),
            prompt: PROMPT.to_string(),
            stream: true,
            temperature: 0.6,
            top_p: 0.9,
            num_predict: 80,
            repeat_penalty: 1.1,
            stop: vec!["<|end|>:".to_string()],
        };

        let url = "http://localhost:11434/api/generate";

        if let Ok(body) = serde_json::to_string(&req) {
            let request = ehttp::Request::post(url, body.as_bytes().to_vec())
                .with_timeout(Some(Duration::from_secs(60 * 30)));

            let response = ehttp::fetch_blocking(&request);
            match response {
                Ok(res) => {
                    for line in res.text().unwrap().split('\n') {
                        let p_line: GenerationResponse =
                            serde_json::from_str(line).expect("error deserializing http response");

                        println!("line: {:?}", p_line);

                        if p_line.done {
                            if let Some(raw_context) = p_line.context {

                                let mut context = Context::from_ollama(raw_context);
                                context.truncate_last(2048);

                                let testing = conn
                                    .execute(
                                        "INSERT INTO chat_context (id, context) VALUES (?1, ?2)",
                                        (
                                            msg_id.to_string(),
                                            context.to_json().expect("Fail to convert context to json"),
                                        ),
                                    )
                                    .await
                                    .expect("error running insert query");

                                println!("result: {:?}", testing);
                            }
                            return;
                        } else {
                            _sender
                                .send(Message {
                                    id: msg_id,
                                    content: p_line.response,
                                })
                                .expect("error sending msg through channel");
                        }
                    }
                }
                Err(err) => {
                    println!("HTTP Error: {:?}", err);
                }
            }
        }
    })
    .detach();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    tokens: Vec<u32>,
}

impl Context {
    pub fn from_ollama(tokens: Vec<u32>) -> Self {
        Self {
            tokens: tokens.into_iter().map(|t| t as u32).collect(),
        }
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(&self.tokens)?)
    }

    pub fn from_json(s: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(s)?)
    }

    pub fn truncate_last(&mut self, max_tokens: usize) {
        if self.tokens.len() > max_tokens {
            let start = self.tokens.len() - max_tokens;
            self.tokens = self.tokens[start..].to_vec();
        }
    }
}
