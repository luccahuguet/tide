//! Simulation suite for the hit counter. The demo (`crates/tide-demo`) pairs it with one revision
//! of the service at a time (`--tide-test-library`), which the suite sees as `tide_candidate`.
use std::sim::SimRoot;
use tide_candidate::{Load, serve};

/// Every page view served is counted once, whatever order the scheduler runs the workers in.
#[sim_test(seeds = 16)]
fn every_view_is_counted(sim: SimRoot) {
    let load = Load { clients: 2, views: 1 };
    let hits = serve(&sim.tasks, &load).unwrap();
    let views = u64::from(load.clients * load.views);
    println!("views served: {views}, hits recorded: {hits}");
    assert_eq!(hits, views, "lost hits");
}
