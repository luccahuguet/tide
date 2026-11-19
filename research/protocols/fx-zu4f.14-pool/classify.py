#!/usr/bin/env python3
"""fx-zu4f.14 crate-level classifier: PQ-evidence gate, then classes C and P; Python >=3.11 stdlib.

fx-zu4f.12's scan.py (classes C and P, unchanged) with one registered
narrowing in front, in two steps (amendment R1): a package is classified
only when it shows a lexical post-quantum discovery token (`pq_evidence`,
the registration's gate, unchanged) AND recorded evidence that the package
itself implements a listed algorithm or uses a dedicated post-quantum
implementation crate in its own code (`pq_impl_evidence`, see PQ_STRICT and
pq_impl_evidence). Lexical and mechanical only: it reads Cargo.toml and .rs
files of pinned sparse checkouts and never builds, runs or installs
anything. The masking, test-item removal, use-tree and module-walk helpers
are fx-zu4f.12's (from fx-zu4f.11 and fx-zu4f.10).

  python3 -I classify.py --self-test
  python3 -I classify.py --clones TARGET/clones --out WORK   # reads candidates.tsv,
      clone-refs.tsv; writes classes.tsv, repositories.tsv and pool.tsv
  python3 -I classify.py --check-narrowing OLD_classes.tsv --out WORK
      # every package R1 admits was admitted with the same classes before
"""
import argparse
import csv
import pathlib
import re
import subprocess
import tomllib

EXCLUDED_DIRS = {'tests', 'test', 'benches', 'examples', '.git', 'target'}
# Class C's source bans, in fx-zu4f.10's first-failure order.
FORBIDDEN = [
    ('Condvar', r'\bCondvar\b'),
    ('select!', r'\bselect\s*!'),
    ('async-fn', r'\basync\s+(?:unsafe\s+)?fn\b'),
    ('.await', r'\.\s*await\b'),
    ('Waker', r'\bWaker\b'),
]
FORBIDDEN_EXACT = {'tokio', 'futures', 'async-std', 'flume', 'loom'}
FORBIDDEN_PREFIX = ('crossbeam', 'parking_lot', 'rayon')
P_BANNED_STD = ('std::net', 'std::process', 'std::fs', 'std::thread')
UNSAFE_BLOCK_OR_FN = r'\bunsafe\s*\{|\bunsafe\s+(?:extern\s*)?fn\b|\bunsafe\s+extern\s*\{'
# PQ gate (narrowing): the bead's algorithms and protocols as lowercase
# alphanumeric tokens. A package passes when one appears, in this order, in
# its package name, a word of its description or keywords, a normal
# dependency's package name, or an identifier of a reached non-test file
# (comments and literals masked). Words and identifiers are compared after
# lowercasing and deleting every non-alphanumeric character.
PQ_TOKENS = ('mlkem', 'kyber', 'mldsa', 'dilithium', 'slhdsa', 'sphincs', 'fndsa', 'falcon', 'hqc',
             'mceliece', 'postquantum', 'rosenpass')
PQ_DEPENDENCIES = {'oqs', 'oqs-sys', 'pqcrypto'}  # PQ implementation crates whose names hold no token
# Implementation-evidence gate (amendment R1, a further narrowing). Strict
# names: a listed algorithm by its standard name, or an ambiguous family name
# only with its parameter set (kyber512, dilithium3, falcon512, hqc128), so
# that `falcon`, `hqc`, `kyber` or `dilithium` alone never suffice; protocol
# names and `postquantum` are discovery tokens only. Matched case-insensitively
# with `-`/`_` allowed inside a name, starting at a word start (see pq_name).
PQ_STRICT = re.compile(r'ml[-_]?kem|ml[-_]?dsa|slh[-_]?dsa|fn[-_]?dsa|sphincs|mc[-_]?eliece'
                       r'|kyber[-_]?(?:512|768|1024)|dilithium[-_]?[235]'
                       r'|falcon[-_]?(?:padded[-_]?)?(?:512|1024)|hqc[-_]?(?:128|192|256)')
# Dedicated PQ implementation crates whose package names hold no strict name.
PQ_DEDICATED = {'oqs', 'oqs-sys', 'pqcrypto', 'pqcrypto-kyber', 'pqcrypto-dilithium', 'pqcrypto-falcon',
                'pqcrypto-hqc', 'pqc_kyber', 'pqc_dilithium', 'crystals-dilithium', 'fips203', 'fips204',
                'fips205'}


def rows(path):
    with path.open(newline='') as f:
        return list(csv.DictReader(f, delimiter='\t'))


def table(path, fields, data):
    with path.open('w', newline='') as f:
        w = csv.DictWriter(f, fields, delimiter='\t', lineterminator='\n')
        w.writeheader()
        w.writerows(data)


def mask_rust(text, literals=True):
    """Blank nested comments and (unless literals=False) Rust literals, retaining offsets."""
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
        if literals or text.startswith(('//', '/*'), start):
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
    """Remove definitely test-only items; return (text, external test module names, removed ranges)."""
    masked = mask_rust(text)
    for match in re.finditer(r'#!\s*\[\s*cfg\s*\(', masked):
        end = balanced_end(masked, masked.index('[', match.start()), '[', ']')
        attr = masked[match.start():end]
        if cfg_without_test(attr[attr.find('(')+1:attr.rfind(')')]) == {False}:
            return re.sub(r'[^\n]', ' ', masked), set(), [(0, len(masked))]
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
    return ''.join(out), external, removed


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


def forbidden_dependencies(deps):
    names = set()
    for alias, spec in deps.items():
        name = spec.get('package', alias) if isinstance(spec, dict) else alias
        if name in FORBIDDEN_EXACT or name.startswith(FORBIDDEN_PREFIX):
            names.add(name)
    return sorted(names)


def unsafe_send_sync(text):
    aliases = [alias for path, alias in use_paths(text)
               if path in {'std::marker::Send', 'std::marker::Sync', 'core::marker::Send', 'core::marker::Sync'}]
    traits = '|'.join(re.escape(name) for name in ['Send', 'Sync'] + aliases)
    return re.search(r'\bunsafe\s+impl\b[^;{}]*?\b(?:' + traits + r')\s+for\b', text)


