#!/usr/bin/env python3
"""fx-zu4f.14 step 1: repository and crates.io search, repository confirmation, pinning.

fx-zu4f.12's search.py, adapted to the post-quantum terms and extended with
crates.io search. Read-only GETs only. GitHub goes through `gh api` (the
owner's authenticated CLI), REPOSITORY search only, never commit search.
crates.io goes through urllib with a generic User-Agent. Request starts are at
least 2 s apart per host (2.5 s for GitHub search); a 403 or 429 waits 300 s, then 300 s, then 600 s,
and a still-failing request stops the run with BLOCKED evidence. Raw responses
stay under TARGET/registration; their URL, times, status and SHA-256 go to the
committed manifests.

  python3 -I search.py search  --target TARGET --out WORK   # GitHub + crates.io
  python3 -I search.py confirm --target TARGET --out WORK   # repos/<name> GETs
  python3 -I search.py pin     --target TARGET --out WORK   # read-only clones

`pin` makes one read-only clone per kept repository under TARGET/clones
(`--filter=blob:none --no-checkout --single-branch`, default branch), checks
out only `*.rs` and `Cargo.toml` (non-cone sparse checkout, no submodules,
hooks disabled, LFS smudge skipped), and records the default-branch HEAD. It
builds and runs nothing.
"""
import argparse
import csv
import datetime
import hashlib
import json
import pathlib
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request

# The bead's algorithms (ML-KEM/Kyber, ML-DSA/Dilithium, SLH-DSA/SPHINCS+,
# FN-DSA/Falcon, HQC, Classic McEliece) and protocols (rosenpass,
# post-quantum TLS or Noise), each spelled the ways a name, description,
# topic, README or crate keyword writes it. Search terms only find
# candidates; classify.py's PQ-evidence gate decides what implements one.
TERMS = ['ML-KEM', 'MLKEM', 'Kyber', 'ML-DSA', 'MLDSA', 'Dilithium', 'SLH-DSA', 'SLHDSA', 'SPHINCS',
         'FN-DSA', 'FNDSA', 'Falcon', 'HQC', 'McEliece', 'post-quantum', 'postquantum', 'PQC',
         'rosenpass', 'quantum-resistant', 'quantum-safe']
GH_FORMS = ['{t} language:Rust stars:>=1000 archived:false fork:false',
            '{t} in:readme language:Rust stars:>=1000 archived:false fork:false']
OWNER_AFFILIATED = {'yazelix', 'luccahuguet'}  # compared case-insensitively
SPACING = {'github-search': 2.5, 'github-core': 2.0, 'crates.io': 2.0}  # search allows 30 a minute
USER_AGENT = 'tide-research-search/0.1 (read-only registration search)'
_last = {}


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def throttle(host):
    delay = _last.get(host, 0.0) + SPACING[host] - time.monotonic()
    if delay > 0:
        time.sleep(delay)
    _last[host] = time.monotonic()


def gh_get(path, store):
    """One throttled GitHub GET with backoff; returns (record, parsed JSON or None on 404)."""
    waits = [300, 300, 600]
    attempt = 0
    while True:
        attempt += 1
        throttle('github-search' if path.startswith('search/') else 'github-core')
        started = now()
        proc = subprocess.run(['gh', 'api', '--include', '-H', 'Accept: application/vnd.github+json', path],
                              stdin=subprocess.DEVNULL, capture_output=True, timeout=120)
        ended = now()
        raw = proc.stdout
        head, body = (re.split(rb'\r?\n\r?\n', raw, maxsplit=1) + [b''])[:2]
        status_line = head.split(b'\n', 1)[0].decode('latin-1').strip()
        status = int(status_line.split()[1]) if status_line.startswith('HTTP/') else None
        store.write_bytes(raw)
        record = dict(url='https://api.github.com/' + path, started_utc=started, ended_utc=ended,
                      attempt=attempt, exit=proc.returncode, http_status=status,
                      response_file=str(store), bytes=len(body),
                      body_sha256=hashlib.sha256(body).hexdigest(),
                      stderr=proc.stderr.decode('utf-8', 'replace').strip()[:300])
        if status in (403, 429) and attempt <= len(waits):
            print(f'{status} on {path}; backing off {waits[attempt-1]} s', file=sys.stderr, flush=True)
            time.sleep(waits[attempt-1])
            continue
        if status == 404:
            return record, None
        if status != 200:
            raise SystemExit(f'BLOCKED: {json.dumps(record)}')
        return record, json.loads(body)


