#[derive(Clone, Debug, Default, Savefile)]
pub struct Packet {
    /// Packet ID
    id: u64,
    /// Time t, the packet showed up.
    t_arrival: u64,
    /// Time t, the packet left the hardware
    t_departure: u64,
    /// Resources requested
    pub resource_req: Vec<f64>,
    /// Resources actually allocated, empty if none.
    resource_alloc: Vec<f64>,
    /// Number of ticks it needs to complete, given requested resources.
    service_time: f64,
    /// Actual service time it has gotten so far.
    adjusted_service_time: f64,

    // Dummy packet, default to false
    is_empty: bool,
}

impl Packet {
    pub fn new(id: u64, t_arrival: u64, service_time: f64, resource_req: Vec<f64>) -> Packet {
        Packet {
            id,
            t_arrival,
            resource_req,
            service_time,
            ..Default::default()
        }
    }

    pub fn new_dummy(id: u64) -> Packet {
        Packet {
            id,
            is_empty: true,
            ..Default::default()
        }
    }
    /// Steps one tick.
    pub fn step(&mut self) -> bool {
        if !self.is_scheduled() {
            return false;
        }

        // Update adjusted service time by the fraction of resources it got.
        let ratio = self.resource_alloc[0] / self.resource_req[0];
        self.adjusted_service_time += ratio;

        // Also update t_departure by 1.
        self.t_departure += 1;

        self.is_completed()
    }

    pub fn load_from_sim(&self) -> Packet {
        let mut pa = Packet::new(
            self.id,
            self.t_arrival,
            self.service_time,
            self.resource_req.to_vec(),
        );
        pa.is_empty = self.is_empty();
        return pa;
    }
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    pub fn is_completed(&self) -> bool {
        self.adjusted_service_time >= self.service_time
    }

    pub fn is_scheduled(&self) -> bool {
        !self.resource_alloc.is_empty()
    }

    pub fn allocate(&mut self, alloc: Vec<f64>) {
        self.resource_alloc = alloc;
    }

    #[allow(dead_code)]
    /// Returns the number of ticks it actually took to service this packet.
    /// Make sure you check whether this packet is completed, using
    /// is_completed().
    pub fn latency(&self) -> u64 {
        self.t_departure - self.t_arrival
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut p = Packet::new(1, 3, 5.0, vec![2.0, 3.0]);
        assert_eq!(p.id, 1);
        assert_eq!(p.t_arrival, 3);
        assert_eq!(p.service_time, 5.0);
        assert_eq!(p.resource_req, [2.0, 3.0]);
        assert_eq!(p.is_completed(), false);
        assert_eq!(p.is_scheduled(), false);
        p.allocate(vec![1.0, 1.5]);
        assert!(p.is_scheduled());
        // Service time is set to 5 and it was allocated with half of what it
        // requested. So needs 5 * 2 ticks to complete.
        for _ in 0..10 {
            p.step();
        }
        assert!(p.is_completed());
    }
}
