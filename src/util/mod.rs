use std::time;
use std::thread;


pub fn wait_exec_result(wait: time::Duration, exec: &Fn() -> bool) -> Result<(), ()>{
    let start = time::Instant::now();

    while start.elapsed() < wait{
        if exec(){ return Ok(()); }
        thread::yield_now();
    }
    
    Err(())
}