def std_references(text):
    """Canonical std paths named by use trees, aliases and qualified paths (lexical)."""
    paths = use_paths(text)
    refs = [p for p, _ in paths]
    aliases = {alias: path for path, alias in paths if alias != '*'}
    for match in re.finditer(r'\b\w+(?:\s*::\s*\w+)+', text):
        head, _, tail = re.sub(r'\s', '', match[0]).partition('::')
        refs.append(aliases.get(head, head) + '::' + tail)
    return refs


def ambient_p(text):
    """First std::net/process/fs/thread path (class P's ban), else None."""
    for ref in std_references(text):
        for prefix in P_BANNED_STD:
            if ref == prefix or ref.startswith(prefix + '::'):
                return prefix
    return None


class Unscannable(ValueError):
    pass


def read(path):
    try:
        return path.read_text(encoding='utf-8')
    except UnicodeDecodeError as error:
        raise Unscannable(f'non-UTF-8 source: {error}') from error


def load_manifests(root):
    """{manifest path: parsed TOML or Unscannable} for every Cargo.toml in the checkout."""
    result = {}
    for p in sorted(root.rglob('Cargo.toml')):
        rel = p.relative_to(root)
        if '.git' in rel.parts or 'target' in rel.parts[:-1]:
            continue
        if not p.resolve().is_relative_to(root):
            result[p] = Unscannable('manifest symlink escapes checkout')
            continue
        try:
            result[p] = tomllib.loads(read(p))
        except (tomllib.TOMLDecodeError, Unscannable) as error:
            result[p] = Unscannable(f'{rel}: {error}')
    return result


def packages(root, manifests):
    """Every Cargo package in the checkout: a Cargo.toml with [package] (or one that
    fails to parse), outside tests/, test/, benches/, examples/, target/ and .git/."""
    out = []
    for p, m in manifests.items():
        rel = p.relative_to(root)
        if set(rel.parts[:-1]) & EXCLUDED_DIRS:
            continue
        if isinstance(m, Unscannable) or 'package' in m:
            out.append(p)
    return out


def dependency_specs(manifest):
    tables = [manifest.get(k, {}) for k in ('dependencies', 'dev-dependencies', 'build-dependencies')]
    for t in manifest.get('target', {}).values():
        if isinstance(t, dict):
            tables += [t.get(k, {}) for k in ('dependencies', 'dev-dependencies', 'build-dependencies')]
    return [spec for t in tables if isinstance(t, dict) for spec in t.items()]


def workspace_members(root, manifests):
    """Package directories that belong to a workspace, by Cargo's rules: a workspace
    root's own package, its `members` globs, and path dependencies of members inside
    the root directory, minus `exclude`; a package with no ancestor [workspace], or
    one its nearest ancestor workspace excludes, is its own workspace root."""
    packages = {p.parent.resolve() for p, m in manifests.items() if isinstance(m, dict) and 'package' in m}
    member = set()
    for wpath, w in manifests.items():
        if not (isinstance(w, dict) and 'workspace' in w):
            continue
        wdir = wpath.parent.resolve()
        ws = w['workspace']
        excluded = [(wdir / e).resolve() for e in ws.get('exclude', []) if isinstance(e, str)]
        inherited = ws.get('dependencies', {})
        found = {wdir} & packages
        for pattern in ws.get('members', []):
            if not isinstance(pattern, str):
                continue
            for d in (wdir.glob(pattern.rstrip('/')) if pattern.rstrip('/') not in ('', '.') else [wdir]):
                if d.resolve() in packages and not any(d.resolve().is_relative_to(e) for e in excluded):
                    found.add(d.resolve())
        queue = list(found)
        while queue:
            d = queue.pop()
            m = manifests.get(d / 'Cargo.toml')
            if not isinstance(m, dict):
                continue
            for name, spec in dependency_specs(m):
                base = d
                if isinstance(spec, dict) and spec.get('workspace') is True and isinstance(inherited.get(name), dict):
                    spec, base = inherited[name], wdir
                if isinstance(spec, dict) and isinstance(spec.get('path'), str):
                    dep = (base / spec['path']).resolve()
                    if (dep in packages and dep not in found and dep.is_relative_to(wdir)
                            and not any(dep.is_relative_to(e) for e in excluded)):
                        found.add(dep)
                        queue.append(dep)
        member |= found
    for d in packages - member:
        nearest = next((manifests.get(a / 'Cargo.toml') for a in d.parents
                        if a.is_relative_to(root) and isinstance(manifests.get(a / 'Cargo.toml'), dict)
                        and 'workspace' in manifests[a / 'Cargo.toml']), None)
        own = manifests.get(d / 'Cargo.toml')
        if nearest is None or 'workspace' in own:
            member.add(d)
        else:
            adir = next(a for a in d.parents if manifests.get(a / 'Cargo.toml') is nearest)
            if any(d.is_relative_to((adir / e).resolve()) for e in nearest['workspace'].get('exclude', []) if isinstance(e, str)):
                member.add(d)
    return member


def workspace_root(root, manifest_path, manifest, manifests):
    explicit = manifest.get('package', {}).get('workspace')
    candidates = ([manifest_path.parent / explicit / 'Cargo.toml'] if isinstance(explicit, str)
                  else [d / 'Cargo.toml' for d in [manifest_path.parent, *manifest_path.parent.parents]
                        if d.is_relative_to(root)])
    for c in candidates:
        m = manifests.get(c) if c in manifests else manifests.get(c.resolve())
        if isinstance(m, dict) and 'workspace' in m:
            return m
        if isinstance(m, Unscannable) and c != manifest_path:
            return m
    return None


