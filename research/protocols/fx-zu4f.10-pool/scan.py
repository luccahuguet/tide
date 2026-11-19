#!/usr/bin/env python3
"""fx-zu4f.10 static archive filter (Python >=3.11; standard library only).

--fetch acquires pinned archives; the default scan is offline and verifies hashes.
--self-test uses synthetic fixtures without reading an upstream archive.
"""
import argparse
import csv
import datetime as dt
import email.utils
import hashlib
import io
import json
import pathlib
import re
import tarfile
import time
import tomllib
import urllib.error
import urllib.request

UA = 'tide-research-snapshot/1.0 (public registry metadata research)'
EXCLUDED_DIRS = {'tests', 'test', 'benches', 'examples', 'bin', '.git', 'target'}
FORBIDDEN = [
    ('Condvar', r'\bCondvar\b'),
    ('select!', r'\bselect\s*!'),
    ('async-fn', r'\basync\s+(?:unsafe\s+)?fn\b'),
    ('.await', r'\.\s*await\b'),
    ('Waker', r'\bWaker\b'),
]


def rows(path):
    with path.open(newline='') as f:
        return list(csv.DictReader(f, delimiter='\t'))


def table(path, fields, data):
    with path.open('w', newline='') as f:
        w = csv.DictWriter(f, fields, delimiter='\t', lineterminator='\n')
        w.writeheader()
        w.writerows(data)


def digest(body):
    return hashlib.sha256(body).hexdigest()


def mask_rust(text):
    """Blank nested comments and Rust literals, retaining source offsets."""
    raw_re = re.compile(r'(?:br|cr|r)(#*)"')
    char_re = re.compile(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\\n])'")
    special = re.compile(r'//|/\*|(?:br|cr|r)#*"|"|\x27')
    out = list(text)
    i = 0
    while candidate := special.search(text, i):
        i = candidate.start()
        start = i
        if text.startswith('//', i):
            end = text.find('\n', i)
            i = len(text) if end < 0 else end
        elif text.startswith('/*', i):
            i += 2
            depth = 1
            while i < len(text) and depth:
                if text.startswith('/*', i):
                    depth += 1
                    i += 2
                elif text.startswith('*/', i):
                    depth -= 1
                    i += 2
                else:
                    i += 1
        else:
            raw = raw_re.match(text, i)
            if raw and (not i or not (text[i-1].isalnum() or text[i-1] == '_')):
                marker = '"' + raw[1]
                end = text.find(marker, raw.end())
                i = len(text) if end < 0 else end + len(marker)
            elif text[i] == '"':
                i += 1
                while i < len(text):
                    if text[i] == '\\':
                        i += 2
                    elif text[i] == '"':
                        i += 1
                        break
                    else:
                        i += 1
            elif text[i] == "'" and (char := char_re.match(text, i)):
                i = char.end()
            else:
                i += 1
                continue
        out[start:i] = ['\n' if c == '\n' else ' ' for c in text[start:i]]
    return ''.join(out)


def balanced_end(text, start, opening='{', closing='}'):
    depth = 0
    for i in range(start, len(text)):
        if text[i] == opening:
            depth += 1
        elif text[i] == closing:
            depth -= 1
            if depth == 0:
                return i + 1
    raise ValueError(f'unbalanced Rust delimiter {opening} at offset {start}')


def cfg_without_test(expr):
    """Possible cfg values with test=false; other flags remain unknown."""
    tokens = re.findall(r'\w+|[(),=]', expr)
    def parse(i):
        name = tokens[i]
        i += 1
        if i < len(tokens) and tokens[i] == '(':
            i += 1
            args = []
            while tokens[i] != ')':
                values, i = parse(i)
                args.append(values)
                if tokens[i] == ',':
                    i += 1
            i += 1
            if name == 'not' and len(args) == 1:
                return {not x for x in args[0]}, i
            result = {name == 'all'}
            if name in ('all', 'any'):
                for values in args:
                    result = {(a and b) if name == 'all' else (a or b) for a in result for b in values}
                return result, i
            return {False, True}, i
        if i < len(tokens) and tokens[i] == '=':
            i += 1
            if i < len(tokens) and tokens[i] not in (',', ')'):
                i += 1
            return {False, True}, i
        return ({False} if name == 'test' else {False, True}), i
    try:
        result, end = parse(0)
        return result if end == len(tokens) else {False, True}
    except (IndexError, ValueError):
        return {False, True}


