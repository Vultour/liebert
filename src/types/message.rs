use std::fmt;

use std::sync;


pub enum Message{
    Data(String),
    Shutdown(String),

    Fatal(String),

    LogInfo(String),
    LogDebug(String)
}


pub type MessageSender      = sync::mpsc::Sender<Message>;
pub type MessageReceiver    = sync::mpsc::Receiver<Message>;


impl Message{
    pub fn get_type(&self) -> String{
        match self{
            &Message::Data(_)       => return String::from("Data"),
            &Message::Shutdown(_)   => return String::from("shutdown"),
            &Message::Fatal(_)      => return String::from("Fatal"),
            &Message::LogInfo(_)    => return String::from("LogInfo"),
            &Message::LogDebug(_)   => return String::from("LogDebug")
        }
    }

    pub fn get_content(&self) -> String{
        match self{
            &Message::Data(ref m)       => return m.clone(),
            &Message::Shutdown(ref m)   => return m.clone(),
            &Message::Fatal(ref m)      => return m.clone(),
            &Message::LogInfo(ref m)    => return m.clone(),
            &Message::LogDebug(ref m)   => return m.clone()
        }
    }
}

impl fmt::Display for Message{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Message [{}]: {}", self.get_type(), self.get_content())
    }
}
