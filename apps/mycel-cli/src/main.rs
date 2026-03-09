use std::env;

use mycel_core::workspace_banner;
use mycel_sim::manifest::SimulatorPaths;
use mycel_sim::simulator_banner;

fn print_usage() {
    println!("mycel <command>");
    println!();
    println!("Commands:");
    println!("  info       Show workspace and simulator scaffold information");
    println!("  help       Show this message");
    println!();
    println!("Planned next commands:");
    println!("  validate   Validate fixture/topology/test/report inputs");
    println!("  sim        Run a simulator test case");
}

fn print_info() {
    let paths = SimulatorPaths::default();

    println!("{}", workspace_banner());
    println!("{}", simulator_banner());
    println!("fixtures: {}", paths.fixtures_root);
    println!("peers: {}", paths.peers_root);
    println!("topologies: {}", paths.topologies_root);
    println!("tests: {}", paths.tests_root);
    println!("reports: {}", paths.reports_root);
}

fn main() {
    let mut args = env::args().skip(1);

    match args.next().as_deref() {
        Some("info") => print_info(),
        Some("help") | None => print_usage(),
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!();
            print_usage();
            std::process::exit(2);
        }
    }
}
