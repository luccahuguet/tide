//@ rules: PROC-1, PROC-2, CAP-1, ROOT-3
// One process contract runs on real Linux (`main`) and under the simulator, whose only test
// adapter is model setup; the build test compares the observations. Real checks never use PIDs
// or timing: the one early `try_wait` names a 30-second sleeper.
use std::io::{Read, Write};
use std::process::{Proc, Stdio};

fn case(proc: &Proc) -> Vec<String> {
    let mut out = Vec::new();
    let s = proc.command("true").stdin(Stdio::null()).status().unwrap();
    out.push(format!("true: {:?} {}", s.code(), s.success()));
    let s = proc.command("sh").args(["-c", "exit 3"]).stdin(Stdio::null()).status().unwrap();
    out.push(format!("sh: {:?} {} {s}", s.code(), s.success()));
    let o = proc.command("echo").arg("hi").output().unwrap();
    out.push(format!("echo: {:?} {:?} {:?}", String::from_utf8_lossy(&o.stdout), o.stderr, o.status.code()));
    let mut cat = proc.command("cat").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    cat.stdin.take().unwrap().write_all(b"piped").unwrap();
    let mut text = String::new();
    cat.stdout.take().unwrap().read_to_string(&mut text).unwrap();
    out.push(format!("cat: {text} {:?}", cat.wait().unwrap().code()));
    // A public pipe field moved to another child: wait_with_output reads the moved end.
    let mut a = proc.command("echo").arg("hi").stdin(Stdio::null()).stdout(Stdio::piped()).spawn().unwrap();
    let mut b = proc.command("printf").arg("b").stdin(Stdio::null()).stdout(Stdio::piped()).spawn().unwrap();
    // Both exit first, their output still in the pipes, so dropping A's own end changes nothing.
    let (sa, sb) = (a.wait().unwrap(), b.wait().unwrap());
    a.stdout = b.stdout.take();
    let o = a.wait_with_output().unwrap();
    out.push(format!("moved: {:?} {} {:?}", String::from_utf8_lossy(&o.stdout), o.status == sa, sb.code()));
    let mut sleeper = proc.command("sleep").arg("30").stdin(Stdio::null()).spawn().unwrap();
    let running = sleeper.try_wait().unwrap().is_none();
    sleeper.kill().unwrap();
    let s = sleeper.wait().unwrap();
    out.push(format!("kill: {running} {:?} {s} {:?}", s.code(), sleeper.wait().unwrap() == s));
    out.push(format!("missing: {:?}", proc.command("tide-missing-program").output().unwrap_err().kind()));
    let only = proc.only(["true", "./true", ""]).only(["true", "sh"]);
    out.push(format!("only: {:?}", only.command("sh").output().unwrap_err().kind()));
    out.push(format!("only path: {:?}", only.command("/usr/bin/true").output().unwrap_err().kind()));
    let denied = proc.only(["/bin/true"]).command("/usr/bin/true").output().unwrap_err().kind();
    assert_eq!(denied, std::io::ErrorKind::PermissionDenied);
    out.push(format!("only absolute path: {denied:?}"));
    out.push(format!("loader: {:?}", only.command("true").env("LD_PRELOAD", "x").output().unwrap_err().kind()));
    out.push(format!("nul: {:?}", only.command("true").arg("a\0").output().unwrap_err().kind()));
    // A command's own PATH reaches only the child; it never changes which program starts.
    let s = only.command("true").env("PATH", "/nonexistent").stdin(Stdio::null()).status().unwrap();
    out.push(format!("command PATH: {}", s.success()));
    out
}

#[tide::main]
fn main(root: Root) {
    println!("OBS:{:?}", case(&root.proc));
}

#[cfg(test)]
mod simulated {
    use super::*;
    use std::sim::{ProcessModel as M, SimRoot};
    use std::time::Duration;

    fn world(sim: &SimRoot) {
        sim.world.proc("true", M::new());
        sim.world.proc("sh", M::new().args(["-c", "exit 3"]).exit(3));
        sim.world.proc("echo", M::new().args(["hi"]).write("hi\n"));
        sim.world.proc("printf", M::new().args(["b"]).write("b"));
        sim.world.proc("cat", M::new().copy());
        sim.world.proc("sleep", M::new().args(["30"]).delay(Duration::from_secs(30), Duration::from_secs(30)));
        sim.world.proc("tide-missing-program", M::missing());
    }

    #[test]
    fn equivalent_program() {
        let observations: Vec<_> = (0..20)
            .map(|seed| {
                let sim = Root::sim(seed);
                world(&sim);
                case(&sim.proc)
            })
            .collect();
        assert!(observations.iter().all(|o| *o == observations[0]));
        println!("OBS:{:?}", observations[0]);
    }

    fn trace(seed: u64) -> String {
        let sim = Root::sim(seed);
        world(&sim);
        case(&sim.proc);
        sim.trace()
    }

    #[test]
    fn fixed_seed_trace() {
        assert_eq!(trace(7), trace(7));
    }

    #[test]
    #[should_panic(expected = "unmodelled effect")]
    fn an_unregistered_program_is_a_coverage_failure() {
        let sim = Root::sim(0);
        let _ = sim.proc.command("git").stdin(Stdio::null()).status();
    }
}