def crates_get(url, store):
    """One throttled crates.io GET with backoff; returns (record, parsed JSON)."""
    waits = [300, 300, 600]
    attempt = 0
    while True:
        attempt += 1
        throttle('crates.io')
        started = now()
        request = urllib.request.Request(url, headers={'User-Agent': USER_AGENT, 'Accept': 'application/json'})
        try:
            with urllib.request.urlopen(request, timeout=120) as response:
                status, body = response.status, response.read()
        except urllib.error.HTTPError as error:
            status, body = error.code, error.read()
        ended = now()
        store.write_bytes(body)
        record = dict(url=url, started_utc=started, ended_utc=ended, attempt=attempt, http_status=status,
                      response_file=str(store), bytes=len(body), body_sha256=hashlib.sha256(body).hexdigest())
        if status in (403, 429) and attempt <= len(waits):
            print(f'{status} on {url}; backing off {waits[attempt-1]} s', file=sys.stderr, flush=True)
            time.sleep(waits[attempt-1])
            continue
        if status != 200:
            raise SystemExit(f'BLOCKED: {json.dumps(record)}')
        return record, json.loads(body)


def github_name(url):
    """owner/repo for a github.com repository URL, else None."""
    if not url:
        return None
    parsed = urllib.parse.urlparse(url.strip())
    host = parsed.netloc.lower().split('@')[-1].split(':')[0]
    if host not in ('github.com', 'www.github.com'):
        return None
    parts = [p for p in parsed.path.split('/') if p]
    if len(parts) < 2:
        return None
    repo = parts[1][:-4] if parts[1].endswith('.git') else parts[1]
    return f'{parts[0]}/{repo}' if repo else None