def normal_dependencies(root, manifest_path, manifest, manifests):
    """(forbidden, all, {Rust name: package}) normal and target dependencies, resolving `workspace = true`."""
    tables = [manifest.get('dependencies', {})]
    tables += [t.get('dependencies', {}) for t in manifest.get('target', {}).values() if isinstance(t, dict)]
    ws = None
    forbidden, names, rust_names = set(), set(), {}
    for deps in tables:
        resolved = {}
        for name, spec in deps.items():
            if isinstance(spec, dict) and spec.get('workspace') is True:
                if ws is None:
                    ws = workspace_root(root, manifest_path, manifest, manifests)
                if isinstance(ws, Unscannable):
                    raise Unscannable(f'workspace manifest unparsable: {ws}')
                inherited = (ws or {}).get('workspace', {}).get('dependencies', {})
                if name not in inherited:
                    raise Unscannable(f'unresolved workspace dependency {name}')
                spec = inherited[name]
            resolved[name] = spec
        forbidden.update(forbidden_dependencies(resolved))
        names.update(spec.get('package', alias) if isinstance(spec, dict) else alias
                     for alias, spec in resolved.items())
        rust_names.update((alias.replace('-', '_'), spec.get('package', alias) if isinstance(spec, dict) else alias)
                          for alias, spec in resolved.items())
    return sorted(forbidden), sorted(names), rust_names


def entries(pkg_dir, manifest):
    """Library and binary targets (never test, bench or example targets)."""
    found = []
    pkg = manifest.get('package', {})
    lib = manifest.get('lib', {})
    lib_path = pkg_dir / lib.get('path', 'src/lib.rs')
    if lib_path.is_file() and ('lib' in manifest or pkg.get('autolib', True)):
        found.append(lib_path)
    if pkg.get('autobins', True):
        if (pkg_dir / 'src/main.rs').is_file():
            found.append(pkg_dir / 'src/main.rs')
        found += sorted((pkg_dir / 'src/bin').glob('*.rs'))
        found += sorted((pkg_dir / 'src/bin').glob('*/main.rs'))
    for b in manifest.get('bin', []):
        name = b.get('name', pkg.get('name', ''))
        path = pkg_dir / b['path'] if 'path' in b else (
            pkg_dir / 'src/main.rs' if name == pkg.get('name') else pkg_dir / f'src/bin/{name}.rs')
        if not path.is_file() and 'path' not in b:
            path = pkg_dir / f'src/bin/{name}/main.rs'
        if path.is_file():
            found.append(path)
    has_bin = any(e != lib_path for e in found)
    return sorted({e.resolve() for e in found}), has_bin


def owned(path, pkg_dir, nested):
    """True when path belongs to this package: not in a nested package or excluded dir."""
    rel = path.relative_to(pkg_dir)
    if set(rel.parts[:-1]) & EXCLUDED_DIRS:
        return False
    return not any(path.is_relative_to(n) for n in nested)


def package_sources(root, pkg_dir, entry_files, nested):
    """(superset {path: (non-test text, removed ranges)}, reached files) for one package.

    The superset is every .rs file under each entry's parent directory that the
    package owns, minus build.rs and external modules declared test-only; the
    module walk from the entries supplies the reached files."""
    superset = {}
    for entry in entry_files:
        if not entry.is_relative_to(root):
            raise Unscannable('source entry escapes checkout')
        for p in sorted(entry.parent.rglob('*.rs')):
            if p.name == 'build.rs' or not p.resolve().is_relative_to(root):
                continue
            if p.is_relative_to(pkg_dir) and not owned(p, pkg_dir, nested):
                continue
            if any(p.is_relative_to(n) for n in nested):
                continue
            superset[p.resolve()] = None
    texts = {}

    def text_of(p):
        if p not in texts:
            try:
                texts[p] = non_test(read(p))
            except ValueError as error:
                raise Unscannable(f'{p.relative_to(root)}: {error}') from error
        return texts[p]

    for p in superset:
        text_of(p)
    reached = set()
    attr_re = re.compile(r'#!?\s*\[')
    mod_re = re.compile(r'(?:pub(?:\([^)]*\))?\s+)?(?:unsafe\s+)?mod\s+(\w+)\s*([;{])')
    word_re = re.compile(r'\w+')

    def visit(path, entry):
        path = path.resolve()
        if path in reached or not path.is_file():
            return
        if not path.is_relative_to(root):
            raise Unscannable('module escapes checkout')
        reached.add(path)
        original = read(path)
        text = text_of(path)[0]

        def walk(start, end, base):
            i = start
            attributes = []
            while i < end:
                if text[i].isspace():
                    i += 1
                    continue
                if text[i] == '#' and (attr := attr_re.match(text, i)):
                    stop = balanced_end(text, attr.end() - 1, '[', ']')
                    attributes.append(original[i:stop])
                    i = stop
                    continue
                module = mod_re.match(text, i)
                if module:
                    name, delimiter = module.groups()
                    opening = module.end() - 1
                    if delimiter == '{':
                        stop = balanced_end(text, opening)
                        walk(opening + 1, stop - 1, base / name)
                        i = stop
                    else:
                        custom = re.search(r'#\s*\[\s*path\s*=\s*"([^"\n]+)"\s*\]', '\n'.join(attributes))
                        options = [path.parent / custom[1]] if custom else [base / (name + '.rs'), base / name / 'mod.rs']
                        for child in options:
                            if child.is_file():
                                visit(child, False)
                                break
                        i = opening + 1
                    attributes = []
                    continue
                attributes = []
                if text[i] in '{([':
                    opening = text[i]
                    i = balanced_end(text, i, opening, {'{': '}', '(': ')', '[': ']'}[opening])
                elif word := word_re.match(text, i):
                    i = word.end()
                else:
                    i += 1
        base = path.parent if entry or path.name == 'mod.rs' else path.with_suffix('')
        try:
            walk(0, len(text), base)
        except ValueError as error:
            raise Unscannable(f'{path.relative_to(root)}: {error}') from error

    for e in entry_files:
        visit(e, True)
    excluded = set()
    for p, (_, modules, _) in list(texts.items()):
        if p not in superset and p not in reached:
            continue
        base = p.parent if p in entry_files or p.name == 'mod.rs' else p.with_suffix('')
        for name in modules:
            child = (base / name).resolve()
            excluded.update(q for q in superset if q == child.with_suffix('.rs') or q.is_relative_to(child))
    excluded -= reached
    files = {p: texts[p] for p in sorted(set(superset) | reached) if p not in excluded}
    return files, reached


def nontest_lines(original, removed):
    """Lines with non-blank text after removing comments and test-only items
    (string and character literals count as code)."""
    text = list(mask_rust(original, literals=False))
    for a, b in removed:
        text[a:b] = ['\n' if c == '\n' else ' ' for c in text[a:b]]
    return sum(1 for line in ''.join(text).split('\n') if line.strip())


