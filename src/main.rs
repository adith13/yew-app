use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::*;

use crate::chat::web_rtc_manager::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::str;

use base64;
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use yew::services::{ConsoleService, IntervalService, Task, TimeoutService};
#[allow(unused_imports)]
use yew::{
    html, html::NodeRef, App, Callback, Component, ComponentLink, Html, InputData, KeyboardEvent,
    ShouldRender,
};

pub enum MessageSender {
    Me,
    Other,
}

pub struct Message {
    sender: MessageSender,
    content: String,
}

impl Message {
    pub fn new(content: String, sender: MessageSender) -> Message {
        Message {
            content: content,
            sender: sender,
        }
    }
}

pub struct ChatModel {
    web_rtc_manager: Rc<RefCell<WebRTCManager>>,
    messages: Vec<Message>,
    link: ComponentLink<Self>,
    value: String,
    chat_value: String,
    node_ref: NodeRef,
}

impl Component for ChatModel {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let web_rtc_manager = WebRTCManager::create_default(link.clone());

        let rc = Rc::new(RefCell::new(web_rtc_manager));

        let model = ChatModel {
            web_rtc_manager: rc.clone(),
            messages: vec![],
            link: link,
            value: "".into(),
            chat_value: "".into(),
            node_ref: NodeRef::default(),
        };

        model
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartAsServer => {
                self.web_rtc_manager
                    .borrow_mut()
                    .set_state(State::Server(ConnectionState::new()));
                WebRTCManager::start_web_rtc(self.web_rtc_manager.clone());
                let re_render = true;
                return re_render;
            }

            Msg::ConnectToServer => {
                self.web_rtc_manager
                    .borrow_mut()
                    .set_state(State::Client(ConnectionState::new()));
                WebRTCManager::start_web_rtc(self.web_rtc_manager.clone());
                let re_render = true;
                return re_render;
            }

            Msg::UpdateWebRTCState(web_rtc_state) => {
                self.value = "".into();
                let debug = ChatModel::get_debug_state_string(&web_rtc_state);
                console::log_1(&debug.into());

                let re_render = true;
                return re_render;
            }

            Msg::ResetWebRTC => {
                let web_rtc_manager = WebRTCManager::create_default(self.link.clone());
                let rc = Rc::new(RefCell::new(web_rtc_manager));
                self.web_rtc_manager = rc;
                self.messages = vec![];
                self.chat_value = "".into();
                self.value = "".into();

                let re_render = true;
                return re_render;
            }

            Msg::UpdateInputValue(val) => {
                self.value = val;
                let re_render = true;
                return re_render;
            }

            Msg::UpdateInputChatValue(val) => {
                self.chat_value = val;
                let re_render = true;
                return re_render;
            }

            Msg::ValidateOffer => {
                let state = self.web_rtc_manager.borrow().get_state();

                match state {
                    State::Server(_connection_state) => {
                        let result = WebRTCManager::validate_answer(
                            self.web_rtc_manager.clone(),
                            &self.value,
                        );

                        if result.is_err() {
                            web_sys::Window::alert_with_message(
                                &web_sys::window().unwrap(),
                                &format!(
                                    "Cannot use answer. Failure reason: {:?}",
                                    result.err().unwrap()
                                ),
                            )
                            .expect("alert should work");
                        }
                    }
                    _ => {
                        let result = WebRTCManager::validate_offer(
                            self.web_rtc_manager.clone(),
                            &self.value,
                        );

                        if result.is_err() {
                            web_sys::Window::alert_with_message(
                                &web_sys::window().unwrap(),
                                &format!(
                                    "Cannot use offer. Failure reason: {:?}",
                                    result.err().unwrap()
                                ),
                            )
                            .expect("alert should work");
                        }
                    }
                };

                let re_render = true;
                return re_render;
            }

            Msg::NewMessage(message) => {
                self.messages.push(message);
                self.scroll_top();
                let re_render = true;
                return re_render;
            }

            Msg::Send => {
                let content = self.chat_value.clone();
                let my_message = Message::new(content.clone(), MessageSender::Me);
                self.messages.push(my_message);
                self.web_rtc_manager.borrow().send_message(content);
                self.chat_value = "".into();
                self.scroll_top();
                let re_render = true;
                return re_render;
            }

