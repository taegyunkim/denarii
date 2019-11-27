pub use self::asset_fairness::*;
pub use self::ceei::*;
pub use self::drf::*;
mod asset_fairness;
mod ceei;
mod drf;

pub trait Algorithm {
    fn allocate(&self, resources: &[f64], demands: &[Vec<f64>]) -> Vec<f64>;
}

