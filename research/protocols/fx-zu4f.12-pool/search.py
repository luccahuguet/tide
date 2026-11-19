#!/usr/bin/env python3
"""fx-zu4f.12 step 1: GitHub repository search and repository confirmation.

Read-only GETs through `gh api` (the owner's authenticated CLI). Never commit
search. Request starts are at least 2 s apart; a 403 or 429 waits 300 s before
one retry, then up to two more (300 s, 600 s); a still-failing request stops
the run with BLOCKED evidence. Raw responses stay under TARGET/registration;
their URL, times, status and SHA-256 go to the committed manifests.

  python3 -I search.py search  --target TARGET --out WORK
  python3 -I search.py confirm --target TARGET --out WORK
  python3 -I search.py pin     --target TARGET --out WORK

`pin` makes one read-only clone per kept repository under TARGET/clones
(`--filter=blob:none --no-checkout --single-branch`, default branch), checks
out only `*.rs` and `Cargo.toml` (non-cone sparse checkout, no submodules,
LFS smudge skipped), and records the default-branch HEAD. It builds and runs
nothing.
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
import urllib.parse

TOPICS = ['blockchain', 'ethereum', 'cryptocurrency', 'web3', 'solana', 'bitcoin',
          'zk', 'zero-knowledge', 'evm', 'defi', 'smart-contracts', 'cryptography']
BASE = 'language:Rust stars:>=1000 archived:false fork:false topic:{t}'
OWNER_AFFILIATED = {'yazelix', 'luccahuguet'}  # compared case-insensitively
SPACING = 2.0
_last = [0.0]


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def get(path, store):
    """One throttled GET with backoff; returns (record, parsed JSON)."""
    waits = [300, 300, 600]
    attempt = 0
    while True:
        attempt += 1
        delay = _last[0] + SPACING - time.monotonic()
        if delay > 0:
            time.sleep(delay)
        _last[0] = time.monotonic()
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


def search(target, out):
    rdir = target / 'registration' / 'search'
    manifest = (out / 'queries.jsonl').open('w')
    hits = {}  # full_name -> list of (topic, slice, page)
    slices = []
    seq = [0]

    def run(topic, lo, hi):
        stars = f'stars:>={lo}' if hi is None else f'stars:{lo}..{hi}'
        q = BASE.format(t=topic).replace('stars:>=1000', stars)
        page = 1
        total = None
        while True:
            seq[0] += 1
            path = 'search/repositories?' + urllib.parse.urlencode(
                dict(q=q, sort='stars', order='desc', per_page=100, page=page))
            rec, data = get(path, rdir / f'{seq[0]:04d}.response')
            if data is None:
                raise SystemExit(f'BLOCKED: 404 on search {rec}')
            if data.get('incomplete_results'):
                rec['note'] = 'incomplete_results=true; retried once'
                manifest.write(json.dumps(dict(rec, topic=topic, query=q, page=page,
                                               total_count=data['total_count'],
                                               incomplete_results=True), sort_keys=True) + '\n')
                time.sleep(30)
                seq[0] += 1
                rec, data = get(path, rdir / f'{seq[0]:04d}.response')
                if data.get('incomplete_results'):
                    raise SystemExit(f'BLOCKED: incomplete_results twice: {rec}')
            total = data['total_count']
            if page == 1 and total > 1000:
                # slice by stars: split this range and discard the capped page
                manifest.write(json.dumps(dict(rec, topic=topic, query=q, page=page, total_count=total,
                                               incomplete_results=False, capped=True), sort_keys=True) + '\n')
                slices.append(dict(topic=topic, query=q, total_count=total, capped=True))
                top = hi if hi is not None else max(lo * 4, 64000)
                mid = (lo + top) // 2
                run(topic, lo, mid)
                run(topic, mid + 1, hi)
                return
            manifest.write(json.dumps(dict(rec, topic=topic, query=q, page=page, total_count=total,
                                           incomplete_results=False, capped=False,
                                           items=len(data['items'])), sort_keys=True) + '\n')
            for item in data['items']:
                hits.setdefault(item['full_name'], []).append(f'{topic}|{stars}|p{page}')
            if page == 1:
                slices.append(dict(topic=topic, query=q, total_count=total, capped=False))
            if page * 100 >= total or not data['items']:
                break
            page += 1

    for topic in TOPICS:
        run(topic, 1000, None)
    manifest.close()
    (out / 'star-slices.json').write_text(json.dumps(slices, indent=2) + '\n')
    with (out / 'search-hits.tsv').open('w', newline='') as f:
        w = csv.writer(f, delimiter='\t', lineterminator='\n')
        w.writerow(['repository', 'hits'])
        for name in sorted(hits, key=str.lower):
            w.writerow([name, ';'.join(hits[name])])
    print(f'{len(hits)} distinct repositories from {seq[0]} requests')


def confirm(target, out):
    rdir = target / 'registration' / 'repos'
    with (out / 'search-hits.tsv').open(newline='') as f:
        names = [r['repository'] for r in csv.DictReader(f, delimiter='\t')]
    fields = ['requested_repository', 'repository', 'repository_id', 'stars', 'language', 'default_branch',
              'archived', 'fork', 'visibility', 'observed_utc', 'outcome', 'drop_reason',
              'response_url', 'response_sha256']
    rows, seen = [], {}
    for i, name in enumerate(names, 1):
        row = dict.fromkeys(fields, '-')
        row['requested_repository'] = name
        if name.split('/')[0].lower() in OWNER_AFFILIATED:
            row.update(outcome='dropped', drop_reason='owner-affiliated')
            rows.append(row)
            continue
        rec, data = get('repos/' + name, rdir / f'{i:04d}.response')
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


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument('step', choices=['search', 'confirm', 'pin'])
    p.add_argument('--target', type=pathlib.Path, required=True)
    p.add_argument('--out', type=pathlib.Path, required=True)
    a = p.parse_args()
    dict(search=search, confirm=confirm, pin=pin)[a.step](a.target.resolve(), a.out.resolve())
