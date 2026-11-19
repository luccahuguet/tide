#!/usr/bin/env python3
"""fx-zu4f.14 admission observations: descriptive lexical counts, never a catch.

fx-zu4f.12's admission-obs.py plus the bead's ambient randomness sources and
where they sit relative to keygen, encapsulation and signing. Run only after
tide-lead's "go" (the real run requires --after-go), on every pool crate and
every class-excluded package with a binary target in a pool repository.
--self-test uses synthetic source only.

Counted, with file:line, in each crate's non-test source (classify.py's
superset; comments, string literals and test-only items masked first):
  * ambient std::env, std::fs, std::net, std::process (fx-zu4f.12): each
    import path in a `use` tree, and each other path reference resolved
    through same-file `use` aliases (lexical; not type resolution, not proof
    of a live call);
  * spawns (fx-zu4f.12): std::thread::spawn, tokio::spawn, tokio::task::spawn
    and tokio::task::spawn_blocking calls. "detached" = the call is a whole
    expression statement, `let _ = ...;` or `drop(...)`; "joined" = followed
    at once by `.join()` or `.await`; otherwise "unresolved";
  * ambient randomness (fx-zu4f.14), as imports and as references, by source:
      getrandom    a path whose first segment is `getrandom`;
      OsRng        a path with a segment `OsRng`;
      thread_rng   a path with a segment `thread_rng` or `ThreadRng`, or the
                   path `rand::rng` (rand 0.9's name for thread_rng);
      rand::random the path `rand::random`;
      /dev/urandom the text `/dev/urandom` in non-test code with comments
                   masked and string literals kept (the one source that is a
                   literal).
    Paths resolve through same-file aliases, so `use rand::random;` then
    `random()` counts. A local item that reuses one of these names counts too
    (lexical over-approximation).
Operation labels (fx-zu4f.14, the bead's "where they enter keygen,
encapsulation or signing"), from names split on `_` and camelCase into
lowercase words:
  keygen  the joined name contains `keygen` or `keypair`, or its words hold
          key/keys together with gen/generate;
  encaps  the joined name contains `encap`;
  sign    its words hold sign, signs, signing, signer, signature or
          signatures (so `signal` and `assign` do not count).
`function_op` labels the innermost `fn` item containing the observation;
`callee_op` labels the callee of the innermost unclosed call whose argument
list contains it (`keypair(&mut OsRng)` gives callee `keypair`). No call
graph: randomness that reaches keygen through other functions is not traced.
Key flag (fx-zu4f.12's rule): an observation is flagged when the innermost
`fn` item containing it also contains an identifier with a key-material word
(key, keys, keypair(s), seed(s), mnemonic(s), secret(s), privkey(s),
privatekey, secretkey, or `private` directly followed by `key`/`keys`).
Observations outside any fn body are never flagged or labelled.
"""
import argparse
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from classify import (Unscannable, balanced_end, entries, load_manifests, mask_rust, non_test,  # noqa: E402
                      package_sources, read, rows, table, use_paths)

AMBIENT = ('std::env', 'std::fs', 'std::net', 'std::process')
SPAWNS = ('std::thread::spawn', 'tokio::spawn', 'tokio::task::spawn', 'tokio::task::spawn_blocking')
KEY_WORDS = {'key', 'keys', 'keypair', 'keypairs', 'seed', 'seeds', 'mnemonic', 'mnemonics', 'secret',
             'secrets', 'privkey', 'privkeys', 'privatekey', 'secretkey'}
SIGN_WORDS = {'sign', 'signs', 'signing', 'signer', 'signature', 'signatures'}
FIELDS = ['set', 'repository', 'crate', 'crate_path', 'file', 'line', 'kind', 'canonical', 'function',
          'function_op', 'callee', 'callee_op', 'key_flag', 'key_identifiers']


def words(identifier):
    return [w.lower() for part in identifier.split('_')
            for w in re.findall(r'[A-Z]+(?=[A-Z][a-z])|[A-Z]?[a-z]+|[A-Z]+|\d+', part)]


