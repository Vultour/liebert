use std::fmt;

use std::sync;


pub enum Message{
    Data(String, i64, String),
    Format(String, Vec<::types::MetricFormat>),
    Shutdown(String),

    Fatal(String),

    LogError(String),
    LogWarn(String),
    LogInfo(String),
    LogDebug(String)
}


pub type MessageSender      = sync::mpsc::Sender<Message>;
pub type MessageReceiver    = sync::mpsc::Receiver<Message>;


impl Message{
    pub fn get_type(&self) -> String{
        match self{
            &Message::Data(_, _, _) => return String::from("Data"),
            &Message::Format(_, _)  => return String::from("Format"),
            &Message::Shutdown(_)   => return String::from("shutdown"),
            &Message::Fatal(_)      => return String::from("Fatal"),
            &Message::LogError(_)   => return String::from("LogError"),
            &Message::LogWarn(_)    => return String::from("LogWarn"),
            &Message::LogInfo(_)    => return String::from("LogInfo"),
            &Message::LogDebug(_)   => return String::from("LogDebug")
        }
    }

    pub fn get_content(&self) -> String{
        match self{
            &Message::Data(ref n, ref t, ref m) => return format!("Data for {} [{}]: {}", n, t, m),
            &Message::Format(ref n, ref m)      => return format!("Format for {}, length {}", n, m.len()),
            &Message::Shutdown(ref m)           => return m.clone(),
            &Message::Fatal(ref m)              => return m.clone(),
            &Message::LogError(ref m)           => return m.clone(),
            &Message::LogWarn(ref m)            => return m.clone(),
            &Message::LogInfo(ref m)            => return m.clone(),
            &Message::LogDebug(ref m)           => return m.clone()
        }
    }
}

impl fmt::Display for Message{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Message [{}]: {}", self.get_type(), self.get_content())
    }
}
