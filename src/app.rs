use wasm_bindgen::prelude::*;
use yew::format::{Json, Text};
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    Text(String),
}

pub struct Model {
    link: ComponentLink<Self>,
    ws: Option<WebSocketTask>,
    messages: Vec<String>,
    input: String,
}

pub enum Msg {
    Connect,
    Connected,
    Disconnected,
    UpdateInput(String),
    Send,
    Received(Result<Message, anyhow::Error>),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        link.send_message(Msg::Connect);
        Self {
            link,
            ws: None,
            messages: vec![],
            input: "".to_string(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect => {
                let callback = self.link.callback(|Json(data)| Msg::Received(data));
                let notification = self.link.callback(|status| match status {
                    WebSocketStatus::Opened => Msg::Connected,
                    WebSocketStatus::Closed | WebSocketStatus::Error => {
                        Msg::Disconnected
                    }
                });
                let task = WebSocketService::connect(
                    "ws://127.0.0.1:8081", // Replace with the WebSocket proxy address
                    callback,
                    notification,
                )
                    .unwrap();
                self.ws = Some(task);
            }
            Msg::Connected => {
                self.messages.push("Connected".to_string());
            }
            Msg::Disconnected => {
                self.messages.push("Disconnected".to_string());
                self.ws = None;
            }
            Msg::UpdateInput(input) => {
                self.input = input;
            }
            Msg::Send => {
                if let Some(ws) = self.ws.as_mut() {
                    let message = Message::Text(self.input.clone());
                    let _ = ws.send(Json(&message));
                }
                self.input = "".to_string();
            }
            Msg::Received(Ok(msg)) => match msg {
                Message::Text(text) => {
                    self.messages.push(text);
                }
            },
            Msg::Received(Err(_)) => {}
        }
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <h1>{ "Yew Chat Client" }</h1>
                <div id="messages">
                    { for self.messages.iter().map(|msg| html! {<p>{ msg }</p>}) }
                </div>
                <input
                    placeholder="Type your message"
                    value=&self.input
                    oninput=self.link.callback(|e: InputData| {
                        Msg::UpdateInput(e.value)
                    })
                    onkeypress=self.link.callback(|e: KeyboardEvent| {
                        if e.key() == "Enter" { Msg::Send } else { Msg::UpdateInput(self.input.clone()) }
                    })
                />
            </>
        }
    }
}
