#!/usr/bin/env python3
"""fx-zu4f.12 admission observations: descriptive lexical counts, never a catch.

Run only after tide-lead's "go" (the real run requires --after-go), on every
pool crate and every class-excluded crate with a binary target in a pool
repository. --self-test uses synthetic source only.

Counted, with file:line, in each crate's non-test source (scan.py's superset):
  * ambient std::env, std::fs, std::net, std::process: each import path in a
    `use` tree, and each other path reference resolved through same-file
    `use` aliases (lexical; not type resolution, not proof of a live call);
  * spawns: std::thread::spawn, tokio::spawn, tokio::task::spawn and
    tokio::task::spawn_blocking calls. "detached" = the call is a whole
    expression statement, `let _ = ...;` or `drop(...)`; "joined" = followed
    at once by `.join()` or `.await`; otherwise "unresolved" (stored,
    returned or transformed; no drop inference).
Key flag (the rule): an observation is flagged when the innermost `fn` item
whose signature-plus-body contains it also contains an identifier with a
key-material word. Identifiers split on `_` and camelCase into lowercase
words; the words are key, keys, keypair, keypairs, seed, seeds, mnemonic,
mnemonics, secret, secrets, privkey, privkeys, privatekey, secretkey, or
`private` directly followed by `key`/`keys`. Comments, string literals and
test-only items are masked first, so only identifiers count. Observations
outside any fn body are never flagged.
"""
import argparse
import csv
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from scan import (Unscannable, balanced_end, entries, load_manifests, non_test,  # noqa: E402
                  package_sources, rows, table, use_paths)

AMBIENT = ('std::env', 'std::fs', 'std::net', 'std::process')
SPAWNS = ('std::thread::spawn', 'tokio::spawn', 'tokio::task::spawn', 'tokio::task::spawn_blocking')
KEY_WORDS = {'key', 'keys', 'keypair', 'keypairs', 'seed', 'seeds', 'mnemonic', 'mnemonics', 'secret',
             'secrets', 'privkey', 'privkeys', 'privatekey', 'secretkey'}
FIELDS = ['set', 'repository', 'crate', 'crate_path', 'file', 'line', 'kind', 'canonical', 'function',
          'key_flag', 'key_identifiers']


def words(identifier):
    return [w.lower() for part in identifier.split('_')
            for w in re.findall(r'[A-Z]+(?=[A-Z][a-z])|[A-Z]?[a-z]+|[A-Z]+|\d+', part)]


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


def category(path):
    for prefix in AMBIENT:
        if path == prefix or path.startswith(prefix + '::'):
            return prefix
    return None


def observe(path, text):
    """Observations in one masked non-test file, with original line numbers."""
    found = []
    spans = functions(text)

    def emit(offset, kind, canonical):
        inside = [s for s in spans if s[0] <= offset < s[1]]
        fn = max(inside) if inside else None
        keys = key_identifiers(text[fn[0]:fn[1]]) if fn else []
        found.append(dict(file=path, line=text.count('\n', 0, offset) + 1, kind=kind, canonical=canonical,
                          function=fn[2] if fn else '-', key_flag='yes' if keys else 'no',
                          key_identifiers=';'.join(keys[:3]) or '-'))
    imports = list(re.finditer(r'\buse\s+[^;]+;', text))
    aliases = {alias: canonical for canonical, alias in use_paths(text) if alias != '*'}
    for match in imports:
        for canonical, _ in use_paths(match[0]):
            if kind := category(canonical):
                emit(match.start(), 'ambient-import:' + kind, canonical)
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
    return found


def crate_files(repo_root, crate_path, manifests):
    manifest_path = (repo_root / crate_path / 'Cargo.toml')
    manifest = manifests[manifest_path]
    package_dirs = [p.parent for p, m in manifests.items() if isinstance(m, Unscannable) or 'package' in m]
    nested = [d for d in package_dirs if d != manifest_path.parent and d.is_relative_to(manifest_path.parent)]
    entry_files, _ = entries(manifest_path.parent, manifest)
    files, _ = package_sources(repo_root, manifest_path.parent, entry_files, nested)
    return {p.relative_to(repo_root).as_posix(): t[0] for p, t in files.items()}


def run(clones, work):
    pool = rows(work / 'pool.tsv')
    repos = {r['repository'] for r in pool}
    targets = [('pool', r['repository'], r['crate'], r['crate_path']) for r in pool]
    targets += [('excluded-bin', r['repository'], r['crate'], r['crate_path']) for r in rows(work / 'scan-output.tsv')
                if r['repository'] in repos and r['class'] == '-' and r['has_bin'] == 'yes']
    out, cache = [], {}
    for set_, repo, crate, crate_path in targets:
        root = (clones / repo.replace('/', '__')).resolve()
        manifests = cache.setdefault(repo, load_manifests(root))
        try:
            files = crate_files(root, crate_path, manifests)
        except (Unscannable, ValueError) as error:
            out.append(dict(set=set_, repository=repo, crate=crate, crate_path=crate_path, file='-', line='-',
                            kind='unscannable', canonical=str(error)[:200], function='-', key_flag='-',
                            key_identifiers='-'))
            continue
        for path, text in sorted(files.items()):
            for o in observe(path, text):
                out.append(dict(o, set=set_, repository=repo, crate=crate, crate_path=crate_path))
    table(work / 'admission-obs.tsv', FIELDS, out)
    print(f'{len(targets)} crates, {len(out)} observations, {sum(o["key_flag"] == "yes" for o in out)} key-flagged')


def self_test():
    text, _, _ = non_test('''use std::{fs as files, env::var as get, thread::spawn as go};
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
#[cfg(test)] fn ignored(mnemonic: u8) { std::fs::read("x"); }
''')
    result = observe('fixture.rs', text)
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
