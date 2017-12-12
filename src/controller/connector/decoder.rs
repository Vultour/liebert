use std::mem;

use std::str::FromStr;

use ::controller::Message;
use ::types;


pub struct Decoder {
    state:  DecoderState,
    host:   String,
    pipe:   super::ConnectorMessageSender,
}

enum DecoderState {
    Normal,
    ProcessingFormat(String, Vec<::types::MetricFormat>)
}


impl Decoder {
    pub fn new(pipe_tx: super::ConnectorMessageSender, host: String) -> Decoder{
        return Decoder {
            state: DecoderState::Normal,
            host: host,
            pipe: pipe_tx
        };
    }

    pub fn decode(&mut self, line: &str) -> Message {
        let line = line.trim();
        
        match self.state {
            DecoderState::Normal => self.process_normal(line),
            DecoderState::ProcessingFormat(_, _) => self.process_format(line)
        }

        

        return Message::Data("0.0.0.0".into(), "Title".into(), 123456, vec![1, 2, 3]);
    }
}

impl Decoder {
    fn process_normal(&mut self, line: &str) {
        trace!("process_normal");
        if line.starts_with("DATA") {
            self.pipe.send(decode_data(&line[5..].trim(), &self.host)).expect("FATAL ERROR: Decoder couldn't send on channel");
        }
        else if line.starts_with("FORMAT") {
            self.state = DecoderState::ProcessingFormat(
                line.split_whitespace().last().expect("FATAL ERROR: Couldn't decode metric ID from FORMAT string").into(),
                Vec::new()
            );
        }
    }

    fn process_format(&mut self, line: &str) {
        trace!("process_format");
        if (line.starts_with("FORMAT_END")) {
            match self.state {
                DecoderState::ProcessingFormat(ref mut n, ref mut f) => {
                    self.pipe.send(::controller::Message::Format(
                        self.host.clone(),
                        mem::replace(n, String::new()),
                        mem::replace(f, Vec::new())
                    ));
                },
                _ => { panic!("Called process_format when not in a format processing state"); }
            }
            trace!("Finished format sequence");
            self.state = DecoderState::Normal;
            return;
        }

        trace!("Processing format message - '{}'", line);
        let mut it = line.split_whitespace();

        let next = it.next().expect("Couldn't extract value type from message");
        let kind = u8::from_str(next).expect(&format!("Couldn't convert format type into integer - {}", next));
        let name = it.next().expect("FATAL ERROR: Couldn't extract metric name from message").to_string();
        let hb = u32::from_str(it.next().expect("Couldn't extract heartbeat from message")).expect("Couldn't convert heartbeat into an integer");
        let min = ::util::rrd_from_str(it.next().expect("Couldn't extract format min from message"));
        let max = ::util::rrd_from_str(it.next().expect("Couldn't extract format max from message"));

        match self.state {
            DecoderState::ProcessingFormat(_, ref mut f) => {
                f.push(::types::MetricFormat::from_id(kind, name, hb, min, max));
            },
            _ => { panic!("Called process_format when not in format state (2)"); }
        }
    }
}


pub fn decode_data(data: &str, host: &str) -> Message {
    let mut it = data.split_whitespace();

    let metric = String::from(it.next().expect("TODO: Handle invalid message 1"));
    let timestamp = u32::from_str(it.next().expect("TODO: Handle invalid message 2")).expect("TODO: Handle timestamp conversion failure");
    let mut data = Vec::<i64>::new();
    for data_point in it {
        data.push(i64::from_str(data_point).expect("TODO: Handle data conversion failure"));
    }

    return Message::Data(host.into(), metric, timestamp, data);
}