def own_test_attributes(root, pkg_dir, nested):
    """#[test] attributes in any .rs file the package directory owns (src/ and tests/ alike)."""
    count, first = 0, ''
    for p in sorted(pkg_dir.rglob('*.rs')):
        rel = p.relative_to(pkg_dir)
        if {'target', '.git'} & set(rel.parts) or any(p.is_relative_to(n) for n in nested):
            continue
        if not p.resolve().is_relative_to(root):
            continue
        try:
            n = len(re.findall(r'#\s*\[\s*test\s*\]', mask_rust(read(p))))
        except (Unscannable, ValueError):
            continue
        if n and not first:
            first = rel.as_posix()
        count += n
    return count, first


def pq_normal(word):
    return re.sub(r'[^0-9a-z]', '', word.lower())


def pq_token(word):
    w = pq_normal(word)
    return next((t for t in PQ_TOKENS if t in w), None)


def pq_evidence(manifest, dep_names, reached_texts):
    """First PQ evidence for a package (see PQ_TOKENS), or None."""
    pkg = manifest.get('package', {})
    if isinstance(pkg.get('name'), str) and (t := pq_token(pkg['name'])):
        return f'name:{t}'
    words = pkg.get('description', '') if isinstance(pkg.get('description'), str) else ''
    words = words.split() + [k for k in pkg.get('keywords', []) if isinstance(k, str)]
    for w in words:
        if t := pq_token(w):
            return f'manifest:{t}'
    for name in dep_names:
        if name in PQ_DEPENDENCIES or (t := pq_token(name)):
            return f'dependency:{name}'
    for rel, text in reached_texts:
        for ident in re.findall(r'\b[A-Za-z_]\w*\b', text):
            if t := pq_token(ident):
                return f'identifier:{rel}:{ident}'
    return None


def pq_name(text):
    """First strict PQ name (PQ_STRICT) in text that starts a word, normalized (`-`/`_` dropped), or None.

    A match starts a word when it opens the text, follows a non-letter, or is an
    uppercase letter after a lowercase one (camel case): `X25519MLKEM768`,
    `HybridMlKem` and `ml-kem-768` count; `XmlDsa`, `HTMLDSA` and `falcon_h1` do not."""
    text = text.encode('ascii', 'replace').decode()
    low = text.lower()
    for m in PQ_STRICT.finditer(low):
        i = m.start()
        if i == 0 or not low[i-1].isalpha() or (text[i].isupper() and text[i-1].islower()):
            return re.sub(r'[-_]', '', m[0])
    return None


def op_name(name):
    """True when a fn name names a key generation, (de)capsulation, signing or verification operation."""
    words = [w.lower() for w in re.findall(r'[A-Z]+(?![a-z])|[A-Z]?[a-z]+|[0-9]+', name)]
    joined = ''.join(words)
    return (any(t in joined for t in ('keygen', 'keypair', 'encap', 'decap')) or bool({'sign', 'verify'} & set(words))
            or bool({'key', 'keys'} & set(words)) and bool({'gen', 'generate'} & set(words)))


def pq_impl_evidence(reached, rust_names):
    """First implementation evidence for a package, or None (amendment R1).

    `reached` is [(path from the repository root, path from the package
    directory, masked non-test text)] for the reached files in path order;
    `rust_names` maps each normal or target dependency's Rust name to its
    package. Two kinds, the first found in this order:
    own-op   a `fn` whose name is an operation (op_name) and whose context holds
             a strict PQ name (pq_name): the fn name, the header of an enclosing
             `impl`, `trait` or inline `mod` block, or a component of the file's
             path in the package; recorded as own-op:FILE:LINE:CONTEXT:FN.
    dep-use  a path (`use` or inline) whose first segment is the Rust name of a
             dependency whose package is a dedicated PQ implementation crate
             (pq_name holds for its name, or it is in PQ_DEDICATED); recorded
             as dep-use:FILE:LINE:PACKAGE.
    A discovery token alone (a name, description, keyword, dependency entry or
    identifier) never suffices."""
    def line(text, pos):
        return text.count('\n', 0, pos) + 1

    for rel, rel_pkg, text in reached:
        parts = pathlib.PurePosixPath(rel_pkg).with_suffix('').parts
        path_name = next((pq_name(c) for c in parts if pq_name(c)), None)
        blocks = []
        for m in re.finditer(r'\b(?:impl|trait|mod)\b[^{;]*\{', text):
            try:
                blocks.append((m.end() - 1, balanced_end(text, m.end() - 1), text[m.start():m.end() - 1]))
            except ValueError:
                continue
        for m in re.finditer(r'\bfn\s+([A-Za-z_]\w*)', text, re.ASCII):
            if not op_name(m[1]):
                continue
            context = pq_name(m[1]) and m[1]
            for start, end, header in blocks:
                if not context and start < m.start() < end and pq_name(header):
                    context = ' '.join(header.split())[:100]
            context = context or (path_name and rel_pkg)
            if context:
                return f'own-op:{rel}:{line(text, m.start())}:{context}:{m[1]}'
    dedicated = {r: p for r, p in rust_names.items() if p in PQ_DEDICATED or pq_name(p)}
    if dedicated:
        names = '|'.join(re.escape(r) for r in sorted(dedicated))
        pattern = re.compile(rf'(?<![\w:])(?:::\s*)?({names})\s*::|\buse\s+(?:::\s*)?({names})\b|'
                             rf'\bextern\s+crate\s+({names})\b')
        for rel, _, text in reached:
            if m := pattern.search(text):
                return f'dep-use:{rel}:{line(text, m.start())}:{dedicated[m[1] or m[2] or m[3]]}'
    return None


