use std::fmt;
use std::sync;


pub type MessageSender = sync::mpsc::Sender<Message>;

pub enum Message {
    Data(String, u32, Vec<i64>),
    Format(String, Vec<super::plugins::Format>),

    Shutdown(String)
}

impl Message{
    pub fn get_type(&self) -> String{
        match self{
            &Message::Data(_, _, _) => return String::from("Data"),
            &Message::Format(_, _)  => return String::from("Format"),
            &Message::Shutdown(_)   => return String::from("Shutdown"),
        }
    }

    pub fn get_content(&self) -> String{
        match self{
            &Message::Data(ref n, ref t, ref m) => return format!("Data for {} [{}]", n, t),
            &Message::Format(ref n, ref m)      => return format!("Format for {}, length {}", n, m.len()),
            &Message::Shutdown(ref m)           => return m.clone(),
        }
    }
}

impl fmt::Display for Message{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f, "Message [{}]: {}", self.get_type(), self.get_content())
    }
}