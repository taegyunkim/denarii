mod algorithms;
mod gurobi;
mod simulator;

extern crate clap;

use algorithms::Algorithm;
use clap::{App, Arg};
use rand::distributions::{Bernoulli, Distribution};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use simulator::Packet;

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

    // Packets not allocated
    let mut pkts: Vec<Packet> = Vec::new();
    let mut completed: Vec<Packet> = Vec::new();

    let p = 0.3;
    // Distribution for packet arrivals.
    let a_dist = Bernoulli::new(p).unwrap();
    let num_resources = 2;

    // TODO: Provide ways to set initial capacity of resources, 1) commandline
    // argument, 2) randomly generate them.
    let capacity: Vec<f64> = (0..num_resources)
        .map(|x| ((x + 1) as f64) * 10.0)
        .collect();

    let alg = algorithms::Drf {};

    let mut num_pkts = 0;
    for t in 0..ticks {
        let add_new_packet: bool = a_dist.sample(&mut rng);
        // New Packet coming
        if add_new_packet {
            let service_time = rng.gen_range(10, 20) as f64;
            let resource_req: Vec<f64> = (0..num_resources)
                .map(|_| rng.gen_range(1, 11) as f64)
                .collect();
            println!(
                "t:{}, service_time:{}, resource_req:{:?}",
                t, service_time, resource_req
            );

            let p: Packet = Packet::new(num_pkts, t, service_time, resource_req);
            num_pkts += 1;
            pkts.push(p);
        }

        // Step each packet.
        let mut done_pkts = 0;
        for pkt in &mut pkts {
            let done = pkt.step();

            if done {
                // TODO: Move instead of copy.
                completed.push(pkt.clone());
                done_pkts += 1;
            }
        }

        // Remove packets that are completed.
        pkts.retain(|pkt| !pkt.is_completed());

        // Check whether a new allocation needs to happen
        if !pkts.is_empty() && (add_new_packet || done_pkts > 0) {
            let mut requests: Vec<Vec<f64>> = Vec::new();
            for pkt in &mut pkts {
                requests.push(pkt.resource_req.clone());
            }

            println!(
                "t: {}, capacity: {:?} requests: {:?}",
                t, capacity, requests
            );
            let coeffs = alg.allocate(&capacity, &requests);
            assert!(coeffs.len() == pkts.len());
            for i in 0..pkts.len() {
                let coeff = coeffs[i];
                let pkt = &mut pkts[i];
                let alloc = pkt.resource_req.iter().map(|x| x * coeff).collect();
                pkt.allocate(alloc);
            }
        }
    }

    for pkt in &completed {
        println!("{:?}", pkt);
    }

    println!("{}: Total number of packets", num_pkts);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conatiner_ops() {
        let mut pkts: Vec<Packet> = Vec::new();
        pkts.push(Packet::new(1, 1, 2.0, vec![1.0, 2.0]));
        pkts.retain(|pkt| pkt.is_scheduled());
        assert!(pkts.is_empty());
    }
}