def classify(root, manifest_path, manifest, manifests, nested, members):
    """Return the scan row fields for one package."""
    pkg_dir = manifest_path.parent
    row = dict(crate='-', crate_path=pkg_dir.relative_to(root).as_posix() or '.', has_bin='-',
               source_files=0, reached_files=0, nontest_lines=0, test_attrs=0, std_evidence='-',
               pq_evidence='-', pq_impl_evidence='-')
    if isinstance(manifest, Unscannable):
        reason = f'unscannable-manifest:{manifest}'
        return dict(row, c_reason=reason, p_reason=reason)
    row['crate'] = manifest['package'].get('name', '-')
    if manifest_path.parent.resolve() not in members:
        return dict(row, c_reason='not-a-workspace-member', p_reason='not-a-workspace-member')
    try:
        deps, dep_names, rust_names = normal_dependencies(root, manifest_path, manifest, manifests)
        entry_files, has_bin = entries(pkg_dir, manifest)
        row['has_bin'] = 'yes' if has_bin else 'no'
        if not entry_files:
            return dict(row, c_reason='no-production-source', p_reason='no-production-source')
        files, reached = package_sources(root, pkg_dir, entry_files, nested)
    except Unscannable as error:
        reason = f'unscannable:{error}'
        return dict(row, c_reason=reason, p_reason=reason)
    rel = {p: p.relative_to(root).as_posix() for p in files}
    row.update(source_files=len(files), reached_files=len(reached))
    row['pq_evidence'] = pq_evidence(manifest, dep_names,
                                     [(rel[p], files[p][0]) for p in sorted(reached) if p in files]) or '-'
    if row['pq_evidence'] == '-':
        return dict(row, c_reason='no-pq-evidence', p_reason='no-pq-evidence')
    row['pq_impl_evidence'] = pq_impl_evidence(
        [(rel[p], p.relative_to(pkg_dir.resolve()).as_posix() if p.is_relative_to(pkg_dir.resolve()) else rel[p],
          files[p][0]) for p in sorted(reached) if p in files], rust_names) or '-'
    if row['pq_impl_evidence'] == '-':
        return dict(row, c_reason='no-pq-implementation-evidence', p_reason='no-pq-implementation-evidence')
    row['nontest_lines'] = sum(nontest_lines(read(p), files[p][2]) for p in files if p in reached)
    row['test_attrs'], test_file = own_test_attributes(root, pkg_dir, nested)

    # Class C: fx-zu4f.10's filter at crate level, in its first-failure order.
    hits = [f'{rel[p]}:{h}' for p in sorted(reached) if p in files for h in concurrency(files[p][0])]
    c_reason = ''
    if not hits:
        c_reason = 'no-std-concurrency-use'
    if not c_reason:
        for label, pattern in FORBIDDEN:
            hit = next((rel[p] for p in files if re.search(pattern, files[p][0])), None)
            if hit:
                c_reason = f'forbidden-source:{label}:{hit}'
                break
    if not c_reason and deps:
        c_reason = f'forbidden-normal-dependency:{deps[0]}'
    if not c_reason:
        hit = next((rel[p] for p in files if unsafe_send_sync(files[p][0])), None)
        if hit:
            c_reason = f'forbidden-source:unsafe-impl-Send-or-Sync:{hit}'

    # Class P, in the bead's order.
    p_reason = ''
    if deps:
        p_reason = f'forbidden-normal-dependency:{deps[0]}'
    if not p_reason:
        hit = next((rel[p] for p in files if re.search(r'\basync\b|\.\s*await\b', files[p][0])), None)
        if hit:
            p_reason = f'async:{hit}'
    if not p_reason:
        for p in files:
            if prefix := ambient_p(files[p][0]):
                p_reason = f'{prefix}:{rel[p]}'
                break
    if not p_reason:
        hit = next((rel[p] for p in files if re.search(UNSAFE_BLOCK_OR_FN, files[p][0])), None)
        if hit:
            p_reason = f'unsafe-block-or-fn:{hit}'
    if not p_reason and not 200 <= row['nontest_lines'] <= 5000:
        p_reason = f'nontest-lines-outside-200-5000:{row["nontest_lines"]}'
    if not p_reason and row['test_attrs'] == 0:
        p_reason = 'no-own-#[test]'
    if not c_reason:
        row['std_evidence'] = ';'.join(hits[:5]) + (f';+{len(hits)-5} more' if len(hits) > 5 else '')
    elif not p_reason:
        row['std_evidence'] = f'first #[test] in {test_file}'
    return dict(row, c_reason=c_reason or '-', p_reason=p_reason or '-')


def scan_repository(root):
    root = root.resolve()
    manifests = load_manifests(root)
    pkgs = packages(root, manifests)
    package_dirs = [p.parent for p in manifests if isinstance(manifests[p], Unscannable) or 'package' in manifests[p]]
    members = workspace_members(root, manifests)
    rows = []
    for p in pkgs:
        nested = [d for d in package_dirs if d != p.parent and d.is_relative_to(p.parent)]
        rows.append(classify(root, p, manifests[p], manifests, nested, members))
    if not rows:
        rows.append(dict(crate='-', crate_path='-', has_bin='-', source_files=0, reached_files=0, nontest_lines=0,
                         test_attrs=0, std_evidence='-', pq_evidence='-', pq_impl_evidence='-',
                         c_reason='no-Cargo-package',
                         p_reason='no-Cargo-package'))
    return rows


FIELDS = ['repository', 'stars', 'head_commit', 'crate', 'crate_path', 'class', 'first_reason',
          'c_reason', 'p_reason', 'has_bin', 'source_files', 'reached_files', 'nontest_lines',
          'test_attrs', 'pq_evidence', 'pq_impl_evidence', 'std_evidence']


def rank_key(row):
    return (-int(row['stars']), row['repository'].lower(), row['crate'], row['crate_path'])


