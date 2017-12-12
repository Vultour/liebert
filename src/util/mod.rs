use std::time;
use std::thread;

use std::str::FromStr;


pub fn wait_exec_result(wait: time::Duration, exec: &Fn() -> bool) -> Result<(), ()>{
    let start = time::Instant::now();

    while start.elapsed() < wait{
        if exec(){ return Ok(()); }
        //thread::yield_now();
        thread::sleep(time::Duration::from_millis(100));
    }
    
    Err(())
}


pub fn rrd_from_str(token: &str) -> Option<i64> {
    let token = token.trim();

    if token == "U" {
        return None;
    }

    return Some(i64::from_str(token).expect("Couldn't convert RRD string token into proper internal representation"));
}

pub fn rrd_to_string(token: &Option<i64>) -> String {
    return match token {
        &Some(x) => x.to_string(),
        &None    => String::from("U")
    };
}



#[test]
fn t_rrd_from_str() {
    assert_eq!(Some(0), rrd_from_str("0"));
    assert_eq!(Some(1), rrd_from_str("1"));
    assert_eq!(Some(99999999999), rrd_from_str("99999999999"));
    assert_eq!(None, rrd_from_str("U"));
}

#[test]
fn t_rrd_to_string() {
    // STUB
}
