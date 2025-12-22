use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use crossbeam_channel::{Receiver, Sender};
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

const LLM_MODEL: &str = "mistral:latest";
const PROMPT: &str = r#"SYSTEM: You are an NPC in a medieval RPG. NPC DATA: Name: Eldrin Role: Blacksmith Personality: Cynical, honest, tired Loyalty: Iron Guild Mood: Irritated WORLD STATE: - Baron raised taxes yesterday - Bandits near north road RULES: - Never mention game mechanics - Max 3 sentences - Speak in medieval tone PLAYER: Do you have any work for me?"#;

#[derive(Serialize)]
struct GenerationRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct GenerationResponse {
    response: String,
    done: bool,
}

fn spawn_task(channel: Res<MessageChannel>) {
    let pool = IoTaskPool::get();

    let _sender = channel.sender.clone();
    let msg_id = 1;
    // Spawn a task on the async compute pool
    pool.spawn(async move {
        let req = GenerationRequest {
            model: LLM_MODEL.to_string(),
            prompt: PROMPT.to_string(),
            stream: true,
        };

        let url = "http://localhost:11434/api/generate";

        if let Ok(body) = serde_json::to_string(&req) {
            let request = ehttp::Request::post(url, body.as_bytes().to_vec())
                .with_timeout(Some(Duration::from_secs(60)));

            ehttp::fetch(request, move |response| {
                match response {
                    Ok(res) => {
                        // println!("Got response: {:?}", res.text());
                        for line in res.text().unwrap().split('\n') {
                            // println!("Got line: {:?}", GenerationResponse::from(line));
                            let p_line: GenerationResponse =
                                serde_json::from_str(line).expect("error deserializing");
                            println!("Got line: {:?}", p_line);

                            if p_line.done {
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
                        println!("Error: {:?}", err);
                    }
                }
            });
        }
    })
    .detach();
}