def search(target, out):
    gdir = target / 'registration' / 'search'
    cdir = target / 'registration' / 'crates-io'
    gdir.mkdir(parents=True, exist_ok=True)
    cdir.mkdir(parents=True, exist_ok=True)
    manifest = (out / 'queries.jsonl').open('w')
    hits = {}  # GitHub full_name -> list of sources
    slices = []
    seq = [0]

    def run(term, form, lo, hi):
        stars = f'stars:>={lo}' if hi is None else f'stars:{lo}..{hi}'
        q = form.format(t=f'"{term}"' if '-' in term else term).replace('stars:>=1000', stars)
        page = 1
        while True:
            seq[0] += 1
            path = 'search/repositories?' + urllib.parse.urlencode(
                dict(q=q, sort='stars', order='desc', per_page=100, page=page))
            rec, data = gh_get(path, gdir / f'{seq[0]:04d}.response')
            if data is None:
                raise SystemExit(f'BLOCKED: 404 on search {rec}')
            if data.get('incomplete_results'):
                manifest.write(json.dumps(dict(rec, source='github', term=term, query=q, page=page,
                                               total_count=data['total_count'], incomplete_results=True,
                                               note='incomplete_results=true; retried once'), sort_keys=True) + '\n')
                time.sleep(30)
                seq[0] += 1
                rec, data = gh_get(path, gdir / f'{seq[0]:04d}.response')
                if data.get('incomplete_results'):
                    raise SystemExit(f'BLOCKED: incomplete_results twice: {rec}')
            total = data['total_count']
            if page == 1 and total > 1000:
                manifest.write(json.dumps(dict(rec, source='github', term=term, query=q, page=page,
                                               total_count=total, incomplete_results=False, capped=True),
                                          sort_keys=True) + '\n')
                slices.append(dict(term=term, query=q, total_count=total, capped=True))
                top = hi if hi is not None else max(lo * 4, 64000)
                mid = (lo + top) // 2
                run(term, form, lo, mid)
                run(term, form, mid + 1, hi)
                return
            manifest.write(json.dumps(dict(rec, source='github', term=term, query=q, page=page, total_count=total,
                                           incomplete_results=False, capped=False, items=len(data['items'])),
                                      sort_keys=True) + '\n')
            for item in data['items']:
                hits.setdefault(item['full_name'], []).append(f'gh:{q}|p{page}')
            if page == 1:
                slices.append(dict(term=term, query=q, total_count=total, capped=False))
            if page * 100 >= total or not data['items']:
                break
            page += 1

    for form in GH_FORMS:
        for term in TERMS:
            run(term, form, 1000, None)

    crates = {}  # crate name -> dict(repository URL, github name or reason, terms)
    cseq = 0
    for term in TERMS:
        # Alphabetical order, following meta.next_page: crates.io refuses
        # relevance-sorted pages beyond the first 1,000 results.
        page = 1
        url = 'https://crates.io/api/v1/crates?' + urllib.parse.urlencode(dict(q=term, per_page=100, sort='alpha'))
        while url:
            cseq += 1
            rec, data = crates_get(url, cdir / f'{cseq:04d}.json')
            total = data['meta']['total']
            manifest.write(json.dumps(dict(rec, source='crates.io', term=term, query=term, page=page,
                                           total_count=total, items=len(data['crates'])), sort_keys=True) + '\n')
            for c in data['crates']:
                entry = crates.setdefault(c['name'], dict(repository=c.get('repository') or '-', terms=[]))
                entry['terms'].append(f'{term}|p{page}')
            following = data['meta'].get('next_page')
            url = 'https://crates.io/api/v1/crates' + following if following and data['crates'] else None
            page += 1
        seen = sum(f'{term}|' in t for e in crates.values() for t in e['terms'])
        if seen != total:
            raise SystemExit(f'BLOCKED: crates.io {term}: {seen} crates paged, meta.total {total}')
    manifest.close()

    with (out / 'crates-io-hits.tsv').open('w', newline='') as f:
        w = csv.writer(f, delimiter='\t', lineterminator='\n')
        w.writerow(['crate', 'repository_url', 'github_repository', 'outcome', 'terms'])
        for name in sorted(crates):
            e = crates[name]
            gh = github_name(e['repository'] if e['repository'] != '-' else None)
            outcome = ('no-repository-url' if e['repository'] == '-' else
                       'repository-not-on-github' if gh is None else 'github')
            if gh:
                hits.setdefault(gh, []).append(f'crates.io:{name}')
            w.writerow([name, e['repository'], gh or '-', outcome, ';'.join(e['terms'])])
    (out / 'star-slices.json').write_text(json.dumps(slices, indent=2) + '\n')
    with (out / 'search-hits.tsv').open('w', newline='') as f:
        w = csv.writer(f, delimiter='\t', lineterminator='\n')
        w.writerow(['repository', 'hits'])
        for name in sorted(hits, key=str.lower):
            w.writerow([name, ';'.join(hits[name])])
    print(f'{len(hits)} distinct repository names from {seq[0]} GitHub and {cseq} crates.io requests; '
          f'{len(crates)} distinct crates.io crates')


def confirm(target, out):
    rdir = target / 'registration' / 'repos'
    rdir.mkdir(parents=True, exist_ok=True)
    with (out / 'search-hits.tsv').open(newline='') as f:
        names = [r['repository'] for r in csv.DictReader(f, delimiter='\t')]
    fields = ['requested_repository', 'repository', 'repository_id', 'stars', 'language', 'default_branch',
              'archived', 'fork', 'visibility', 'observed_utc', 'outcome', 'drop_reason',
              'response_url', 'response_sha256']
    rows, seen, lower = [], {}, set()
    for i, name in enumerate(names, 1):
        row = dict.fromkeys(fields, '-')
        row['requested_repository'] = name
        if name.split('/')[0].lower() in OWNER_AFFILIATED:
            row.update(outcome='dropped', drop_reason='owner-affiliated')
            rows.append(row)
            continue
        if name.lower() in lower:  # same name in another letter case: one GET only
            row.update(outcome='dropped', drop_reason='duplicate-name-case')
            rows.append(row)
            continue
        lower.add(name.lower())
        rec, data = gh_get('repos/' + urllib.parse.quote(name), rdir / f'{i:04d}.response')
        row.update(observed_utc=rec['started_utc'], response_url=rec['url'], response_sha256=rec['body_sha256'])
        if data is None:
            row.update(outcome='dropped', drop_reason='not-found-404')
            rows.append(row)
            continue
        row.update(repository=data['full_name'], repository_id=data['id'], stars=data['stargazers_count'],
                   language=data.get('language') or '-', default_branch=data['default_branch'],
                   archived=data['archived'], fork=data['fork'], visibility=data.get('visibility', '-'))
        reasons = []
        if data['owner']['login'].lower() in OWNER_AFFILIATED:
            reasons.append('owner-affiliated')
        if data.get('language') != 'Rust':
            reasons.append('language-not-Rust')
        if data['stargazers_count'] < 1000:
            reasons.append('stars-below-1000')
        if data['archived']:
            reasons.append('archived')
        if data['fork']:
            reasons.append('fork')
        if data.get('private') or data.get('visibility') != 'public':
            reasons.append('not-public')
        if data['id'] in seen:
            reasons.append('duplicate-of:' + seen[data['id']])
        seen.setdefault(data['id'], name)
        row.update(outcome='dropped' if reasons else 'kept', drop_reason=reasons[0] if reasons else '-')
        rows.append(row)
    with (out / 'candidates.tsv').open('w', newline='') as f:
        w = csv.DictWriter(f, fields, delimiter='\t', lineterminator='\n')
        w.writeheader()
        w.writerows(rows)
    kept = sum(r['outcome'] == 'kept' for r in rows)
    print(f'{len(rows)} candidates, {kept} kept')