def run(clones, out):
    candidates = {r['repository']: r for r in rows(out / 'candidates.tsv') if r['outcome'] == 'kept'}
    output = []
    for ref in rows(out / 'clone-refs.tsv'):
        repo = ref['repository']
        checkout = clones / repo.replace('/', '__')
        head = subprocess.run(['git', '-C', str(checkout), 'rev-parse', 'HEAD'], capture_output=True,
                              stdin=subprocess.DEVNULL, text=True, check=True).stdout.strip()
        if head != ref['head_commit']:
            raise SystemExit(f'{repo}: checkout HEAD {head} != pinned {ref["head_commit"]}')
        for r in scan_repository(checkout):
            c, p = r['c_reason'] == '-', r['p_reason'] == '-'
            r.update(repository=repo, stars=candidates[repo]['stars'], head_commit=head,
                     class_='C' if c else 'P' if p else '-')
            r['class'] = r.pop('class_')
            r['first_reason'] = '-' if c or p else (
                r['c_reason'] if r['c_reason'] == r['p_reason'] else f'C:{r["c_reason"]} | P:{r["p_reason"]}')
            output.append(r)
        print(repo, len(output), flush=True)
    output.sort(key=rank_key)
    table(out / 'classes.tsv', FIELDS, output)
    repos = []
    for ref in rows(out / 'clone-refs.tsv'):
        mine = [r for r in output if r['repository'] == ref['repository']]
        pq = sum(r['pq_evidence'] != '-' for r in mine)
        impl = sum(r['pq_impl_evidence'] != '-' for r in mine)
        kept = sum(r['class'] != '-' for r in mine)
        packages = sum(r['crate_path'] != '-' for r in mine)
        reason = ('no-Cargo-package' if not packages else 'no-pq-evidence-in-any-package' if not pq else
                  'no-pq-implementation-evidence-in-any-package' if not impl else '-')
        repos.append(dict(repository=ref['repository'], stars=candidates[ref['repository']]['stars'],
                          head_commit=ref['head_commit'], packages=packages, pq_packages=pq,
                          pq_impl_packages=impl, class_c_or_p=kept,
                          outcome='dropped' if reason != '-' else 'kept', drop_reason=reason))
    repos.sort(key=lambda r: (-int(r['stars']), r['repository'].lower()))
    table(out / 'repositories.tsv', ['repository', 'stars', 'head_commit', 'packages', 'pq_packages',
                                     'pq_impl_packages', 'class_c_or_p', 'outcome', 'drop_reason'], repos)
    pool, per_repo = [], {}
    for r in output:
        if r['class'] == '-' or per_repo.get(r['repository'], 0) >= 3:
            continue
        per_repo[r['repository']] = per_repo.get(r['repository'], 0) + 1
        pool.append(dict(pool_rank=len(pool) + 1, arm_a='yes' if len(pool) < 10 else 'no',
                         **{k: r[k] for k in ('class', 'repository', 'stars', 'crate', 'crate_path', 'head_commit',
                                              'nontest_lines', 'pq_evidence', 'pq_impl_evidence', 'std_evidence')},
                         also_p='yes' if r['class'] == 'C' and r['p_reason'] == '-' else 'no'))
        if len(pool) == 30:
            break
    table(out / 'pool.tsv', ['pool_rank', 'class', 'also_p', 'arm_a', 'repository', 'stars', 'crate',
                             'crate_path', 'head_commit', 'nontest_lines', 'pq_evidence', 'pq_impl_evidence',
                             'std_evidence'], pool)
    print(f'{len(output)} package rows; {sum(r["pq_evidence"] != "-" for r in output)} with PQ evidence; '
          f'{sum(r["pq_impl_evidence"] != "-" for r in output)} with implementation evidence in '
          f'{sum(r["outcome"] == "kept" for r in repos)} repositories; {sum(r["class"]=="C" for r in output)} C; '
          f'{sum(r["class"]=="P" for r in output)} P; pool {len(pool)}')


def check_narrowing(old_path, out):
    """Check that OUT/classes.tsv narrows OLD_classes.tsv (the registration's): the
    same package rows, the same discovery evidence, and every admitted row
    (implementation evidence) was admitted before with the same class and reasons."""
    def key(r):
        return (r['repository'], r['crate_path'], r['crate'])
    old = {key(r): r for r in rows(old_path)}
    new = {key(r): r for r in rows(out / 'classes.tsv')}
    assert len(old) == len(rows(old_path)) and len(new) == len(rows(out / 'classes.tsv')), 'duplicate keys'
    assert set(old) == set(new), sorted(set(old) ^ set(new))
    same = ('stars', 'head_commit', 'has_bin', 'source_files', 'reached_files', 'pq_evidence')
    widened, changed = [], []
    for k, n in new.items():
        o = old[k]
        changed += [(k, f) for f in same if n[f] != o[f]]
        if n['pq_impl_evidence'] != '-':
            if o['pq_evidence'] == '-' or any(n[f] != o[f] for f in ('class', 'c_reason', 'p_reason',
                                                                    'first_reason', 'nontest_lines',
                                                                    'test_attrs', 'std_evidence')):
                widened.append(k)
        elif n['class'] != '-' or (o['pq_evidence'] == '-' and n != {**o, 'pq_impl_evidence': '-'}):
            widened.append(k)
    old_admitted = sum(r['class'] != '-' for r in old.values())
    new_admitted = sum(r['class'] != '-' for r in new.values())
    print(f'narrowing check: {len(new)} rows; old PQ gate {sum(r["pq_evidence"] != "-" for r in old.values())}, '
          f'new gate {sum(r["pq_impl_evidence"] != "-" for r in new.values())}; class C or P {old_admitted} -> '
          f'{new_admitted}; changed fields {len(changed)}; widened {len(widened)}')
    if changed or widened:
        raise SystemExit(f'NOT a narrowing: changed {changed[:5]} widened {widened[:5]}')
    print('narrowing check: passed')