def operations(name):
    """Operation labels for a function or callee name (see the docstring)."""
    if not name or name == '-':
        return '-'
    w = words(name)
    joined = ''.join(w)
    labels = []
    if 'keygen' in joined or 'keypair' in joined or ({'key', 'keys'} & set(w) and {'gen', 'generate'} & set(w)):
        labels.append('keygen')
    if 'encap' in joined:
        labels.append('encaps')
    if SIGN_WORDS & set(w):
        labels.append('sign')
    return ';'.join(labels) or '-'


def key_identifiers(text):
    found = []
    for ident in sorted(set(re.findall(r'\b[A-Za-z_]\w*\b', text))):
        w = words(ident)
        if KEY_WORDS & set(w) or any(a == 'private' and b in ('key', 'keys') for a, b in zip(w, w[1:])):
            found.append(ident)
    return found


def functions(text):
    """(start, end, name) of each `fn` item with a body, in the masked text."""
    spans = []
    for m in re.finditer(r'\bfn\s+(\w+)', text):
        i = m.end()
        while i < len(text) and text[i] not in '{;':
            if text[i] in '([':
                i = balanced_end(text, i, text[i], ')' if text[i] == '(' else ']')
            else:
                i += 1
        if i < len(text) and text[i] == '{':
            spans.append((m.start(), balanced_end(text, i), m[1]))
    return spans


def callee(text, offset, floor):
    """Name of the innermost unclosed call around offset, not looking before floor."""
    depth = 0
    for i in range(offset - 1, floor - 1, -1):
        c = text[i]
        if c in ')]':
            depth += 1
        elif c == '[':
            depth -= 1 if depth else 0
        elif c == '(':
            if depth:
                depth -= 1
                continue
            m = re.search(r'(\w+)\s*(?:::\s*<[^()]*>\s*)?$', text[floor:i])
            return m[1] if m else '-'
        elif c in ';{}' and depth == 0:
            return '-'
    return '-'


def category(path):
    for prefix in AMBIENT:
        if path == prefix or path.startswith(prefix + '::'):
            return prefix
    return None


def randomness(path):
    """Randomness source label for a canonical path, else None."""
    segments = path.split('::')
    if segments[0] == 'getrandom':
        return 'getrandom'
    if 'OsRng' in segments:
        return 'OsRng'
    if {'thread_rng', 'ThreadRng'} & set(segments) or path == 'rand::rng' or path.startswith('rand::rng::'):
        return 'thread_rng'
    if path == 'rand::random' or path.startswith('rand::random::'):
        return 'rand::random'
    return None


