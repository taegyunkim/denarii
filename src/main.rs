mod algorithms;
mod gurobi;
mod simulator;

extern crate clap;

extern crate savefile;

use savefile::prelude::*;

#[macro_use]
extern crate savefile_derive;

use algorithms::Algorithm;
use clap::{App, Arg};
use std::collections::HashMap;
//use rand::distributions::{Bernoulli, Distribution};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Poisson};
use simulator::Packet;

const MAX_RUN: usize = 4;
const MAX_TRACE: usize = 10000;

fn main() {
    // TODO: Put this in a .yaml file
    let matches = App::new("Denarii")
        .version("0.1.0")
        .author("Taegyun Kim <k.taegyun@gmail.com>")
        .about("Discrete time based simulator for resource allocation in packet based network ddevices.")
        .arg(
            Arg::with_name("num_resources")
                .short("n_r")
                .long("num_resources")
                .default_value("2")
                .help("The number of resources on the hardware."),
        )
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
        .arg(
            Arg::with_name("distribution")
                .short("dis")
                .long("distribution")
                .default_value("1")
                .help("Poisson Distribution"), // todo: add uni, poi, bern
        )
        .arg(
            Arg::with_name("runs")
                .short("r")
                .long("runs")
                .default_value("1")
                .help("The number of runs to run the simulator."),
        )
        .arg(
            Arg::with_name("algorithm")
                .short("alg")
                .long("algorithm")
                .default_value("1")
                .help("Dominant Fairness Strategy."),
        )
        
        .get_matches();
    let num_resources = matches
        .value_of("num_resources")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let seed: u64 = matches.value_of("seed").unwrap().parse::<u64>().unwrap();
    let rng: StdRng = SeedableRng::seed_from_u64(seed);
    let ticks = matches.value_of("ticks").unwrap().parse::<usize>().unwrap();
    let runs = matches.value_of("runs").unwrap().parse::<usize>().unwrap();

    // TODO (mimee): Get string parsing to work.
    let distribution_enum = matches
        .value_of("distribution")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let algorithm_enum = matches
        .value_of("algorithm")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    // Valid checks
    if distribution_enum != 1 {
        println!("Distribution has to be 1.");
        return;
    }
    if runs > MAX_RUN {
        println!("Runs has to be at most {}", MAX_RUN);
        return;
    }
    if ticks > MAX_TRACE {
        println!("Ticks has to be at most {}.", MAX_TRACE);
        return;
    }
    if algorithm_enum != 1 {
        println!("Algorithm has to be DRF.");
        return;
    }

    // Packets not allocated
    let mut pkts: Vec<Packet> = Vec::new();
    let mut completed: Vec<Packet> = Vec::new();

    // // Use bernoulli for now.
    // let p = 0.3;
    // add a switch for different distributions

    // Distribution for packet arrivals.
    //let a_dist = Bernoulli::new(p).unwrap();

    // TODO: Provide ways to 2) randomly generate them.
    let capacity: Vec<f64> = (0..num_resources)
        .map(|x| ((x + 1) as f64) * 10.0)
        .collect();

    let alg = algorithms::Drf {};

    // parameters for the run
    let mut key = HashMap::new();
    // 1 for Poisson
    key.insert("num_resources", num_resources);
    key.insert("distribution", distribution_enum);
    key.insert("ticks", ticks);
    key.insert("runs", runs);

    // truncate to ticks
    let trace = load_trace(key, ticks as usize, rng);

    println!("{}: total number in trace", trace.len());

    // generate trace
    let mut num_pkts = 0;
    for t in 0..ticks {
        let pa: Packet = trace[t as usize].load_from_sim();

        // TODO: match on tid instead of index
        // It should be a group of packets.
        let add_new_packet: bool = !pa.is_empty();
        if !add_new_packet {
            // treat the trace as a deterministic sample.
            continue;
        }

        // TODO: Poisson could give multiple arrivals per timestep
        num_pkts += 1;
        pkts.push(pa);

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
            run_allocation(&mut pkts, t, &capacity, &alg);
        }
    }

    for pkt in &completed {
        println!("{:?} completes", pkt);
    }

    println!("{}: Total number of packets", num_pkts);
}

fn store_traces(key: HashMap<&str, usize>, traces: &Vec<Vec<Packet>>) {
    // store based on key.dist...

    // best to use parquet and interface with spark?gi
    for (i, item) in traces.iter().enumerate() {
        save_file(key_to_filename(&key, i), 0, item).unwrap();
    }
    return;
}

