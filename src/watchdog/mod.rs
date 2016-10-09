use std::io;
use std::thread;
use std::sync;

use super::types;


pub struct Watchdog{
    mon_handles:    Vec<thread::JoinHandle<()>>,
    spawned:        Vec<thread::JoinHandle<()>>,
    channel_out:    sync::mpsc::Sender<types::Message>
}


impl Watchdog{
    pub fn new(tx: sync::mpsc::Sender<types::Message>) -> Watchdog{
        Watchdog{
            mon_handles:    Vec::new(),
            spawned:        Vec::new(),
            channel_out:    tx
        }
    }

    pub fn monitor(&mut self) -> Result<(), io::Error>{
        loop{
            let handle: thread::JoinHandle<()>;
            let local_tx = self.channel_out.clone();

            match self.mon_handles.pop(){
                Some(h) => { handle = h; }
                None    => { break; }
            }

            debug!("Watchdog spawning monitor thread for {}", handle.thread().name().unwrap_or("unknown"));
            match thread::Builder::new()
                .name(format!("monitor_{}", handle.thread().name().unwrap_or("unknown")))
                .spawn(
                    || {
                        let h: thread::JoinHandle<()> = handle;
                        let tx: sync::mpsc::Sender<types::Message> = local_tx;
                        let thread_name = format!("{}", h.thread().name().unwrap_or("unknown"));
                        debug!("DEBUG: Watchdog started monitoring thread {}", thread_name);
                        match h.join(){
                            Ok(_)   => { debug!("WATCHDOG: Thread {} terminated gracefully", thread_name); }
                            Err(_)  => { tx.send(types::Message::Fatal(format!("WATCHDOG: Thread {} crashed", thread_name))).expect("FATAL ERROR: [BUG] Control channel is closed"); }
                        }
                    }
            ){
                Ok(h)   => { self.spawned.push(h); }
                Err(e)  => { return Err(e); }
            }
        }

        Ok::<(), io::Error>(())
    }

    pub fn join(&mut self) -> Result<(), String>{
        loop{
            let handle: thread::JoinHandle<()>;

            match self.spawned.pop(){
                Some(h) => { handle = h; }
                None    => { break; }
            }


            let thread_name = format!("{}", handle.thread().name().unwrap_or("unknown"));
            debug!("Watchdog waiting for shutdown of {}", thread_name);

            match handle.join(){
                Ok(_)   => {  }
                Err(e)  => { panic!("FATAL ERROR: [BUG] Watchdog thread {} has crashed. -- {:?}", thread_name, e); }
            }
        }

        Ok(())
    }
}

impl Watchdog{
    pub fn watch(&mut self, handle: thread::JoinHandle<()>) -> &mut Watchdog{
        self.mon_handles.push(handle);
        self
    }
}