def git(*args, cwd=None):
    env = dict(__import__('os').environ, GIT_LFS_SKIP_SMUDGE='1', GIT_TERMINAL_PROMPT='0')
    proc = subprocess.run(['git', '-c', 'core.hooksPath=/dev/null', *args], cwd=cwd, env=env,
                          stdin=subprocess.DEVNULL, capture_output=True, timeout=3600)
    if proc.returncode:
        raise SystemExit(f'BLOCKED: git {args} in {cwd}: {proc.stderr.decode()[-500:]}')
    return proc.stdout.decode().strip()


def pin(target, out):
    with (out / 'candidates.tsv').open(newline='') as f:
        kept = [r for r in csv.DictReader(f, delimiter='\t') if r['outcome'] == 'kept']
    fields = ['repository', 'clone_url', 'default_branch', 'head_commit', 'clone_started_utc',
              'clone_ended_utc', 'checkout']
    path = out / 'clone-refs.tsv'
    done = {}
    if path.exists():
        with path.open(newline='') as f:
            done = {r['repository']: r for r in csv.DictReader(f, delimiter='\t')}
    rows = []
    for r in kept:
        name = r['repository']
        if name in done:
            rows.append(done[name])
            continue
        dest = target / 'clones' / name.replace('/', '__')
        url = f'https://github.com/{name}.git'
        started = now()
        if dest.exists():
            raise SystemExit(f'BLOCKED: partial clone {dest} exists; inspect it before rerunning')
        git('clone', '--filter=blob:none', '--no-checkout', '--single-branch',
            '--branch', r['default_branch'], url, str(dest))
        git('sparse-checkout', 'set', '--no-cone', '*.rs', 'Cargo.toml', cwd=dest)
        git('checkout', r['default_branch'], cwd=dest)
        head = git('rev-parse', 'HEAD', cwd=dest)
        rows.append(dict(repository=name, clone_url=url, default_branch=r['default_branch'], head_commit=head,
                         clone_started_utc=started, clone_ended_utc=now(),
                         checkout='sparse --no-cone: *.rs Cargo.toml'))
        with path.open('w', newline='') as f:
            w = csv.DictWriter(f, fields, delimiter='\t', lineterminator='\n')
            w.writeheader()
            w.writerows(rows)
        print(name, head, flush=True)
    print(f'{len(rows)} pinned')


def self_test():
    assert github_name('https://github.com/RustCrypto/KEMs') == 'RustCrypto/KEMs'
    assert github_name('https://github.com/a/b.git') == 'a/b'
    assert github_name('git+https://github.com/a/b') == 'a/b'
    assert github_name('github.com/a/b') is None  # no scheme: no host to read
    assert github_name('https://github.com/a/b/tree/main/ml-kem/') == 'a/b'
    assert github_name('https://gitlab.com/a/b') is None and github_name('https://github.com/a') is None
    print('self-test: passed')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('step', choices=['search', 'confirm', 'pin', 'self-test'])
    p.add_argument('--target', type=pathlib.Path)
    p.add_argument('--out', type=pathlib.Path)
    a = p.parse_args()
    if a.step == 'self-test':
        self_test()
    else:
        dict(search=search, confirm=confirm, pin=pin)[a.step](a.target.resolve(), a.out.resolve())
