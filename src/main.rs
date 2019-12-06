mod algorithms;
mod gurobi;
mod simulator;

extern crate clap;

use clap::{App, Arg};
use rand::distributions::{Bernoulli, Distribution};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// Args for the program
// n the number of packets to generate
// lambda the average packet inter arrival time (in msec?) for poisson
// distribution
// mu the average packet processing time (in msec)

fn main() {
    let matches = App::new("Denarii")
        .version("0.1.0")
        .author("Taegyun Kim <k.taegyun@gmail.com>")
        .about("Discrete time based simulator for resource allocation in packet based network ddevices.")
        .arg(
            Arg::with_name("ticks")
                .short("t")
                .long("ticks")
                .default_value("100")
                .help("The number of ticks to run the simulator."),
        )
        .arg(
            Arg::with_name("seed")
                .short("s")
                .long("seed")
                .default_value("1")
                .help("Random seed"),
        )
        .get_matches();

    let seed: u64 = matches.value_of("seed").unwrap().parse::<u64>().unwrap();
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);

    let ticks = matches.value_of("ticks").unwrap().parse::<u64>().unwrap();

    let p = 0.3;
    let dist = Bernoulli::new(p).unwrap();
    let mut cnt = 0;
    for t in 0..ticks {
        let value = dist.sample(&mut rng);
        if value {
            println!("{}: new packet arrived", t);
            cnt += 1;
        }
    }

    println!("{}: Total number of packets", cnt);
}
