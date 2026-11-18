//! Private wire contract shared by the trusted driver and harness, never a candidate API.
#![allow(dead_code)] // Each consumer uses a different half of this private protocol.
use crate::__codec::{Decode, Decoder, Encode, Encoder};
use std::collections::BTreeMap;

pub(crate) const FORMAT: &str = "tide-replay/2";
/// The format before fx-qlek.14, whose identity hashed every file in each dependency directory.
const FORMAT_ALL_FILES: &str = "tide-replay/1";
pub(crate) const VERSIONS: &str = "trace/1;generator/1;parameters=splitmix64(seed^6a09e667f3bcc909);scheduler=seed^7469646573636864;model/1";
pub(crate) const CONTEXT: &str = "__TIDE_REPLAY_CONTEXT";
pub(crate) const CONTROLS: &[&str] = &[CONTEXT, "TIDE_SIM_SEED", "TIDE_TEST_SEED"];

// Reuse Tide's strict JSON reader (including duplicate-key detection), without deriving
// against the injected std or adding a second serializer to the toolchain.
macro_rules! object {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub(crate) struct $name { $(pub(crate) $field: $ty),* }
        impl Encode for $name {
            fn __encode(&self, out: &mut Encoder) {
                out.object_begin(); $(out.field(stringify!($field), &self.$field);)* out.object_end();
            }
        }
        impl Decode for $name {
            fn __decode(input: &mut Decoder<'_>) -> Result<Self, crate::json::Error> {
                $(let mut $field: Option<$ty> = None;)*
                input.object_begin()?;
                while let Some(index) = input.next_field(&[$(stringify!($field)),*])? {
                    let name = [$(stringify!($field)),*].get(index).copied()
                        .ok_or_else(|| crate::json::Error::new("unknown replay field".into(), 1, 1))?;
                    match name { $(stringify!($field) => input.field(&mut $field, name)?,)* _ => unreachable!() }
                }
                Ok(Self { $($field: input.required($field, stringify!($field))?),* })
            }
        }
    };
}

object!(Build {
    source: BTreeMap<String, String>,
    library: BTreeMap<String, String>,
    dependencies: Vec<BTreeMap<String, String>>,
    compiler: String,
    target: String,
    profile: String,
    options: Vec<String>,
    driver: String,
    versions: String,
});
object!(Receipt { format: String, build: Build, executable: String, tests: BTreeMap<String, BTreeMap<String, String>> });
object!(Choice {
    kind: String,
    bound: String,
    value: String
});
object!(Input { kind: String, key: Vec<u8>, value: Vec<u8> });
object!(RootRun { ordinal: u64, seed: String, inputs: Vec<Input> });
object!(Manifest {
    format: String,
    build: Build,
    test: String,
    evaluation: u64,
    seed: String,
    config: BTreeMap<String, String>,
    choices: Vec<Choice>,
    roots: Vec<RootRun>,
    injected: Vec<String>,
    complete: bool,
    outcome: String,
    trace: String,
    trace_hash: String,
    input_hash: String,
});
object!(Context {
    receipt: Receipt,
    directory: String,
    completion: String,
    published: String,
    run: String,
    command: Vec<String>,
    replay: Option<Manifest>,
});