def observe(path, text, literal_text=None):
    """Observations in one masked non-test file, with original line numbers.

    literal_text is the same file with comments and test-only items masked
    but string literals kept (for /dev/urandom); None skips that source."""
    found = []
    spans = functions(text)

    def emit(offset, kind, canonical):
        inside = [s for s in spans if s[0] <= offset < s[1]]
        fn = max(inside) if inside else None
        keys = key_identifiers(text[fn[0]:fn[1]]) if fn else []
        call = callee(text, offset, fn[0]) if fn else '-'
        found.append(dict(file=path, line=text.count('\n', 0, offset) + 1, kind=kind, canonical=canonical,
                          function=fn[2] if fn else '-', function_op=operations(fn[2]) if fn else '-',
                          callee=call, callee_op=operations(call) if fn else '-',
                          key_flag='yes' if keys else 'no', key_identifiers=';'.join(keys[:3]) or '-'))
    imports = list(re.finditer(r'\buse\s+[^;]+;', text))
    aliases = {alias: canonical for canonical, alias in use_paths(text) if alias != '*'}
    for match in imports:
        for canonical, _ in use_paths(match[0]):
            if kind := category(canonical):
                emit(match.start(), 'ambient-import:' + kind, canonical)
            if source := randomness(canonical):
                emit(match.start(), 'randomness-import:' + source, canonical)
    body = list(text)
    for match in imports:
        body[match.start():match.end()] = ['\n' if c == '\n' else ' ' for c in match[0]]
    body = ''.join(body)
    for match in re.finditer(r'\b\w+(?:\s*::\s*\w+)*', body):
        spelling = re.sub(r'\s', '', match[0])
        head, sep, tail = spelling.partition('::')
        canonical = aliases.get(head, head) + (sep + tail if sep else '')
        if kind := category(canonical):
            emit(match.start(), 'ambient-reference:' + kind, canonical)
        if source := randomness(canonical):
            emit(match.start(), 'randomness-reference:' + source, canonical)
        if canonical not in SPAWNS:
            continue
        opening = re.match(r'\s*\(', body[match.end():])
        if not opening:
            continue
        call_end = balanced_end(body, match.end() + opening.end() - 1, '(', ')')
        prefix = body[max(body.rfind(c, 0, match.start()) for c in ';{}') + 1:match.start()].strip()
        suffix = body[call_end:]
        if (prefix == '' or re.fullmatch(r'let\s+_\s*=', prefix)) and re.match(r'\s*;', suffix):
            kind = 'detached-spawn'
        elif re.fullmatch(r'(?:std\s*::\s*mem\s*::\s*)?drop\s*\(', prefix) and re.match(r'\s*\)\s*;', suffix):
            kind = 'detached-spawn'
        elif re.match(r'\s*\.\s*(?:join\s*\(|await\b)', suffix):
            kind = 'joined-spawn'
        else:
            kind = 'unresolved-spawn'
        emit(match.start(), kind, canonical)
    if literal_text is not None:
        for match in re.finditer(r'/dev/urandom', literal_text):
            emit(match.start(), 'randomness-literal:/dev/urandom', '/dev/urandom')
    found.sort(key=lambda o: (o['line'], o['kind']))
    return found


def literal_view(original, removed):
    """The file with comments and removed test-only ranges blanked, literals kept."""
    out = list(mask_rust(original, literals=False))
    for a, b in removed:
        out[a:b] = ['\n' if c == '\n' else ' ' for c in out[a:b]]
    return ''.join(out)


def crate_files(repo_root, crate_path, manifests):
    manifest_path = (repo_root / crate_path / 'Cargo.toml')
    manifest = manifests[manifest_path]
    package_dirs = [p.parent for p, m in manifests.items() if isinstance(m, Unscannable) or 'package' in m]
    nested = [d for d in package_dirs if d != manifest_path.parent and d.is_relative_to(manifest_path.parent)]
    entry_files, _ = entries(manifest_path.parent, manifest)
    files, _ = package_sources(repo_root, manifest_path.parent, entry_files, nested)
    return {p.relative_to(repo_root).as_posix(): (t[0], literal_view(read(p), t[2])) for p, t in files.items()}


def run(clones, work):
    pool = rows(work / 'pool.tsv')
    repos = {r['repository'] for r in pool}
    targets = [('pool', r['repository'], r['crate'], r['crate_path']) for r in pool]
    targets += [('excluded-bin', r['repository'], r['crate'], r['crate_path']) for r in rows(work / 'classes.tsv')
                if r['repository'] in repos and r['class'] == '-' and r['has_bin'] == 'yes']
    out, cache = [], {}
    for set_, repo, crate, crate_path in targets:
        root = (clones / repo.replace('/', '__')).resolve()
        manifests = cache.setdefault(repo, load_manifests(root))
        try:
            files = crate_files(root, crate_path, manifests)
        except (Unscannable, ValueError) as error:
            out.append(dict(dict.fromkeys(FIELDS, '-'), set=set_, repository=repo, crate=crate,
                            crate_path=crate_path, kind='unscannable', canonical=str(error)[:200]))
            continue
        for path, (text, literal_text) in sorted(files.items()):
            for o in observe(path, text, literal_text):
                out.append(dict(o, set=set_, repository=repo, crate=crate, crate_path=crate_path))
    table(work / 'admission-obs.tsv', FIELDS, out)
    rand = [o for o in out if o['kind'].startswith('randomness')]
    print(f'{len(targets)} crates, {len(out)} observations, {sum(o["key_flag"] == "yes" for o in out)} key-flagged, '
          f'{len(rand)} randomness, {sum(o["function_op"] != "-" or o["callee_op"] != "-" for o in rand)} '
          f'of them labelled keygen/encaps/sign')


