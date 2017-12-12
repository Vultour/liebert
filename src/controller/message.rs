use std::fmt;
use std::sync;

use ::types;


pub type MessageSender = sync::mpsc::Sender<Message>;

pub enum Message {
    Data(String, String, u32, Vec<i64>),
    Format(String, String, Vec<::types::MetricFormat>),

    Shutdown(String)
}

impl Message{
    pub fn get_type(&self) -> String{
        match self{
            &Message::Data(_, _, _, _) => return String::from("Data"),
            &Message::Format(_, _, _)  => return String::from("Format"),
            &Message::Shutdown(_)      => return String::from("Shutdown"),
        }
    }

    pub fn get_content(&self) -> String{
        match self{
            &Message::Data(ref h, ref n, ref t, ref m) => return format!("Data for {}-{} [{}]", h, n, t),
            &Message::Format(ref h, ref n, ref m)      => return format!("Format for {}-{}, length {}", h, n, m.len()),
            &Message::Shutdown(ref m)       => return m.clone(),
        }
    }
}

impl Clone for Message {
    fn clone(&self) -> Message {
        match self {
            &Message::Data(ref h, ref n, ref t, ref m)  => Message::Data(h.clone(), n.clone(), t.to_owned(), m.clone()),
            &Message::Format(ref h, ref m, ref f)       => Message::Format(h.clone(), m.clone(), f.clone()),
            &Message::Shutdown(ref m)                   => Message::Shutdown(m.clone())
        }
    }
}

impl fmt::Display for Message{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Message [{}]: {}", self.get_type(), self.get_content())
    }
}