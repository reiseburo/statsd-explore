/**
 * A simple statsd explorer for dumping statsd output to the console
 */
extern crate async_std;
extern crate statsd_parser;

use async_std::task;
use async_std::net::UdpSocket;
use log::*;
use statsd_parser::*;
use std::collections::HashMap;

fn print_metric(metric: Message, counters: &mut HashMap<String, f64>) {
    match metric.metric {
        Metric::Counter(c) => {
            let mut actual = c.value;

            if counters.contains_key(&metric.name) {
                actual = actual + counters.get(&metric.name).unwrap();
            }

            println!("📈 +{} => {}\t{}", c.value, actual, metric.name);
            counters.insert(metric.name, actual);
        },
        Metric::Gauge(g) => {
            println!("📏 {}\t\t{}", g.value, metric.name);
        },
        Metric::Timing(t) => {
            println!("⏱  {}ms\t\t{}", t.value, metric.name);
        },
        Metric::Histogram(h) => {
            println!("📊 {}\t\t{}", h.value, metric.name);
        },
        _ => {
        },
    }
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    task::block_on(async {
        info!("Listening on port 8125");
        let socket = UdpSocket::bind("127.0.0.1:8125").await?;

        let mut counters = HashMap::<String, f64>::new();

        loop {
            let mut buf = [0; 1024];
            let (amt, _src) = socket.recv_from(&mut buf).await?;

            debug!("Received {} bytes", amt);

            if let Ok(metric_str) = String::from_utf8(buf.to_vec()) {
                let m = metric_str.trim_matches(char::from(0));
                if let Ok(metric) = statsd_parser::parse(m) {
                    print_metric(metric, &mut counters);
                }
                else {
                    error!("Could not parse: {}", metric_str);
                }
            }
            else {
                error!("Unable to coerce bytes to UTF-8");
            }
        }
    })
}