def non_test(text):
    """Remove definitely test-only items; return external test module names."""
    masked = mask_rust(text)
    for match in re.finditer(r'#!\s*\[\s*cfg\s*\(', masked):
        end = balanced_end(masked, masked.index('[', match.start()), '[', ']')
        attr = masked[match.start():end]
        if cfg_without_test(attr[attr.find('(')+1:attr.rfind(')')]) == {False}:
            return re.sub(r'[^\n]', ' ', masked), set()
    external = set()
    removed = []
    for match in re.finditer(r'#\s*\[\s*(cfg\s*\(|test\s*\])', masked):
        if any(a <= match.start() < b for a, b in removed):
            continue
        attr_open = masked.index('[', match.start())
        attr_end = balanced_end(masked, attr_open, '[', ']')
        attr = masked[attr_open+1:attr_end-1].strip()
        is_test = attr == 'test'
        if attr.startswith('cfg'):
            expr = attr[attr.find('(')+1:attr.rfind(')')]
            is_test = cfg_without_test(expr) == {False}
        if not is_test:
            continue
        i = attr_end
        while True:
            tail = re.match(r'\s*#\s*\[', masked[i:])
            if not tail:
                break
            i = balanced_end(masked, i + tail.end()-1, '[', ']')
        item_start = i
        item = re.match(r'\s*(?:pub(?:\([^)]*\))?\s+)?(?:(?:unsafe|async|default|auto)\s+)*(?:fn|mod|struct|enum|union|trait|impl|use|const|static|type|extern|macro_rules)\b', masked[i:])
        field = item is None
        angles = 0
        while i < len(masked):
            if masked[i] in '([':
                opening = masked[i]
                i = balanced_end(masked, i, opening, ')' if opening == '(' else ']')
            elif masked[i] == '{':
                i = balanced_end(masked, i)
                if not field:
                    break
            elif masked[i] == '}':
                break
            elif field and masked[i] == ',' and angles == 0:
                i += 1
                break
            elif field and masked[i] == '<':
                angles += 1
                i += 1
            elif field and masked[i] == '>':
                angles = max(0, angles-1)
                i += 1
            elif masked[i] == ';':
                module = re.search(r'\bmod\s+(\w+)\s*$', masked[item_start:i])
                if module:
                    external.add(module[1])
                i += 1
                break
            else:
                i += 1
        removed.append((match.start(), i))
    out = list(masked)
    for a, b in removed:
        out[a:b] = ['\n' if c == '\n' else ' ' for c in masked[a:b]]
    return ''.join(out), external


def use_paths(text):
    """Expand ordinary nested Rust use trees to (path, local alias) pairs."""
    paths = []
    for match in re.finditer(r'\buse\s+([^;]+);', text):
        tokens = re.findall(r'::|\w+|[{},*]', match[1])
        def tree(i, prefix):
            path = list(prefix)
            while i < len(tokens):
                token = tokens[i]
                if token == '{':
                    i += 1
                    while i < len(tokens) and tokens[i] != '}':
                        i = tree(i, path)
                        if i < len(tokens) and tokens[i] == ',':
                            i += 1
                    return i + 1
                if token in (',', '}'):
                    break
                if token == 'as':
                    paths.append(('::'.join(path), tokens[i+1]))
                    return i + 2
                if token != '::':
                    path.append(token)
                i += 1
            if path:
                if path[-1] == 'self':
                    path.pop()
                if path:
                    paths.append(('::'.join(path), path[-1]))
            return i
        tree(0, [])
    return paths


def concurrency(text):
    paths = use_paths(text)
    references = [p for p, _ in paths]
    aliases = {alias: path for path, alias in paths if alias != '*'}
    for match in re.finditer(r'\b\w+(?:\s*::\s*\w+)+', text):
        head, _, tail = re.sub(r'\s', '', match[0]).partition('::')
        references.append(aliases.get(head, head) + '::' + tail)
    for path, alias in paths:
        if alias == '*':
            if path == 'std::sync':
                references += ['std::sync::'+name for name in ('mpsc','Mutex','RwLock','atomic') if re.search(r'\b'+name+r'\b', text)]
            if path == 'std::thread':
                references += ['std::thread::'+name for name in ('spawn','scope') if re.search(r'\b'+name+r'\s*\(', text)]
            if path == 'std::sync::atomic' and re.search(r'\bAtomic\w+\b', text):
                references.append('std::sync::atomic')
    accepted = re.compile(r'^std::(?:thread::(?:spawn|scope)(?:::|$)|sync::(?:mpsc|Mutex|RwLock|atomic)(?:::|$))')
    return sorted({p for p in references if accepted.search(p)})