def self_test():
    import tempfile
    text, external, _ = non_test('''// async fn comment() {}
const S: &str = r###"select! async fn f() {}"###;
use std::{sync::{Mutex as M, atomic::AtomicUsize}, thread::{self, spawn as go}};
#[cfg(test)] mod tests { async fn only_test() { x.await; } }
#[cfg(all(test, feature="x"))] mod extra;
#[cfg(not(test))] fn production() { let _ = M::new(0); }
''')
    assert external == {'extra'} and 'only_test' not in text
    assert not any(re.search(pattern, text) for _, pattern in FORBIDDEN)
    assert 'std::sync::Mutex' in concurrency(text)
    assert ambient_p('use std::{fs as f}; fn x(){ f::read(""); }') == 'std::fs'
    assert ambient_p('fn x(){ ::std::process::exit(0) }') == 'std::process'
    assert ambient_p('std::thread_local!{} fn f(){ core::net::Ipv4Addr::LOCALHOST; }') is None
    assert re.search(UNSAFE_BLOCK_OR_FN, mask_rust('pub const unsafe fn f() {}'))
    assert re.search(UNSAFE_BLOCK_OR_FN, mask_rust('fn f() { unsafe { g() } }'))
    assert re.search(UNSAFE_BLOCK_OR_FN, mask_rust('unsafe extern "C" fn f() {}'))
    assert not re.search(UNSAFE_BLOCK_OR_FN, mask_rust('unsafe impl Send for X {} // unsafe { }'))
    assert nontest_lines('fn a() {}\n// c\n\nconst S: &str = "x\n\ny";\n#[cfg(test)]\nmod t {\n fn b() {}\n}\n',
                         non_test('fn a() {}\n// c\n\nconst S: &str = "x\n\ny";\n#[cfg(test)]\nmod t {\n fn b() {}\n}\n')[2]) == 3
    body = '\n'.join(f'pub fn mlkem_sign_f{i}(x: u64) -> u64 {{ x.wrapping_add({i}) }}' for i in range(250))
    plain = body.replace('mlkem_', '')
    assert pq_token('SPHINCS+') == 'sphincs' and pq_token('McEliece') == 'mceliece'
    assert pq_token('X25519MLKEM768') == 'mlkem' and pq_token('post-quantum') == 'postquantum'
    assert pq_token('signal') is None and pq_token('Classic') is None
    for name, want in [('X25519MLKEM768', 'mlkem'), ('HybridMlKem', 'mlkem'), ('ml-kem-768', 'mlkem'),
                       ('pqcrypto-mlkem', 'mlkem'), ('Falcon512', 'falcon512'), ('fn_dsa', 'fndsa'),
                       ('dilithium3', 'dilithium3'), ('hqc_128', 'hqc128'), ('SPHINCS+', 'sphincs'),
                       ('ClassicMcEliece', 'mceliece'), ('falcon_h1_capability', None), ('falcon_dma_valid', None),
                       ('Falcon', None), ('hqc', None), ('Kyber', None), ('KyberSwap', None), ('Dilithium', None),
                       ('XmlDsa', None), ('HTMLDSA', None), ('rosenpass', None), ('post-quantum', None),
                       ('frodo640', None), ('saber', None)]:
        assert pq_name(name) == want, (name, pq_name(name))
    assert all(op_name(n) for n in ('keygen', 'KeyGen', 'generate_keypair', 'generate_key', 'encapsulate', 'decaps',
                                    'sign', 'verify_signature', 'mlkem_sign_f0'))
    assert not any(op_name(n) for n in ('signal', 'assign', 'signature', 'capability', 'complete', 'key_size'))
    impl = pq_impl_evidence
    assert impl([('a/src/lib.rs', 'src/lib.rs', 'pub struct MlKem768;\nimpl Kem for MlKem768 {\n'
                  '    fn encapsulate(&self) {}\n}\n')], {}) == 'own-op:a/src/lib.rs:3:impl Kem for MlKem768:encapsulate'
    assert impl([('a/src/ml_kem/ops.rs', 'src/ml_kem/ops.rs', 'fn keygen() {}')], {}) == \
        'own-op:a/src/ml_kem/ops.rs:1:src/ml_kem/ops.rs:keygen'
    assert impl([('a/src/lib.rs', 'src/lib.rs', 'const ML_DSA_65_PUBLIC_KEY_SIZE: usize = 1952;\n'
                  'fn falcon_h1_capability() {}\nimpl Falcon { fn sign() {} }\nfn sign() {}')], {}) is None
    assert impl([('a/src/lib.rs', 'src/lib.rs', 'fn f() {\n    ::ml_kem::MlKem768::new();\n}')],
                {'ml_kem': 'ml-kem'}) == 'dep-use:a/src/lib.rs:2:ml-kem'
    assert impl([('a/src/lib.rs', 'src/lib.rs', 'use crate::ml_kem::X;\nuse aws_lc_rs::kem::ML_KEM_768;')],
                {'ml_kem': 'ml-kem', 'aws_lc_rs': 'aws-lc-rs', 'q': 'oqs'}) is None
    with tempfile.TemporaryDirectory() as tmp:
        root = pathlib.Path(tmp).resolve()
        def put(path, content):
            (root / path).parent.mkdir(parents=True, exist_ok=True)
            (root / path).write_text(content)
        put('Cargo.toml', '[workspace]\nmembers=["crates/*"]\n[workspace.dependencies]\nrt={package="tokio",version="1"}\n')
        put('crates/conc/Cargo.toml', '[package]\nname="conc"\nversion="0.1.0"\n')
        put('crates/conc/src/lib.rs', 'mod inner;\n#[cfg(test)] mod tests;\n')
        put('crates/conc/src/inner.rs', 'use std::sync::Mutex;\npub static KYBER_M: Mutex<u8> = Mutex::new(0);\n'
            'pub fn kyber768_keygen() {}\n')
        put('crates/conc/src/tests.rs', 'async fn ignored() { x.await }')
        put('crates/conc/tests/it.rs', 'async fn integration() {} #[test] fn t() {}')
        put('crates/logic/Cargo.toml', '[package]\nname="logic"\nversion="0.1.0"\n')
        put('crates/logic/src/lib.rs', body + '\n#[cfg(test)]\nmod tests { #[test] fn ok() { std::fs::read("x"); } }\n')
        put('crates/asyncy/Cargo.toml', '[package]\nname="asyncy"\nversion="0.1.0"\n[target.\'cfg(unix)\'.dependencies]\nrt={workspace=true}\n')
        put('crates/asyncy/src/main.rs', body)
        put('crates/small/Cargo.toml', '[package]\nname="small"\nversion="0.1.0"\n')
        put('crates/small/src/lib.rs', 'pub fn kyber512_sign() {}\n#[test] fn t() {}\n')
        put('crates/plain/Cargo.toml', '[package]\nname="plain"\nversion="0.1.0"\n')
        put('crates/plain/src/lib.rs', '// ML-KEM is only named in a comment\nconst S: &str = "kyber";\n'
            + plain + '\n#[test] fn t() {}\n')
        put('crates/pqdesc/Cargo.toml', '[package]\nname="pqdesc"\nversion="0.1.0"\n'
            'description="Classic McEliece KEM"\n')
        put('crates/pqdesc/src/lib.rs', plain + '\n#[test] fn t() {}\n')
        put('crates/pqdep/Cargo.toml', '[package]\nname="pqdep"\nversion="0.1.0"\n[dependencies]\nq={package="oqs",version="0.9"}\n')
        put('crates/pqdep/src/lib.rs', 'use q::kem::Kem;\n' + plain + '\n#[test] fn t() {}\n')
        put('crates/unsafe/Cargo.toml', '[package]\nname="uns"\nversion="0.1.0"\n')
        put('crates/unsafe/src/lib.rs', body + '\nfn g() { unsafe { h() } }\n#[test] fn t() {}\n')
        put('crates/notest/Cargo.toml', '[package]\nname="notest"\nversion="0.1.0"\n')
        put('crates/notest/src/lib.rs', body)
        put('crates/nested/Cargo.toml', '[package]\nname="outer"\nversion="0.1.0"\n[lib]\npath="lib.rs"\n')
        put('crates/nested/lib.rs', body + '\n#[test] fn t() {}\n')
        put('crates/nested/inner/Cargo.toml', '[package]\nname="inner"\nversion="0.1.0"\n')
        put('crates/nested/inner/src/lib.rs', 'pub async fn a() {}')
        put('examples/demo/Cargo.toml', '[package]\nname="demo"\nversion="0.1.0"\n')
        put('examples/demo/src/main.rs', 'fn main() {}')
        put('crates/broken/Cargo.toml', '[package]\nname="a"\nname="b"\n')
        put('Cargo.toml', '[workspace]\nmembers=["crates/*", "tools/used"]\nexclude=["crates/excl"]\n'
            '[workspace.dependencies]\nrt={package="tokio",version="1"}\nauto={path="extra/auto"}\n')
        put('crates/excl/Cargo.toml', '[package]\nname="excl"\nversion="0.1.0"\n')
        put('crates/excl/src/lib.rs', body + '\n#[test] fn t() {}\n')
        put('orphan/Cargo.toml', '[package]\nname="orphan"\nversion="0.1.0"\n')
        put('orphan/src/lib.rs', body + '\n#[test] fn t() {}\n')
        put('tools/used/Cargo.toml', '[package]\nname="used"\nversion="0.1.0"\n[dependencies]\nauto={workspace=true}\n')
        put('tools/used/src/lib.rs', 'pub fn f() {}')
        put('extra/auto/Cargo.toml', '[package]\nname="auto"\nversion="0.1.0"\n')
        put('extra/auto/src/lib.rs', body + '\n#[test] fn t() {}\n')
        got = {r['crate']: r for r in scan_repository(root)}
        assert set(got) == {'conc', 'logic', 'asyncy', 'small', 'uns', 'notest', 'outer', 'inner', '-', 'excl',
                            'orphan', 'used', 'auto', 'plain', 'pqdesc', 'pqdep'}, set(got)
        assert got['plain']['c_reason'] == got['plain']['p_reason'] == 'no-pq-evidence', got['plain']
        assert got['pqdesc']['p_reason'] == 'no-pq-implementation-evidence', got['pqdesc']
        assert got['pqdesc']['pq_evidence'] == 'manifest:mceliece', got['pqdesc']
        assert got['pqdep']['p_reason'] == '-' and got['pqdep']['pq_evidence'] == 'dependency:oqs', got['pqdep']
        assert got['pqdep']['pq_impl_evidence'] == 'dep-use:crates/pqdep/src/lib.rs:1:oqs', got['pqdep']
        assert got['logic']['pq_evidence'] == 'identifier:crates/logic/src/lib.rs:mlkem_sign_f0', got['logic']
        assert got['logic']['pq_impl_evidence'] == 'own-op:crates/logic/src/lib.rs:1:mlkem_sign_f0:mlkem_sign_f0'
        assert got['conc']['pq_impl_evidence'] == 'own-op:crates/conc/src/inner.rs:3:kyber768_keygen:kyber768_keygen'
        assert got['used']['p_reason'] == 'no-pq-evidence', got['used']
        assert got['orphan']['p_reason'] == 'not-a-workspace-member', got['orphan']
        assert got['excl']['p_reason'] == '-' and got['auto']['p_reason'] == '-', (got['excl'], got['auto'])
        assert got['inner']['c_reason'] == 'not-a-workspace-member'
        assert scan_repository(root / 'orphan')[0]['p_reason'] == '-'
        empty = root / 'empty'
        empty.mkdir()
        assert scan_repository(empty)[0]['c_reason'] == 'no-Cargo-package'
        assert got['conc']['c_reason'] == '-' and got['conc']['std_evidence'].startswith('crates/conc/src/inner.rs:std::sync::Mutex')
        assert got['conc']['test_attrs'] == 1
        assert got['logic']['p_reason'] == '-' and got['logic']['c_reason'] == 'no-std-concurrency-use', got['logic']
        assert got['logic']['nontest_lines'] == 250, got['logic']
        assert got['asyncy']['p_reason'] == 'forbidden-normal-dependency:tokio' and got['asyncy']['has_bin'] == 'yes'
        assert got['small']['p_reason'].startswith('nontest-lines-outside-200-5000:'), got['small']
        assert got['uns']['p_reason'].startswith('unsafe-block-or-fn:'), got['uns']
        assert got['notest']['p_reason'] == 'no-own-#[test]', got['notest']
        assert got['outer']['p_reason'] == '-', got['outer']
        assert got['-']['c_reason'].startswith('unscannable-manifest:'), got['-']
    print('self-test: passed')


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--self-test', action='store_true')
    ap.add_argument('--clones', type=pathlib.Path)
    ap.add_argument('--out', type=pathlib.Path)
    ap.add_argument('--check-narrowing', type=pathlib.Path, metavar='OLD_classes.tsv')
    a = ap.parse_args()
    if a.self_test:
        self_test()
    elif a.check_narrowing:
        if not a.out:
            ap.error('--out is required')
        check_narrowing(a.check_narrowing, a.out.resolve())
    else:
        if not (a.clones and a.out):
            ap.error('--clones and --out are required')
        run(a.clones.resolve(), a.out.resolve())
