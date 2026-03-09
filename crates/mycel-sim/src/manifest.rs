//! Language-neutral scaffold mapping for the simulator directories.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulatorPaths {
    pub fixtures_root: &'static str,
    pub peers_root: &'static str,
    pub topologies_root: &'static str,
    pub tests_root: &'static str,
    pub reports_root: &'static str,
}

impl Default for SimulatorPaths {
    fn default() -> Self {
        Self {
            fixtures_root: "fixtures/object-sets",
            peers_root: "sim/peers",
            topologies_root: "sim/topologies",
            tests_root: "sim/tests",
            reports_root: "sim/reports",
        }
    }
}
