use std::str::FromStr;

use ::controller::Message;


pub struct Decoder {
    state: DecoderState
}

enum DecoderState {
    Normal,
    ProcessingFormat(vec<::controller::plugins::Format>)
}


pub fn new(pipe_tx: super::ConnectorMessageSender) -> Decoder{
    return Decoder{ state: DecoderState::Normal };
}


impl Decoder {
    pub fn decode(&self, line: &str) -> Message {
        let clean = line.trim();
        
        switch self.

        if clean.starts_with("DATA") {
            return decode_data(&clean[5..].trim())
        } else if clean.starts_with("FORMAT")

        return Message::Data("Title".into(), 123456, vec![1, 2, 3]);
    }
}


pub fn decode_data(data: &str) -> Message {
    let mut it = data.split_whitespace();

    let metric = String::from(it.next().expect("TODO: Handle invalid packet 1"));
    let timestamp = u32::from_str(it.next().expect("TODO: Handle invalid packet 2")).expect("TODO: Handle timestamp conversion failure");
    let mut data = Vec::<i64>::new();
    for data_point in it {
        data.push(i64::from_str(data_point).expect("TODO: Handle data conversion failure"));
    }

    return Message::Data(metric, timestamp, data);
}