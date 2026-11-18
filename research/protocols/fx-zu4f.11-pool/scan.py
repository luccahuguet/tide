#!/usr/bin/env python3
"""fx-zu4f.11 mechanical checkout scan; Python >=3.11, no upstream execution.

Adapted from fx-zu4f.10-pool/scan.py. Run only on a pinned checkout containing
Cargo.toml and Rust source blobs. --self-test uses synthetic source only.
"""
import argparse
import csv
import hashlib
import json
import pathlib
import re
import tomllib

EXCLUDED_DIRS = {'tests', 'test', 'benches', 'examples', '.git', 'target'}
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
            elif masked[i] in ')}]':
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
    if any(not p.resolve().is_relative_to(root.resolve()) for p in paths):
        raise ValueError('source symlink escapes checkout')
    paths = sorted({p.resolve() for p in paths})
    texts = {p: non_test(p.read_text(encoding='utf-8')) for p in paths}
    reachable = set()

    def visit(path):
        path = path.resolve()
        if path in reachable or not path.is_file():
            return
        if not path.resolve().is_relative_to(root.resolve()):
            raise ValueError('library module escapes checkout root')
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


class ManifestError(ValueError):
    pass


def manifests(root):
    result = []
    for p in sorted(root.rglob('Cargo.toml')):
        if '.git' in p.relative_to(root).parts:
            continue
        if not p.resolve().is_relative_to(root.resolve()):
            raise ValueError('manifest symlink escapes checkout')
        try:
            manifest = tomllib.loads(p.read_text(encoding='utf-8'))
        except (tomllib.TOMLDecodeError,UnicodeDecodeError) as error:
            raise ManifestError(f'{p.relative_to(root)}:{error}') from error
        result.append((p,manifest))
    return result


def source_texts(root):
    """All package source supersets; positive evidence requires a reached entry."""
    sources, reached = {}, set()
    for manifest_path, manifest in manifests(root):
        if 'package' not in manifest:
            continue
        package = manifest_path.parent
        entries = []
        lib = package / manifest.get('lib', {}).get('path', 'src/lib.rs')
        if lib.is_file() and ('lib' in manifest or manifest.get('package', {}).get('autolib', True)):
            entries.append(lib)
        if manifest.get('package', {}).get('autobins', True):
            if (package/'src/main.rs').is_file():
                entries.append(package/'src/main.rs')
            entries.extend(sorted((package/'src/bin').glob('*.rs')))
            entries.extend(sorted((package/'src/bin').glob('*/main.rs')))
        for binary in manifest.get('bin', []):
            name = binary.get('name', manifest['package'].get('name', ''))
            default = 'src/main.rs' if name == manifest['package'].get('name') else f'src/bin/{name}.rs'
            entry = package / binary.get('path', default)
            if not entry.is_file():
                entry = package / f'src/bin/{name}/main.rs'
            if entry.is_file():
                entries.append(entry)
        for entry in sorted(set(entries)):
            if not entry.resolve().is_relative_to(root.resolve()):
                raise ValueError('source entry escapes checkout')
            found, reachable = library_sources(root, {'lib': {'path': entry.relative_to(root).as_posix()}})
            sources.update(found)
            reached.update(reachable)
    return sorted(sources.items()), reached


def dependencies(root):
    all_manifests = dict(manifests(root))
    resolved_manifests = {p.resolve():m for p,m in all_manifests.items()}
    forbidden = []
    unresolved = []
    for path, manifest in all_manifests.items():
        workspace = manifest.get('package', {}).get('workspace')
        candidates = [path.parent/workspace/'Cargo.toml'] if workspace else [p/'Cargo.toml' for p in [path.parent, *path.parent.parents] if p.is_relative_to(root)]
        inherited = {}
        for parent in candidates:
            parent = parent.resolve()
            if parent in resolved_manifests and 'workspace' in resolved_manifests[parent]:
                inherited = resolved_manifests[parent]['workspace'].get('dependencies', {})
                break
        normal = [manifest.get('dependencies', {})]
        normal += [target.get('dependencies', {}) for target in manifest.get('target', {}).values()]
        for deps in normal:
            resolved = {}
            for name, spec in deps.items():
                if isinstance(spec, dict) and spec.get('workspace'):
                    if name not in inherited:
                        unresolved.append(f'{path.relative_to(root)}:{name}')
                        continue
                    spec = inherited[name]
                resolved[name] = spec
            forbidden.extend((path.relative_to(root).as_posix(), name) for name in forbidden_dependencies({'dependencies': resolved}))
    if not forbidden and unresolved:
        raise ValueError('unresolved workspace dependency '+unresolved[0])
    return sorted(set(forbidden))