def self_test():
    source = '''use std::{fs as files, env::var as get, thread::spawn as go};
fn load_seed(path: &str) -> Vec<u8> {
    files::read(path).unwrap()
}
fn plain() {
    get("HOME").ok();
    let _ = go(|| {});
    go(|| {});
    let h = go(|| {});
    go(|| {}).join().unwrap();
    // std::net::TcpStream::connect("ignored"); let secret = 1;
    let s = "private key";
}
fn sign(privateKey: u8) { std::process::exit(0); }
async fn bg() { tokio::spawn(async {}); tokio::task::spawn_blocking(|| {}).await; }
#[cfg(test)] fn ignored(mnemonic: u8) { std::fs::read("x"); OsRng; }
use rand::{rngs::OsRng, random};
pub fn generate_keypair() -> Kp { Kp::from_seed(&mut OsRng) }
pub fn encapsulate(pk: &Pk) -> Ct { let r: u64 = random(); inner(pk, r) }
fn main() { let kp = MlKem::key_gen(&mut rand::thread_rng()); let x = rand::rng(); }
fn try_sign_with_rng(m: &[u8]) { getrandom::fill(&mut buf).unwrap(); }
fn signal() { std::fs::File::open("/dev/urandom"); } // "/dev/urandom" in a comment
'''
    text, _, removed = non_test(source)
    result = observe('fixture.rs', text, literal_view(source, removed))
    kinds = [(r['line'], r['kind'], r['key_flag']) for r in result]
    assert (3, 'ambient-reference:std::fs', 'yes') in kinds, kinds
    assert (6, 'ambient-reference:std::env', 'no') in kinds, kinds
    assert (14, 'ambient-reference:std::process', 'yes') in kinds, kinds
    assert [k for k in kinds if k[1] == 'detached-spawn'] == [(7, 'detached-spawn', 'no'), (8, 'detached-spawn', 'no'),
                                                              (15, 'detached-spawn', 'no')], kinds
    assert sum(k[1] == 'unresolved-spawn' for k in kinds) == 1 and sum(k[1] == 'joined-spawn' for k in kinds) == 2
    assert sum(k[1].startswith('ambient-import') for k in kinds) == 2, kinds
    assert not any(r['line'] in (11, 16) for r in result)
    assert key_identifiers('let seedPhrase = x; let keypair_bytes = y; let monkey = z;') == ['keypair_bytes', 'seedPhrase']
    rand = [(r['line'], r['kind'], r['function_op'], r['callee'], r['callee_op']) for r in result
            if r['kind'].startswith('randomness')]
    assert rand == [(17, 'randomness-import:OsRng', '-', '-', '-'),
                    (17, 'randomness-import:rand::random', '-', '-', '-'),
                    (18, 'randomness-reference:OsRng', 'keygen', 'from_seed', '-'),
                    (19, 'randomness-reference:rand::random', 'encaps', '-', '-'),
                    (20, 'randomness-reference:thread_rng', '-', 'key_gen', 'keygen'),
                    (20, 'randomness-reference:thread_rng', '-', '-', '-'),
                    (21, 'randomness-reference:getrandom', 'sign', '-', '-'),
                    (22, 'randomness-literal:/dev/urandom', '-', 'open', '-')], rand
    assert operations('signal') == '-' and operations('generate_key') == 'keygen' and operations('SignMessage') == 'sign'
    print('self-test: passed')


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--after-go', action='store_true', help='required for the real run')
    ap.add_argument('--clones', type=pathlib.Path)
    ap.add_argument('--work', type=pathlib.Path)
    a = ap.parse_args()
    if a.self_test:
        self_test()
    elif not (a.after_go and a.clones and a.work):
        ap.error('the real run needs --after-go, --clones and --work (pool crates only after go)')
    else:
        run(a.clones.resolve(), a.work.resolve())