def forbidden_dependencies(manifest):
    tables = [manifest.get('dependencies', {})]
    tables += [target.get('dependencies', {}) for target in manifest.get('target', {}).values()]
    packages = set()
    for deps in tables:
        for alias, spec in deps.items():
            name = spec.get('package', alias) if isinstance(spec, dict) else alias
            if name in {'tokio', 'futures', 'async-std', 'flume', 'loom'} or name.startswith(('crossbeam', 'parking_lot', 'rayon')):
                packages.add(name)
    return sorted(packages)


def unsafe_send_sync(text):
    aliases = [alias for path, alias in use_paths(text)
               if path in {'std::marker::Send', 'std::marker::Sync', 'core::marker::Send', 'core::marker::Sync'}]
    traits = '|'.join(re.escape(name) for name in ['Send', 'Sync'] + aliases)
    return re.search(r'\bunsafe\s+impl\b[^;{}]*?\b(?:' + traits + r')\s+for\b', text)


def library_sources(root, manifest):
    entry = (root / manifest.get('lib', {}).get('path', 'src/lib.rs')).resolve()
    if not entry.is_file() or not entry.resolve().is_relative_to(root.resolve()):
        return [], set()
    # shortcut: lexical source superset, use a compiler graph only if a future protocol permits compilation.
    paths = [p for p in sorted(entry.parent.rglob('*.rs')) if not (set(p.relative_to(root).parts[:-1]) & EXCLUDED_DIRS) and p.name != 'build.rs']
    texts = {p: non_test(p.read_text(encoding='utf-8')) for p in paths}
    reachable = set()

    def visit(path):
        path = path.resolve()
        if path in reachable or not path.is_file():
            return
        if not path.resolve().is_relative_to(root.resolve()):
            raise ValueError('library module escapes archive root')
        reachable.add(path)
        original = path.read_text(encoding='utf-8')
        text, modules = texts.setdefault(path, non_test(original))
        attr_re = re.compile(r'#!?\s*\[')
        mod_re = re.compile(r'(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?mod\s+(\w+)\s*([;{])')
        word_re = re.compile(r'\w+')

        def walk(start, end, base):
            i = start
            attributes = []
            while i < end:
                if text[i].isspace():
                    i += 1
                    continue
                if text[i] == '#' and (attr := attr_re.match(text, i)):
                    stop = balanced_end(text, attr.end()-1, '[', ']')
                    attributes.append(original[i:stop])
                    i = stop
                    continue
                module = mod_re.match(text, i)
                if module:
                    name, delimiter = module.groups()
                    opening = module.end()-1
                    if delimiter == '{':
                        stop = balanced_end(text, opening)
                        walk(opening+1, stop-1, base/name)
                        i = stop
                    else:
                        custom = re.search(r'#\s*\[\s*path\s*=\s*"([^"\n]+)"\s*\]', '\n'.join(attributes))
                        options = [path.parent/custom[1]] if custom else [base/(name+'.rs'), base/name/'mod.rs']
                        for child in options:
                            if child.is_file():
                                visit(child)
                                break
                        i = opening+1
                    attributes = []
                    continue
                attributes = []
                if text[i] in '{([':
                    opening = text[i]
                    i = balanced_end(text, i, opening, {'{':'}','(':')','[':']'}[opening])
                elif word := word_re.match(text, i):
                    i = word.end()
                else:
                    i += 1
        base = path.parent if path == entry or path.name == 'mod.rs' else path.with_suffix('')
        try:
            walk(0, len(text), base)
        except ValueError as error:
            raise ValueError(f'{path.relative_to(root)}: {error}') from error

    visit(entry)
    excluded = set()
    for p, (_, modules) in texts.items():
        base = p.parent if p == entry or p.name == 'mod.rs' else p.with_suffix('')
        for name in modules:
            child = base / name
            excluded.update(q for q in paths if q == child.with_suffix('.rs') or q.is_relative_to(child))
    excluded -= reachable
    sources = [(p.relative_to(root).as_posix(), text) for p, (text, _) in sorted(texts.items()) if p not in excluded]
    return sources, {p.relative_to(root).as_posix() for p in reachable}