def inspect(root):
    root = root.resolve()
    try:
        available = manifests(root)
        deps = dependencies(root)
    except ManifestError as error:
        return 'unscannable-manifest:'+str(error), '', 0
    if not available:
        return 'no-Cargo-manifest', '', 0
    if deps:
        path, name = deps[0]
        return f'forbidden-normal-dependency:{name}:{path}', '', 0
    sources, reachable = source_texts(root)
    hits = [(path, hit) for path, text in sources if path in reachable for hit in concurrency(text)]
    if not sources:
        return 'no-production-source', '', 0
    if not hits:
        return 'no-std-concurrency-use', '', len(sources)
    for label, pattern in FORBIDDEN:
        for path, text in sources:
            if re.search(pattern, text):
                return f'forbidden-source:{label}:{path}', '', len(sources)
    for path, text in sources:
        if unsafe_send_sync(text):
            return f'forbidden-source:unsafe-impl-Send-or-Sync:{path}', '', len(sources)
    return '', ';'.join(path+':'+hit for path, hit in hits), len(sources)


def self_test():
    import tempfile
    text, external = non_test('''// async fn comment() {}
const S: &str = r###"select! async fn f() {}"###;
use std::{sync::{Mutex as M, atomic::AtomicUsize}, thread::{self, spawn as go}};
#[cfg(test)] mod tests { async fn only_test() { x.await; } }
#[cfg(all(test, feature="x"))] mod extra;
#[cfg(not(test))] fn production() { let _ = M::new(0); }
''')
    assert external == {'extra'} and 'only_test' not in text
    assert not any(re.search(pattern, text) for _, pattern in FORBIDDEN)
    assert 'std::sync::Mutex' in concurrency(text)
    assert concurrency('use std::thread as t; t :: scope(||{});') == ['std::thread::scope']
    assert not concurrency(mask_rust('// std::sync::Mutex\n const X:&str="std::sync::Mutex";'))
    assert unsafe_send_sync('use core::marker::Send as S; unsafe impl<T> S for X<T> {}')
    parameter, _ = non_test('fn f(#[cfg(test)] x: usize) { let _ = std::sync::Mutex::new(()); }')
    assert 'x: usize' not in parameter and 'std::sync::Mutex' in parameter
    assert parameter.count('(') == parameter.count(')')
    with tempfile.TemporaryDirectory() as tmp:
        root = pathlib.Path(tmp).resolve()
        (root/'app/src/bin').mkdir(parents=True)
        (root/'support/src').mkdir(parents=True)
        (root/'Cargo.toml').write_text('[workspace]\nmembers=["app","support"]\n[workspace.dependencies]\nrenamed={package="tokio",version="1"}\n')
        (root/'app/Cargo.toml').write_text('[package]\nname="app"\nversion="0.1.0"\n')
        (root/'app/src/main.rs').write_text('mod helper; #[cfg(test)] mod test_helpers;')
        (root/'app/src/helper.rs').write_text('use std::sync::Mutex;')
        (root/'app/src/test_helpers.rs').write_text('async fn ignored(){}')
        (root/'support/Cargo.toml').write_text('[package]\nname="support"\nversion="0.1.0"\n')
        (root/'support/src/lib.rs').write_text('fn ordinary(){}')
        assert inspect(root)[0] == '', inspect(root)
        (root/'support/src/lib.rs').write_text('pub async fn bad(){}')
        assert inspect(root)[0].startswith('forbidden-source:async-fn'), inspect(root)
        (root/'support/src/lib.rs').write_text('fn ordinary(){}')
        with (root/'support/Cargo.toml').open('a') as f:
            f.write('[target.\'cfg(unix)\'.dependencies]\nrenamed={workspace=true}\n')
        assert dependencies(root) == [('support/Cargo.toml','tokio')]
        assert inspect(root)[0].startswith('forbidden-normal-dependency:tokio')
        (root/'support/src/lib.rs').write_text('fn unfinished(')
        assert inspect(root)[0].startswith('forbidden-normal-dependency:tokio')
        with (root/'support/Cargo.toml').open('a') as f:
            f.write('[dependencies]\nmissing={workspace=true}\n')
        assert dependencies(root) == [('support/Cargo.toml','tokio')]
        (root/'support/Cargo.toml').write_text('[package]\nname="support"\nversion="0.1.0"\n[dependencies]\nmissing={workspace=true}\n')
        try:
            dependencies(root)
        except ValueError as error:
            assert 'unresolved workspace dependency' in str(error)
        else:
            raise AssertionError('unknown dependency was silently accepted')
        (root/'Cargo.toml').write_text('[package]\nname="one"\nname="two"\n')
        assert inspect(root)[0].startswith('unscannable-manifest:Cargo.toml:')
    print('self-test: passed')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--self-test',action='store_true')
    p.add_argument('--repo',type=pathlib.Path)
    a = p.parse_args()
    if a.self_test:
        self_test()
    else:
        reason,evidence,count = inspect(a.repo)
        print(json.dumps(dict(outcome='excluded' if reason else 'kept',first_reason=reason,std_evidence=evidence,source_files=count),sort_keys=True))