pub(crate) fn encoded<T: Encode>(value: &T) -> String {
    String::from_utf8(crate::json::encode(value)).unwrap()
}
pub(crate) fn decoded<T: Decode>(bytes: &[u8]) -> Result<T, String> {
    crate::json::decode(bytes).map_err(|e| format!("replay JSON: {e}"))
}
pub(crate) fn read<T: Decode>(path: &std::path::Path) -> Result<T, String> {
    decoded(&std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?)
}
pub(crate) fn configuration(mut fields: BTreeMap<String, String>) -> BTreeMap<String, String> {
    fields.extend([
        ("scheduler".into(), "seeded-baton/1".into()),
        ("root_initial_args".into(), "[]".into()),
        ("root_initial_environment".into(), "{}".into()),
        ("root_initial_files".into(), "{}".into()),
        ("root_initial_stdin".into(), "[]".into()),
        ("setup".into(), "matching-source;no-external-inputs".into()),
    ]);
    fields
}
impl Build {
    /// REPLAY-2's refusal: the first component that differs, with one short line per changed,
    /// missing or added entry (hashes cut to 12 digits), at most `DIFF_LINES` of them.
    pub(crate) fn compare(&self, current: &Self) -> Result<(), String> {
        const DIFF_LINES: usize = 8;
        let mut lines = Vec::new();
        let scalar = |lines: &mut Vec<String>, recorded: &str, current: &str| {
            lines.push(format!("recorded {:?}, current {:?}", short(recorded), short(current)));
        };
        let label = if self.source != current.source {
            entries(&mut lines, &self.source, &current.source);
            "source"
        } else if self.library != current.library {
            entries(&mut lines, &self.library, &current.library);
            "paired-library source"
        } else if self.compiler != current.compiler {
            scalar(&mut lines, &self.compiler, &current.compiler);
            "compiler"
        } else if self.target != current.target {
            scalar(&mut lines, &self.target, &current.target);
            "target"
        } else if self.profile != current.profile {
            scalar(&mut lines, &self.profile, &current.profile);
            "profile"
        } else if self.options != current.options {
            let sides = [("recorded", &self.options, &current.options), ("current", &current.options, &self.options)];
            for (side, from, to) in sides {
                let only = from.iter().filter(|o| !to.contains(o));
                lines.extend(only.map(|o| format!("only {side}: {o:?}")));
            }
            if lines.is_empty() {
                lines.push("the same options in a different order or number".into());
            }
            "options"
        } else if self.dependencies != current.dependencies {
            let empty = BTreeMap::new();
            for i in 0..self.dependencies.len().max(current.dependencies.len()) {
                let side = |build: &Self| build.dependencies.get(i).unwrap_or(&empty).clone();
                entries(&mut lines, &side(self), &side(current));
            }
            "dependency artifact"
        } else if self.driver != current.driver {
            scalar(&mut lines, &self.driver, &current.driver);
            "tide-admit"
        } else if self.versions != current.versions {
            scalar(&mut lines, &self.versions, &current.versions);
            "configuration/format"
        } else {
            return Ok(());
        };
        let more = lines.len().saturating_sub(DIFF_LINES);
        lines.truncate(DIFF_LINES);
        if more > 0 {
            lines.push(format!("and {more} more"));
        }
        Err(format!(
            "replay {label} mismatch; use fresh --seed for changed inputs\n  {}",
            lines.join("\n  ")
        ))
    }
}
/// One line per key whose value differs between a recorded and a current map.
fn entries(lines: &mut Vec<String>, recorded: &BTreeMap<String, String>, current: &BTreeMap<String, String>) {
    let mut keys: Vec<&String> = recorded.keys().chain(current.keys()).collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        let value = |map: &BTreeMap<String, String>| map.get(key).map_or("missing".into(), |v| shown(&short(v)));
        if recorded.get(key) != current.get(key) {
            let (old, new) = (value(recorded), value(current));
            lines.push(format!("{}: recorded {old}, current {new}", shown(key)));
        }
    }
}
/// Text from a recording, or a path built from one, escaped for the terminal (RUN-6): control
/// characters, bidirectional overrides and other non-printable characters print as escapes.
pub(crate) fn shown(text: &str) -> String {
    text.escape_debug().to_string()
}
/// A value with its trailing SHA-256 hex digest cut to 12 digits.
fn short(value: &str) -> String {
    let split = value.len().saturating_sub(64);
    match value.get(split..) {
        Some(digest) if value.len() >= 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()) => {
            format!("{}{}", &value[..split], &digest[..12])
        }
        _ => value.into(),
    }
}
impl Manifest {
    pub(crate) fn seal(&mut self) {
        self.trace_hash = digest(self.trace.as_bytes());
        self.input_hash.clear();
        self.input_hash = digest(encoded(self).as_bytes());
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.format == FORMAT_ALL_FILES {
            return Err(format!(
                "replay format mismatch: this recording is {FORMAT_ALL_FILES}, which this driver no longer replays; re-record it with a fresh run of the same test with --seed {}, then replay the new manifest",
                // A recorded string reaches the terminal only as a canonical decimal.
                decimal::<u64>(&self.seed).map_or_else(|_| "N".into(), |seed| seed.to_string())
            ));
        }
        if self.format != FORMAT {
            return Err(format!("replay format mismatch (expected {FORMAT})"));
        }
        if self.build.versions != VERSIONS {
            return Err("replay configuration/format mismatch".into());
        }
        if !self.injected.is_empty() {
            return Err("replay configuration: external injected inputs are unsupported".into());
        }
        decimal::<u64>(&self.seed)?;
        for (index, root) in self.roots.iter().enumerate() {
            if root.ordinal != index as u64 {
                return Err("replay root ordinal mismatch".into());
            }
            decimal::<u64>(&root.seed)?;
            for input in &root.inputs {
                if !matches!(
                    input.kind.as_str(),
                    "arg" | "environment" | "stdin" | "file" | "proc"
                ) {
                    return Err("replay configuration: unsupported root input".into());
                }
            }
        }
        for choice in &self.choices {
            choice.validate()?;
        }
        if self.trace_hash != digest(self.trace.as_bytes()) {
            return Err("replay trace digest mismatch".into());
        }
        let mut copy = self.clone();
        copy.seal();
        if self.input_hash != copy.input_hash {
            return Err("replay configuration/input digest mismatch".into());
        }
        if !self.complete {
            return Err("replay recording is incomplete; use fresh --seed".into());
        }
        Ok(())
    }
}
pub(crate) fn decimal<T: std::str::FromStr + ToString>(value: &str) -> Result<T, String> {
    value
        .parse::<T>()
        .ok()
        .filter(|n| n.to_string() == value)
        .ok_or_else(|| "replay integer must be a canonical decimal in range".into())
}
impl Choice {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let bound = decimal::<u128>(&self.bound)?;
        let value = decimal::<u128>(&self.value)?;
        let max = match self.kind.as_str() {
            "integer" => bound,
            "signed" if (1..=128).contains(&bound) => u128::MAX >> (128 - bound),
            "float" if bound == 32 || bound == 64 => u128::MAX >> (128 - bound),
            "index" if bound > 0 && bound <= usize::MAX as u128 => bound - 1,
            "bool" | "more" if bound == 1 => 1,
            _ => return Err("replay choice kind/bound mismatch".into()),
        };
        if value > max {
            return Err("replay choice value outside bound".into());
        }
        Ok(())
    }
}
pub(crate) fn shell(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
pub(crate) fn first_difference(a: &str, b: &str) -> Option<String> {
    if a == b {
        return None;
    }
    let (mut a, mut b) = (a.split_inclusive('\n'), b.split_inclusive('\n'));
    for line in 1.. {
        match (a.next(), b.next()) {
            (None, None) => unreachable!(),
            (a, b) if a != b => {
                return Some(format!(
                    "first differing trace line {line}: recorded {a:?}, current {b:?}"
                ));
            }
            _ => (),
        }
    }
    unreachable!()
}

// SHA-256 (FIPS 180-4); shared with the runtime so manifests need no host utility or new
// dependency graph in the directly compiled conformance std. Known-answer tests below.
pub(crate) fn digest(bytes: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&(bytes.len() as u64).wrapping_mul(8).to_be_bytes());
    for block in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(word.try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = state;
        for i in 0..64 {
            let t1 = h
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let t2 = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (s, v) in state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *s = s.wrapping_add(v);
        }
    }
    state.iter().map(|v| format!("{v:08x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sha256_and_strict_wire_boundaries() {
        assert_eq!(
            digest(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            digest(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
        let choice = Choice {
            kind: "integer".into(),
            bound: u128::MAX.to_string(),
            value: u128::MAX.to_string(),
        };
        choice.validate().unwrap();
        assert_eq!(
            decoded::<Choice>(encoded(&choice).as_bytes()).unwrap(),
            choice
        );
        for bad in [
            r#"{"kind":"bool","kind":"bool","bound":"1","value":"1"}"#,
            r#"{"kind":"bool","bound":"1","value":"1"}x"#,
            r#"{"kind":"bool""#,
        ] {
            assert!(decoded::<Choice>(bad.as_bytes()).is_err());
        }
        assert!(
            Choice {
                kind: "bool".into(),
                bound: "1".into(),
                value: "2".into()
            }
            .validate()
            .is_err()
        );
        assert!(first_difference("x\n", "x").is_some());
    }
    #[test]
    fn identity_refusals_are_short_per_entry_lines() {
        let build = |files: &[(&str, String)]| Build {
            source: BTreeMap::new(),
            library: BTreeMap::new(),
            dependencies: vec![files.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()],
            compiler: String::new(),
            target: String::new(),
            profile: String::new(),
            options: vec![],
            driver: String::new(),
            versions: VERSIONS.into(),
        };
        let names: Vec<String> = (0..10).map(|i| format!("lib{i}.rlib")).collect();
        let recorded: Vec<_> = names.iter().map(|n| (n.as_str(), digest(n.as_bytes()))).collect();
        let mut current = recorded.clone();
        current[0].1 = digest(b"changed");
        current.remove(1);
        assert_eq!(build(&recorded).compare(&build(&recorded)), Ok(()));
        let message = build(&recorded).compare(&build(&current)).unwrap_err();
        let lines: Vec<&str> = message.lines().collect();
        assert_eq!(lines[0], "replay dependency artifact mismatch; use fresh --seed for changed inputs");
        assert_eq!(lines[1], format!("  lib0.rlib: recorded {}, current {}", &recorded[0].1[..12], &current[0].1[..12]));
        assert_eq!(lines[2], format!("  lib1.rlib: recorded {}, current missing", &recorded[1].1[..12]));
        assert_eq!(lines.len(), 3);
        let unrelated: Vec<_> = names.iter().map(|n| (n.as_str(), digest(b"other"))).collect();
        let message = build(&recorded).compare(&build(&unrelated)).unwrap_err();
        assert_eq!(message.lines().count(), 1 + 8 + 1);
        assert!(message.ends_with("\n  and 2 more"), "{message}");
        let hostile = build(&[("lib\u{202e}\n.rlib", "x".into())]);
        let message = hostile.compare(&build(&[])).unwrap_err();
        assert!(message.ends_with("  lib\\u{202e}\\n.rlib: recorded x, current missing"), "{message}");
    }
}