def inspect(root):
    manifest = tomllib.loads((root/'Cargo.toml').read_text())
    sources, reachable = library_sources(root, manifest)
    hits = [(path, hit) for path, text in sources if path in reachable for hit in concurrency(text)]
    if not sources:
        return 'no-library-source', '', 0
    if not hits:
        return 'no-std-concurrency-use', '', len(sources)
    for label, pattern in FORBIDDEN:
        for path, text in sources:
            if re.search(pattern, text):
                return f'forbidden-source:{label}:{path}', '', len(sources)
    deps = forbidden_dependencies(manifest)
    if deps:
        return f'forbidden-normal-dependency:{deps[0]}', '', len(sources)
    for path, text in sources:
        if unsafe_send_sync(text):
            return f'forbidden-source:unsafe-impl-Send-or-Sync:{path}', '', len(sources)
    return '', ';'.join(path+':'+hit for path, hit in hits), len(sources)


def self_test():
    import tempfile
    source = '''// async fn comment() {}
/* Condvar /* Waker */ */
const S: &str = r###"select! async fn f() {}"###;
use std::{sync::{Mutex as M, atomic::AtomicUsize}, thread::{self, spawn as go}};
#[cfg(test)] mod tests { async fn only_test() { x.await; } }
#[cfg(all(test, feature="x"))] mod extra;
#[cfg(not(test))] fn production() { let _ = M::new(0); }
'''
    text, external = non_test(source)
    assert external == {'extra'} and 'only_test' not in text
    assert not any(re.search(pattern, text) for _, pattern in FORBIDDEN)
    assert 'std::sync::Mutex' in concurrency(text)
    assert 'std::thread::spawn' in concurrency(text)
    assert concurrency('use std::thread as t; t :: scope(||{});') == ['std::thread::scope']
    assert concurrency('std \n :: sync \t :: Mutex::new(0);') == ['std::sync::Mutex::new']
    assert not concurrency(' ' * 100_000)
    assert concurrency('fn f() { my::Mutex::new(0); }') == []
    assert not concurrency(mask_rust('// std::sync::Mutex\n const X:&str="std::sync::Mutex";'))
    assert re.search(FORBIDDEN[2][1], 'pub async unsafe fn f(){}')
    assert unsafe_send_sync('unsafe impl<T: Copy> std::marker::Send for X<T> {}')
    assert unsafe_send_sync('use core::marker::Send as S; unsafe impl<T> S for X<T> {}')
    assert not non_test('#![cfg(test)]\nasync fn f(){}')[0].strip()
    fields, _ = non_test('struct X { #[cfg(test)] test: Option<Condvar>, real: std::sync::Mutex<()> }')
    assert 'Condvar' not in fields and 'std::sync::Mutex' in fields
    assert fields.count('{') == fields.count('}') == 1
    m = tomllib.loads('''[dependencies]
renamed = { package="tokio", version="1", optional=true }
[dev-dependencies]
loom = "0.7"
[target.'cfg(unix)'.dependencies]
crossbeam-utils = "0.8"
''')
    assert forbidden_dependencies(m) == ['crossbeam-utils', 'tokio']
    assert forbidden_dependencies({'dev-dependencies': {'loom':'0.7'}}) == []
    with tempfile.TemporaryDirectory() as tmp:
        root = pathlib.Path(tmp)
        (root/'src/tests').mkdir(parents=True)
        (root/'src/lib.rs').write_text('use std::sync::Mutex; #[cfg(test)] mod support;')
        (root/'src/support.rs').write_text('async fn ignored(){}')
        (root/'src/tests/test.rs').write_text('async fn ignored(){}')
        (root/'Cargo.toml').write_text('[package]\nname="fixture"\nversion="0.1.0"\n')
        assert inspect(root)[0] == ''
        (root/'src/lib.rs').write_text('use std::sync::Mutex; pub async fn bad(){}')
        assert inspect(root)[0].startswith('forbidden-source:async-fn')
        (root/'src/lib.rs').write_text('fn ordinary(){}')
        (root/'src/main.rs').write_text('use std::sync::Mutex;')
        assert inspect(root)[0] == 'no-std-concurrency-use'
        (root/'src/lib.rs').write_text('mod production;')
        (root/'src/support.rs').write_text('fn ordinary(){}')
        (root/'src/production.rs').write_text('use std::sync::Mutex;')
        assert inspect(root)[0] == ''
    print('self-test: passed')