fn generate_trace(key: &HashMap<&str, usize>, mut rng: StdRng) -> Vec<Vec<Packet>> {
    // generate for MAX_RUN runs and MAX_TRACE ticks
    // TODO: add type "simulated" to Packet
    let ticks = MAX_TRACE;
    let runs = MAX_RUN;
    let num_resources = key["num_resources"]; // put these all in key

    // todo: make this dynamic
    // `if key.distribution = "poisson", key.lambda = 2.0`
    let poi = Poisson::new(2.0).unwrap();
    let dist = poi;
    println!("Generate trace uses key {:?}", key);

    let mut traces: Vec<Vec<Packet>> = Vec::new();
    for _ in 0..runs {
        let mut pkts: Vec<Packet> = Vec::new();
        let mut num_pkts = 0;

        for tid in 0..ticks {
            // There may be multiple at a timestep
            let add_new_packet: u64 = dist.sample(&mut rng);
            println!("{} sample", add_new_packet);

            // New Packet(s) coming
            if add_new_packet > 0 {
                for pid in 0..add_new_packet {
                    let service_time = rng.gen_range(10, 20) as f64;
                    let resource_req: Vec<f64> = (0..num_resources)
                        .map(|_| rng.gen_range(1, 11) as f64)
                        .collect();
                    println!(
                        "tid:{}, service_time:{}, resource_req:{:?}",
                        tid, service_time, resource_req
                    );

                    // ID = timestep + num of packets + packet index at its time so it is strictly increasing.
                    let pa: Packet = Packet::new(
                        tid as u64 + num_pkts + pid as u64,
                        tid as u64,
                        service_time,
                        resource_req,
                    );
                    num_pkts += 1;
                    pkts.push(pa);
                }
            } else {
                // At leasr 1 packet per tick?
                // so we ought to construct an empty packet?
                // id = tick
                let emp: Packet = Packet::new_dummy(tid as u64);
                pkts.push(emp);
            }
        }
        traces.push(pkts);
        println!("{} packets came from a Poisson(2) distribution", num_pkts);
    }
    return traces;
}

fn key_to_filename<'a>(key: &'a HashMap<&str, usize>, ind: usize) -> &'a str {
    // Must guarantee that `key` has those fields.
    // Not secure.
    let dist = key["distribution"].to_string()
        + "_"
        + &key["num_resources"].to_string()
        + "_"
        + &ind.to_string()
        + ".denarii.data";
    return Box::leak(dist.into_boxed_str());
}

fn load_trace(key: HashMap<&str, usize>, _truncate: usize, rng: StdRng) -> Vec<Packet> {
    // TODO: add a trace class.
    // TODO: finalize key's fields {runs, distributions, p, lambda, ...}
    // TODO: Load file based on key content
    // This is not very efficient.
    // With given runs, the filename param would also change.
    let traces: Vec<Vec<Packet>> = load_file(key_to_filename(&key, 0), 0).unwrap_or(Vec::new());

    // if in cache, load it, truncate and return
    if traces.len() > 0 {
        println!("Found existing trace of given key.");
        return traces[0][0.._truncate].to_vec();
    }

    // only generates when not in cache
    let gen_traces = generate_trace(&key, rng);
    println!("{} runs generated", gen_traces.len());
    println!("{} packets generated at 0th trace", gen_traces[0].len());

    // if not in cache, store it
    store_traces(key, &gen_traces);

    // truncate and return the first one, if no run param given.
    // Truncate up till arrival time < ticks, not using tid!!
    return gen_traces[0][0.._truncate].to_vec();
}

fn run_allocation(pkts: &mut [Packet], t: usize, capacity: &[f64], alg: &dyn Algorithm) {
    let mut requests: Vec<Vec<f64>> = Vec::new();
    for pkt in pkts.iter() {
        requests.push(pkt.resource_req.clone());
    }
    println!(
        "t: {}, capacity: {:?} requests: {:?}",
        t, capacity, requests
    );
    let coeffs = alg.allocate(&capacity, &requests);
    assert!(coeffs.len() == pkts.len());
    for (i, pkt) in pkts.iter_mut().enumerate() {
        let alloc = pkt.resource_req.iter().map(|x| x * coeffs[i]).collect();
        pkt.allocate(alloc);
    }
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
