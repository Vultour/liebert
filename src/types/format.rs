pub type MetricFormatVec = Vec<MetricFormat>;


pub enum MetricFormat {
    Gauge(String, u32, Option<i64>, Option<i64>),  // Name, heartbeat, min, max
    Counter(String, u32, Option<i64>, Option<i64>) // Name, heartbeat, min, max
}


impl MetricFormat {
    pub fn to_id(&self) -> u8 {
        return match self {
            &MetricFormat::Gauge(_, _, _, _)     => 1,
            &MetricFormat::Counter(_, _, _, _)   => 2
        };
    }

    pub fn from_id(id: u8, name: String, heartbeat: u32, min: Option<i64>, max: Option<i64>) -> MetricFormat {
        return match id {
            1 => MetricFormat::Gauge(name, heartbeat, min, max),
            2 => MetricFormat::Counter(name, heartbeat, min, max),
            _ => panic!("FATAL ERROR: [Bug] Tried to create MetricFormat from an unknown id")
        };
    }
}

impl Clone for MetricFormat {
    fn clone(&self) -> MetricFormat {
        match self {
            &MetricFormat::Gauge(ref n, ref hb, ref min, ref max) => MetricFormat::Gauge(n.clone(), hb.to_owned(), min.clone(), max.clone()),
            &MetricFormat::Counter(ref n, ref hb, ref min, ref max) => MetricFormat::Counter(n.clone(), hb.to_owned(), min.clone(), max.clone())
        }
    }
}

impl PartialEq for MetricFormat {
    fn eq(&self, other: &MetricFormat) -> bool {
        if self.to_id() != other.to_id() { return false; }

        match (self, other) {
            (&MetricFormat::Gauge(ref sn, ref shb, ref smin, ref smax), &MetricFormat::Gauge(ref on, ref ohb, ref omin, ref omax)) => {
                if (sn != on) { return false; }
                if (shb != ohb) { return false; }
                if (smin != omin) { return false; }
                if (smax != omax) { return false; }
                return true;
            },
            (&MetricFormat::Counter(ref sn, ref shb, ref smin, ref smax), &MetricFormat::Counter(ref on, ref ohb, ref omin, ref omax)) => {
                if (sn != on) { return false; }
                if (shb != ohb) { return false; }
                if (smin != omin) { return false; }
                if (smax != omax) { return false; }
                return true;
            },
            _ => { return false; }
        };
    }
}

impl Eq for MetricFormat { }