def acquire(work, target, candidates):
    directory = target/'archives'
    directory.mkdir(parents=True, exist_ok=True)
    path = work/'archive-hashes.tsv'
    existing = {r['crate']:r for r in rows(path)} if path.exists() else {}
    records = []
    last = 0.0
    for index, row in enumerate(candidates, 1):
        name, version = row['crate'], row['version']
        filename = f'{name}-{version}.crate'
        dest = directory/filename
        if name in existing:
            record = existing[name]
            assert record['version'] == version and digest(dest.read_bytes()) == record['sha256']
        else:
            url = f'https://static.crates.io/crates/{name}/{filename}'
            for attempt in range(3):
                time.sleep(max(0, 1.1 - (time.monotonic()-last)))
                last = time.monotonic()
                start = dt.datetime.now(dt.UTC).isoformat()
                try:
                    with urllib.request.urlopen(urllib.request.Request(url, headers={'User-Agent': UA}), timeout=90) as response:
                        body = response.read()
                    dest.write_bytes(body)
                    record = dict(crate=name, version=version, url=url, started_utc=start, ended_utc=dt.datetime.now(dt.UTC).isoformat(), bytes=len(body), sha256=digest(body), filename=filename)
                    break
                except (urllib.error.URLError, TimeoutError) as error:
                    with (work/'fetch-errors.log').open('a') as log:
                        log.write(f'{start} {url} attempt={attempt+1} {error}\n')
                    if attempt == 2:
                        (work/'BLOCKED').write_text(f'Archive acquisition failed after three attempts: {url}: {error}; see fetch-errors.log\n')
                        raise
                    delay = 30*(attempt+1)
                    retry = error.headers.get('Retry-After') if isinstance(error, urllib.error.HTTPError) else None
                    if retry:
                        try:
                            delay = max(delay, float(retry))
                        except ValueError:
                            delay = max(delay, (email.utils.parsedate_to_datetime(retry)-dt.datetime.now(dt.UTC)).total_seconds())
                    time.sleep(delay)
        records.append(record)
        table(path, ['crate','version','url','started_utc','ended_utc','bytes','sha256','filename'], records)
        if index % 25 == 0:
            print(f'archives {index}/{len(candidates)}', flush=True)


def run(work, target, fetch):
    metadata = rows(work/'stars.tsv')
    assert len(metadata) == len({r['crate'] for r in metadata}) == 1000
    assert [int(r['rank']) for r in metadata] == list(range(1,1001))
    candidates = [r for r in metadata if r['status']=='kept']
    if fetch:
        acquire(work, target, candidates)
    archives = {r['crate']:r for r in rows(work/'archive-hashes.tsv')}
    assert set(archives) == {r['crate'] for r in candidates}
    output = []
    for row in metadata:
        reason = row['status'].removeprefix('excluded:') if row['status'] != 'kept' else ''
        evidence, count = '', 0
        if row['status'] == 'kept':
            archive = archives[row['crate']]
            assert archive['version'] == row['version']
            body = (target/'archives'/archive['filename']).read_bytes()
            assert digest(body) == archive['sha256']
            unpack = target/'unpacked'
            with tarfile.open(fileobj=io.BytesIO(body), mode='r:gz') as tar:
                if any(not (m.isfile() or m.isdir()) for m in tar.getmembers()):
                    raise ValueError('archive contains non-file/directory member')
                tar.extractall(unpack, filter='data')
            root = unpack/f'{row["crate"]}-{row["version"]}'
            assert root.is_dir()
            reason, evidence, count = inspect(root)
        output.append(dict(rank=row['rank'], crate=row['crate'], version=row['version'], repository=row['github_repository'], downloads=row['downloads'], stars=row['stars'], outcome='excluded' if reason else 'kept', first_reason=reason, source_files=count, std_evidence=evidence or '-'))
        if len(output) % 100 == 0:
            print(f'scanned {len(output)}/1000', flush=True)
    fields = ['rank','crate','version','repository','downloads','stars','outcome','first_reason','source_files','std_evidence']
    table(work/'scan-output.tsv', fields, output)
    pool = [r for r in output if r['outcome']=='kept'][:30]
    table(work/'pool.tsv', ['pool_rank']+fields, [dict(pool_rank=i,**r) for i,r in enumerate(pool,1)])
    print(json.dumps(dict(snapshot=len(metadata), after_stars=len(candidates), after_scan=sum(r['outcome']=='kept' for r in output), pool=len(pool)), sort_keys=True))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--work', type=pathlib.Path, default=pathlib.Path(__file__).resolve().parent)
    parser.add_argument('--target', type=pathlib.Path, default=pathlib.Path('/mnt/build/tide-agents/fx-zu4f.10-target'))
    parser.add_argument('--fetch', action='store_true')
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        self_test()
    else:
        assert args.target.resolve().is_relative_to(pathlib.Path('/mnt/build'))
        (args.target/'tmp').mkdir(parents=True, exist_ok=True)
        run(args.work, args.target, args.fetch)


if __name__ == '__main__':
    main()