            Msg::Disconnect => {
                let web_rtc_manager = WebRTCManager::create_default(self.link.clone());
                let rc = Rc::new(RefCell::new(web_rtc_manager));
                self.web_rtc_manager = rc;
                self.messages = vec![];
                self.chat_value = "".into();
                self.value = "".into();
                let re_render = true;
                return re_render;
            }
        }
    }

    fn view(&self) -> Html {
        match &self.web_rtc_manager.borrow().get_state() {
            State::DefaultState => {
                html! {
                    <>
                        { self.get_chat_header() }

                        <main class="msger-chat" id="chat-main" ref=self.node_ref.clone()>
                            <div class="msg left-msg">

                                <div class="msg-bubble">

                                    <div class="msg-text">
                                        {"Hi, welcome to SimpleChat!
                                        To start you need to establish connection with your friend. Either click the button below to start generate an offer and create a code to send to your friend."}
                                        <br/>
                                        <button
                                            class="msger-send-btn"
                                            style="border-radius: 3px; padding: 10px; font-size: 1em; border: none; margin-left: 0px; margin-top: 6px;"
                                            onclick=self.link.callback(|_| Msg::StartAsServer)>
                                            {"I will generate an offer first!"}
                                        </button>
                                    </div>
                                </div>
                            </div>

                            <div class="msg right-msg">

                                <div class="msg-bubble">

                                    <div class="msg-text">
                                        {"Alternatively, if your friend has already a code click the button below."}
                                        <br/>
                                            <button
                                                class="msger-send-btn"
                                                style="border-radius: 3px; padding: 10px; font-size: 1em; border: none; margin-left: 0px; margin-top: 6px; float: right;"
                                                onclick=self.link.callback(|_| Msg::ConnectToServer)>
                                                {"My friend already send me a code!"}
                                            </button>
                                    </div>
                                </div>
                            </div>
                        </main>

                        { self.get_input_for_chat_message() }
                    </>
                }
            }

            State::Server(connection_state) => {
                html! {
                    <>
                        { self.get_chat_header() }

                        <main class="msger-chat" id="chat-main" ref=self.node_ref.clone()>
                        {
                            if
                                connection_state.data_channel_state.is_some() &&
                                connection_state.data_channel_state.unwrap() == RtcDataChannelState::Open
                            {
                                html! {

                                    <>
                                        { self.get_messages_as_html() }
                                    </>
                                }
                            } else if connection_state.ice_gathering_state.is_some() {
                                html! {
                                    <>

                                        <div class="msg left-msg">

                                            <div class="msg-bubble">
                                                <div class="msg-info">
                                                </div>

                                                <div class="msg-text">
                                                    { self.get_offer_and_candidates() }
                                                </div>
                                            </div>
                                        </div>

                                        <div class="msg left-msg">

                                            <div class="msg-bubble">
                                                <div class="msg-info">
                                                </div>

                                                <div class="msg-text">
                                                    { "And then paste his/her answer below "}
                                                    { self.get_validate_offer_or_answer() }
                                                </div>
                                            </div>
                                        </div>
                                    </>
                                }
                            } else {
                                html! {}
                            }
                        }
                        </main>

                        { self.get_input_for_chat_message() }
                    </>
                }
            }

            State::Client(connection_state) => {
                html! {
                    <>
                        { self.get_chat_header() }

                        <main class="msger-chat" id="chat-main" ref=self.node_ref.clone()>
                        {

                            if connection_state.data_channel_state.is_some()
                                && connection_state.data_channel_state.unwrap() == RtcDataChannelState::Open
                            {
                                html! {
                                    <>
                                        { self.get_messages_as_html() }
                                    </>
                                }
                            } else if connection_state.ice_gathering_state.is_some() {
                                html! {

                                    <div class="msg right-msg">

                                        <div class="msg-bubble">
                                            <div class="msg-info">
                                            </div>

                                            <div class="msg-text">
                                                { self.get_offer_and_candidates() }
                                            </div>
                                        </div>
                                    </div>

                                }
                            } else {
                                html! {

                                <>
                                    <div class="msg right-msg">

                                        <div class="msg-bubble">
                                            <div class="msg-info">
                                            </div>

                                            <div class="msg-text">
                                                { "Paste here the offer given by your friend:" }
                                                { self.get_validate_offer_or_answer() }
                                            </div>
                                        </div>
                                    </div>

                                    <div class="msg right-msg">

                                        <div class="msg-bubble">
                                            <div class="msg-info">
                                            </div>

                                            <div class="msg-text">
                                                { "If after a while the connection cannot be establish, it is probably because there is a network issue between the 2 computers." }
                                            </div>
                                        </div>
                                    </div>
                                </>
                                }
                            }
                        }

                        </main>

                        { self.get_input_for_chat_message() }
                    </>
                }
            }
        }
    }
}

fn main() {}
