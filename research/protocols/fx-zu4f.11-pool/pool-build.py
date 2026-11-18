#!/usr/bin/env python3
"""fx-zu4f.11 repository search, metadata, sequential commit-only counts and scan.

Python >=3.11, standard library, git and gh. Never builds/runs upstream code.
Resumes from immutable response caches and per-repository checkpoints.
"""
import argparse
import datetime as dt
import email.utils
import hashlib
import importlib.util
import json
import math
import os
import pathlib
import posixpath
import shutil
import subprocess
import time
import urllib.parse
from scan import rows, table, inspect, dependencies, source_texts, ManifestError

WORK = pathlib.Path(__file__).resolve().parent
TARGET = pathlib.Path('/mnt/build/tide-agents/fx-zu4f.11-target')
SPEC = importlib.util.spec_from_file_location('marker_count',WORK/'count-activity-markers.py')
COUNTER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(COUNTER)
THRESHOLDS = (50,40,30,25,20)


def utc():
    return dt.datetime.now(dt.UTC).isoformat()


def sha(body):
    return hashlib.sha256(body).hexdigest()


def save(path,value):
    temporary = path.with_suffix(path.suffix+'.tmp')
    temporary.write_text(json.dumps(value,indent=2)+'\n')
    temporary.replace(path)


def command(args,timeout=300):
    return subprocess.run(args,stdin=subprocess.DEVNULL,capture_output=True,timeout=timeout)


def git(repo,*args,timeout=300):
    result = command(['git','-c','core.hooksPath=/dev/null','-C',str(repo),*args],timeout)
    if result.returncode:
        raise RuntimeError(result.stderr.decode(errors='replace'))
    return result.stdout


class API:
    def __init__(self):
        self.raw = TARGET/'registration'/'repository-api'
        self.raw.mkdir(parents=True,exist_ok=True)
        self.manifest = WORK/'queries.jsonl'
        self.previous = [json.loads(x) for x in self.manifest.read_text().splitlines()] if self.manifest.exists() else []
        self.last = 0

    def get(self,endpoint,params=None):
        url = 'https://api.github.com/'+endpoint
        if params:
            url += '?'+urllib.parse.urlencode(params)
        cache = self.raw/(sha(url.encode())+'.body.json')
        if cache.exists():
            return json.loads(cache.read_bytes())
        prior = [r for r in self.previous if r['url']==url]
        for attempt in range(3):
            delay = 2-(time.monotonic()-self.last)
            if attempt == 0 and prior and prior[-1]['http_status'] in (403,429):
                delay = max(delay,300-(time.time()-dt.datetime.fromisoformat(prior[-1]['ended_utc']).timestamp()))
            time.sleep(max(0,delay))
            self.last = time.monotonic()
            start = utc()
            try:
                result = command(['gh','api','--include','--method','GET',url],150)
            except subprocess.TimeoutExpired as error:
                result = subprocess.CompletedProcess([],124,stdout=error.stdout or b'',stderr=(error.stderr or b'')+b'\nAPI command timed out after 150 seconds')
            header,separator,body = result.stdout.partition(b'\r\n\r\n')
            if not separator:
                header,separator,body = result.stdout.partition(b'\n\n')
            headers = {}
            for line in header.decode(errors='replace').splitlines()[1:]:
                if ':' in line:
                    k,v = line.split(':',1);headers[k.lower()] = v.strip()
            try:
                status = int(header.splitlines()[0].split()[1])
                value = json.loads(body)
            except (ValueError,IndexError):
                status,value = 0,{}
            index = len(self.previous)+1
            response = self.raw/f'{index:06d}.response'
            response.write_bytes(result.stdout)
            (self.raw/f'{index:06d}.stderr').write_bytes(result.stderr)
            record = dict(url=url,started_utc=start,ended_utc=utc(),attempt=attempt+1,http_status=status,exit=result.returncode,response_file=str(response),sha256=sha(result.stdout),body_sha256=sha(body),bytes=len(result.stdout),total_count=value.get('total_count'),incomplete_results=value.get('incomplete_results'),stderr=result.stderr.decode(errors='replace'))
            self.previous.append(record)
            with self.manifest.open('a') as f:
                f.write(json.dumps(record,sort_keys=True)+'\n')
            if result.returncode == 0 and status == 200 and (not value.get('incomplete_results') or value.get('total_count',0)>=1000):
                cache.write_bytes(body)
                return value
            if attempt == 2:
                (WORK/'BLOCKED').write_text('GitHub repository acquisition refused/incomplete after polite retries.\n'+json.dumps(record,indent=2)+'\n')
                raise RuntimeError((WORK/'BLOCKED').read_text())
            wait = 300*(attempt+1) if status in (403,429) else 30*(attempt+1)
            if headers.get('x-ratelimit-remaining') == '0':
                wait = max(wait,int(headers.get('x-ratelimit-reset','0'))-time.time()+2)
            try:
                wait = max(wait,float(headers.get('retry-after','0')))
            except ValueError:
                try:
                    wait = max(wait,email.utils.parsedate_to_datetime(headers['retry-after']).timestamp()-time.time())
                except (ValueError,TypeError,OverflowError):
                    pass
            print(json.dumps(dict(event='api-retry',status=status,wait_seconds=wait,url=url)),flush=True)
            time.sleep(max(0,wait))


def discover(api,snapshot):
    path = WORK/'search-repositories.json'
    if path.exists():
        return json.loads(path.read_text())
    found,slices = {},[]
    def search(lo,hi):
        star = f'{lo}..{hi}' if hi is not None else f'>={lo}'
        q = f'language:Rust stars:{star} pushed:>={snapshot["window_start_utc"]} is:public'
        params = dict(q=q,sort='stars',order='desc',per_page=100,page=1)
        first = api.get('search/repositories',params)
        total = first['total_count']
        slices.append(dict(query=q,total_count=total,incomplete_results=first['incomplete_results'],capped=total>=1000))
        save(WORK/'star-slices.json',slices)
        if total >= 1000:
            if hi == lo:
                (WORK/'BLOCKED').write_text(f'Indivisible star slice still capped: {q}: {total}\n')
                raise RuntimeError((WORK/'BLOCKED').read_text())
            middle = (lo+hi)//2 if hi is not None else lo*2-1
            search(lo,middle)
            search(middle+1,hi)
            return
        for page in range(1,math.ceil(total/100)+1):
            params['page'] = page
            value = first if page==1 else api.get('search/repositories',params)
            for item in value['items']:
                found[item['id']] = item
        print(json.dumps(dict(event='repository-slice',stars=star,total=total,distinct=len(found))),flush=True)
    search(1000,None)
    value = sorted(found.values(),key=lambda r:(-r['stargazers_count'],r['full_name']))
    save(path,value)
    return value


def materialize(repo,commit,manifests_only=False):
    # Read Git blobs directly: no checkout hooks, smudge filters or repository code.
    remotes = git(repo,'remote').decode().splitlines()
    if 'origin' in remotes:
        git(repo,'config','remote.origin.partialclonefilter','blob:none')
    tree = git(repo,'ls-tree','-r','-z',commit)
    inventory = {}
    for entry in tree.split(b'\0'):
        if not entry:
            continue
        metadata,name = entry.split(b'\t',1)
        mode,kind,oid = metadata.split()
        relative = pathlib.PurePosixPath(os.fsdecode(name))
        if relative.is_absolute() or '..' in relative.parts:
            raise ValueError('escaping Git tree path')
        inventory[relative] = (mode,kind,oid)
    selected, links = {}, {}
    def include(relative, seen):
        if relative in seen:
            raise ValueError('cyclic scanner source symlink')
        mode,kind,oid = inventory[relative]
        if mode == b'120000':
            target = git(repo,'cat-file','blob',oid.decode()).decode('utf-8')
            resolved = pathlib.PurePosixPath(posixpath.normpath(posixpath.join(str(relative.parent),target)))
            if resolved.is_absolute() or '..' in resolved.parts or resolved not in inventory:
                raise ValueError('escaping or unavailable scanner source symlink')
            include(resolved,seen | {relative})
            links[relative] = target
        elif mode in (b'100644',b'100755') and kind == b'blob':
            selected[relative] = oid
        else:
            raise ValueError('non-regular scanner source target')
    for relative in inventory:
        if relative.name == 'Cargo.toml' or (not manifests_only and relative.suffix == '.rs'):
            include(relative,set())
    selected = sorted(selected.items())
    requests = b''.join(oid+b'\n' for _,oid in selected)
    if requests and 'origin' in remotes:
        fetched = subprocess.run(['git','-c','core.hooksPath=/dev/null','-c','fetch.negotiationAlgorithm=noop','-C',str(repo),'fetch','--no-tags','--no-write-fetch-head','--no-auto-maintenance','--recurse-submodules=no','--filter=blob:none','--refmap=','--stdin','origin'],input=requests,capture_output=True,timeout=900)
        (TARGET/'registration'/'measurements'/(repo.name+'-source-fetch.log')).write_bytes(fetched.stdout+fetched.stderr)
        if fetched.returncode:
            raise RuntimeError(fetched.stderr.decode(errors='replace'))
    result = subprocess.run(['git','-c','core.hooksPath=/dev/null','-C',str(repo),'cat-file','--batch'],input=requests,capture_output=True,timeout=900)
    if result.returncode:
        raise RuntimeError(result.stderr.decode(errors='replace'))
    offset = 0
    for relative,oid in selected:
        end = result.stdout.index(b'\n',offset)
        header = result.stdout[offset:end].split()
        assert header[:2] == [oid,b'blob'],header
        size = int(header[2]);start=end+1
        body = result.stdout[start:start+size]
        assert result.stdout[start+size:start+size+1]==b'\n'
        dest = repo/pathlib.Path(relative)
        dest.parent.mkdir(parents=True,exist_ok=True)
        dest.write_bytes(body)
        offset = start+size+1
    assert offset==len(result.stdout)
    for relative,target in links.items():
        dest = repo/pathlib.Path(relative)
        dest.parent.mkdir(parents=True,exist_ok=True)
        if dest.is_symlink():
            dest.unlink()
        dest.symlink_to(target)
        assert dest.resolve().is_relative_to(repo.resolve())
    return dict(git_tree_sha256=sha(tree),materialized_files=len(selected)+len(links))


def choose_threshold(scan_counts):
    return next((t for t in THRESHOLDS if scan_counts[t]>=10),20)


def source_directories(repo):
    sources, reached = source_texts(repo.resolve())
    directories = {pathlib.PurePosixPath(path).parent for path,text in sources if path in reached and text.strip()}
    roots = []
    for directory in sorted(directories,key=lambda p:(len(p.parts),p.as_posix())):
        if not any(directory.is_relative_to(parent) for parent in roots):
            roots.append(directory)
    return sorted(p.as_posix() for p in roots)


def measure(api,snapshot,candidates):
    directory = TARGET/'repositories';directory.mkdir(exist_ok=True)
    checkpoints = TARGET/'registration'/'measurements';checkpoints.mkdir(exist_ok=True)
    output,activity,scanned = [],[],[]
    scanner_sha256 = sha((WORK/'scan.py').read_bytes())
    for index,candidate in enumerate(candidates,1):
        name = candidate['full_name']
        checkpoint = checkpoints/(str(candidate['id'])+'.json')
        if checkpoint.exists():
            state = json.loads(checkpoint.read_text())
        else:
            observed = utc()
            metadata = api.get('repos/'+name)
            canonical = metadata['full_name']
            reason = next((label for failed,label in [
                (metadata.get('language')!='Rust','not-Rust'),
                (metadata.get('stargazers_count',0)<1000,'below-1000-stars'),
                (metadata.get('archived'),'archived'),(metadata.get('fork'),'fork'),
                (metadata.get('private') or metadata.get('visibility','public')!='public','not-public')]
                if failed),'')
            record = dict(requested_repository=name,repository=canonical,repository_id=metadata['id'],stars=metadata['stargazers_count'],language=metadata.get('language'),default_branch=metadata['default_branch'],archived=metadata['archived'],fork=metadata['fork'],private=metadata['private'],observed_utc=observed,outcome='excluded' if reason else 'kept',first_reason=reason or '-')
            state = dict(candidate=record)
            if not reason:
                repo = directory/str(metadata['id'])
                started = utc()
                if repo.exists():
                    # Only an unfinished, task-owned clone is removed on retry.
                    shutil.rmtree(repo)
                args = ['git','-c','core.hooksPath=/dev/null','clone','--filter=tree:0','--no-checkout','--single-branch','--shallow-since='+snapshot['window_start_utc'],'https://github.com/'+canonical+'.git',str(repo)]
                log = checkpoints/(str(candidate['id'])+'-clone.log')
                for attempt in range(3):
                    try:
                        clone = command(args,600)
                    except subprocess.TimeoutExpired:
                        clone = subprocess.CompletedProcess(args,124,stdout=b'',stderr=b'clone timeout after 600 seconds')
                    with log.open('ab') as f:
                        f.write((utc()+' attempt='+str(attempt+1)+' command='+json.dumps(args)+'\n').encode()+clone.stdout+clone.stderr+b'\n')
                    if not clone.returncode:
                        break
                    if repo.exists():
                        shutil.rmtree(repo)
                    if b'no commits selected for shallow requests' in clone.stderr.lower() or b'error processing shallow info' in clone.stderr.lower():
                        args = [arg for arg in args if not arg.startswith('--shallow-since=')]
                        print(json.dumps(dict(event='clone-full-commits-fallback',repository=canonical)),flush=True)
                        continue
                    if attempt == 2:
                        (WORK/'BLOCKED').write_text(f'Commit-only clone failed after retries: {canonical}; exit={clone.returncode}; log={log}\n')
                        raise RuntimeError((WORK/'BLOCKED').read_text())
                    rate_limited = b'403' in clone.stderr or b'429' in clone.stderr
                    delay = 300*(attempt+1) if rate_limited else 30*(attempt+1)
                    print(json.dumps(dict(event='clone-retry',repository=canonical,wait_seconds=delay)),flush=True)
                    time.sleep(delay)
                else:
                    (WORK/'BLOCKED').write_text(f'Commit-only clone failed: {canonical}; log={log}\n')
                    raise RuntimeError((WORK/'BLOCKED').read_text())
                pin = git(repo,'rev-parse','HEAD').decode().strip()
                refs = git(repo,'for-each-ref','--format=%(refname) %(objectname)').decode()
                counts = COUNTER.counts(repo,pin,snapshot['window_start_utc'],snapshot['snapshot_utc'])
                count = dict(repository=canonical,repository_id=metadata['id'],stars=record['stars'],default_branch=record['default_branch'],commit=pin,clone_started_utc=started,clone_ended_utc=utc(),clone_command=json.dumps(args),refs_sha256=sha(refs.encode()),**counts)
                count['command'] = json.dumps(count['command'])
                count['activity_outcome'] = 'kept' if count['commits']>=50 else 'excluded'
                for threshold in THRESHOLDS:
                    count['T'+str(threshold)] = 'kept' if count['commits']>=50 and count['marked']*100>=threshold*count['commits'] else 'excluded'
                state.update(activity=count,refs=refs)
                if count['T20']=='kept':
                    remote_refs = git(repo,'ls-remote','--refs','origin','refs/heads/*','refs/tags/*').decode()
                    state['remote_refs'] = remote_refs
                    acquisition = materialize(repo,pin,manifests_only=True)
                    try:
                        needs_source = not dependencies(repo.resolve())
                    except ManifestError:
                        needs_source = False
                    if needs_source:
                        acquisition = materialize(repo,pin)
                    exclusion,evidence,files = inspect(repo)
                    state['scanner_sha256'] = scanner_sha256
                    state['scan'] = dict(repository=canonical,repository_id=metadata['id'],stars=record['stars'],commit=pin,outcome='excluded' if exclusion else 'kept',first_reason=exclusion,source_files=files,std_evidence=evidence or '-',**acquisition)
                else:
                    shutil.rmtree(repo)
            save(checkpoint,state)
        if 'scan' in state and state.get('scanner_sha256') != scanner_sha256:
            repo = directory/str(state['candidate']['repository_id'])
            assert git(repo,'rev-parse','HEAD').decode().strip() == state['activity']['commit']
            exclusion,evidence,files = inspect(repo)
            state['scan'].update(outcome='excluded' if exclusion else 'kept',first_reason=exclusion,source_files=files,std_evidence=evidence or '-')
            state['scanner_sha256'] = scanner_sha256
            save(checkpoint,state)
        metadata_url = 'https://api.github.com/repos/'+state['candidate']['requested_repository']
        observation = next(r for r in reversed(api.previous) if r['url']==metadata_url and r['http_status']==200 and r['exit']==0)
        provenance = dict(observed_utc=observation['started_utc'],response_url=metadata_url,response_sha256=observation['sha256'])
        if any(state['candidate'].get(key)!=value for key,value in provenance.items()):
            state['candidate'].update(provenance)
            save(checkpoint,state)
        state['candidate']['first_reason'] = state['candidate']['first_reason'] or '-'
        output.append(state['candidate'])
        if 'activity' in state:
            activity.append(state['activity'])
        if 'scan' in state:
            scanned.append(state['scan'])
        table(WORK/'candidates.tsv',list(output[0]),output)
        if activity:
            table(WORK/'activity-markers.tsv',list(activity[0]),activity)
        if scanned:
            table(WORK/'scan-output.tsv',list(scanned[0]),scanned)
        print(json.dumps(dict(event='measured',index=index,total=len(candidates),metadata_survivors=len(activity),activity_survivors=sum(r['commits']>=50 for r in activity),marker20=sum(r['T20']=='kept' for r in activity),scan_survivors=sum(r['outcome']=='kept' for r in scanned))),flush=True)
    ids = [r['repository_id'] for r in output]
    assert len(ids)==len(set(ids)),'canonical duplicate; stop rather than double-count'
    fields = ['repository','repository_id','stars','commit','outcome','first_reason','source_files','std_evidence','git_tree_sha256','materialized_files']
    if not scanned:
        table(WORK/'scan-output.tsv',fields,[])
    marker_counts = {t:sum(r['T'+str(t)]=='kept' for r in activity) for t in THRESHOLDS}
    kept_ids = {r['repository_id'] for r in scanned if r['outcome']=='kept'}
    scan_counts = {t:sum(r['T'+str(t)]=='kept' and r['repository_id'] in kept_ids for r in activity) for t in THRESHOLDS}
    threshold = choose_threshold(scan_counts)
    selected_ids = {r['repository_id'] for r in activity if r['T'+str(threshold)]=='kept'} & kept_ids
    pool = sorted((r for r in scanned if r['repository_id'] in selected_ids),key=lambda r:(-r['stars'],r['repository']))[:30]
    table(WORK/'pool.tsv',['pool_rank']+fields,[dict(pool_rank=i,**r) for i,r in enumerate(pool,1)])
    source_dirs = [dict(pool_rank=rank,repository=row['repository'],commit=row['commit'],source_directory=directory)
                   for rank,row in enumerate(pool,1)
                   for directory in source_directories(TARGET/'repositories'/str(row['repository_id']))]
    table(WORK/'b-source-dirs.tsv',['pool_rank','repository','commit','source_directory'],source_dirs)
    summary = dict(distinct_repositories=len(candidates),after_repository_filter=len(activity),after_activity=sum(r['commits']>=50 for r in activity),marker_counts=marker_counts,scan_counts=scan_counts,chosen_T=threshold,after_markers=marker_counts[threshold],after_scan=scan_counts[threshold],pool_size=len(pool),completed_utc=utc())
    save(WORK/'selection.json',summary)
    print(json.dumps(dict(event='complete',**summary)),flush=True)


def self_test():
    import tempfile
    from unittest.mock import patch, Mock
    assert choose_threshold({50:10,40:11,30:12,25:13,20:14}) == 50
    assert choose_threshold({50:9,40:9,30:10,25:11,20:12}) == 30
    assert choose_threshold({50:0,40:0,30:0,25:0,20:9}) == 20
    with tempfile.TemporaryDirectory() as temporary:
        temporary = pathlib.Path(temporary)
        work,target = temporary/'work',temporary/'target'
        work.mkdir();target.mkdir()
        class FixtureAPI:
            def get(self,endpoint,params):
                query = params['q']
                if 'stars:>=1000' in query:
                    return dict(total_count=1000,incomplete_results=False,items=[])
                if 'stars:1000..1999' in query:
                    items = [dict(id=1,stargazers_count=1200,full_name='fixture/a'),dict(id=2,stargazers_count=1500,full_name='fixture/b')]
                else:
                    items = [dict(id=3,stargazers_count=2000,full_name='fixture/c')]
                return dict(total_count=len(items),incomplete_results=False,items=items)
        with patch.dict(globals(),WORK=work,TARGET=target):
            found = discover(FixtureAPI(),{'window_start_utc':'2025-01-01T00:00:00Z'})
        assert [r['id'] for r in found] == [3,2,1]
        success = subprocess.CompletedProcess([],0,stdout=b'HTTP/2.0 200 OK\r\n\r\n{"id":1}',stderr=b'')
        retry_at = email.utils.formatdate(time.time()+900,usegmt=True)
        refusal = subprocess.CompletedProcess([],1,stdout=('HTTP/2.0 429 Too Many Requests\r\nRetry-After: '+retry_at+'\r\n\r\n{}').encode(),stderr=b'HTTP 429')
        invalid_retry = subprocess.CompletedProcess([],1,stdout=b'HTTP/2.0 429 Too Many Requests\r\nRetry-After: invalid\r\n\r\n{}',stderr=b'HTTP 429')
        with patch.dict(globals(),WORK=work,TARGET=target):
            api = API()
            with patch.dict(globals(),command=lambda *a,**k:success), patch.object(time,'sleep'):
                assert api.get('repos/fixture/success') == {'id':1}
                assert api.get('repos/fixture/success') == {'id':1}
            assert len(api.previous)==1
            refusals = iter([invalid_retry,refusal,refusal])
            with patch.dict(globals(),command=lambda *a,**k:next(refusals)), patch.object(time,'sleep') as sleeping:
                try:
                    api.get('repos/fixture/refused')
                except RuntimeError:
                    pass
                else:
                    raise AssertionError('persistent rate limit did not stop acquisition')
                assert len(api.previous)==4 and (work/'BLOCKED').exists()
                assert 300 in [call.args[0] for call in sleeping.call_args_list]
                assert sum(call.args[0]>800 for call in sleeping.call_args_list)==1
            with patch.dict(globals(),command=Mock(side_effect=subprocess.TimeoutExpired(['gh'],150))), patch.object(time,'sleep') as sleeping:
                try:
                    api.get('repos/fixture/timeout')
                except RuntimeError:
                    pass
                else:
                    raise AssertionError('persistent API timeout did not stop acquisition')
                assert [r['exit'] for r in api.previous[-3:]]==[124,124,124]
                assert {30,60}.issubset({call.args[0] for call in sleeping.call_args_list})
        repo = temporary/'repo';repo.mkdir()
        git(repo,'init','--quiet')
        git(repo,'config','user.name','Fixture')
        git(repo,'config','user.email','fixture@example.invalid')
        (repo/'Cargo.toml').write_text('[package]\nname="fixture"\nversion="0.1.0"\n')
        (repo/'src').mkdir();(repo/'src/lib.rs').write_text('use std::sync::Mutex;')
        (repo/'README.md').write_text('must not materialize')
        git(repo,'add','.');git(repo,'commit','--quiet','-m','fixture')
        pin = git(repo,'rev-parse','HEAD').decode().strip()
        (repo/'Cargo.toml').unlink();(repo/'src/lib.rs').unlink();(repo/'README.md').unlink()
        acquisition = materialize(repo,pin)
        assert acquisition['materialized_files']==2 and not (repo/'README.md').exists()
        assert inspect(repo)[0]=='',inspect(repo)
        git(repo,'config','uploadpack.allowFilter','true')
        git(repo,'config','uploadpack.allowAnySHA1InWant','true')
        copy = temporary/'partial'
        cloned = command(['git','clone','--filter=tree:0','--no-checkout',repo.as_uri(),str(copy)])
        assert cloned.returncode==0,cloned.stderr
        (target/'registration/measurements').mkdir(parents=True)
        with patch.dict(globals(),WORK=work,TARGET=target):
            materialize(copy,pin,manifests_only=True)
            assert (copy/'Cargo.toml').exists() and not (copy/'src/lib.rs').exists()
            acquisition = materialize(copy,pin)
        assert acquisition['materialized_files']==2 and inspect(copy)[0]==''
        assert not (copy/'README.md').exists()
        assert source_directories(copy)==['src']
        (repo/'Cargo.toml').rename(repo/'manifest.in')
        (repo/'Cargo.toml').symlink_to('manifest.in')
        (repo/'shared.inc').write_text('fn helper() {}')
        (repo/'src/shared.rs').symlink_to('../shared.inc')
        git(repo,'add','-A');git(repo,'commit','--quiet','-m','internal symlinks')
        pin = git(repo,'rev-parse','HEAD').decode().strip()
        linked = temporary/'linked'
        assert command(['git','clone','--filter=tree:0','--no-checkout',repo.as_uri(),str(linked)]).returncode==0
        with patch.dict(globals(),WORK=work,TARGET=target):
            materialize(linked,pin)
        assert (linked/'Cargo.toml').is_symlink() and (linked/'src/shared.rs').is_symlink()
        assert inspect(linked)[0]=='' and not (linked/'README.md').exists()
        (repo/'src/escape.rs').symlink_to('../../../outside.rs')
        git(repo,'add','-A');git(repo,'commit','--quiet','-m','escaping fixture link')
        unsafe = temporary/'unsafe'
        assert command(['git','clone','--filter=tree:0','--no-checkout',repo.as_uri(),str(unsafe)]).returncode==0
        pin = git(repo,'rev-parse','HEAD').decode().strip()
        with patch.dict(globals(),WORK=work,TARGET=target):
            try:
                materialize(unsafe,pin)
            except ValueError as error:
                assert 'escaping or unavailable' in str(error)
            else:
                raise AssertionError('outside source symlink was accepted')
    print('self-test: passed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--discover-only',action='store_true')
    parser.add_argument('--self-test',action='store_true')
    args = parser.parse_args()
    if args.self_test:
        self_test()
        raise SystemExit(0)
    assert TARGET.resolve().is_relative_to(pathlib.Path('/mnt/build'))
    api = API();snapshot=json.loads((WORK/'snapshot.json').read_text())
    candidates = discover(api,snapshot)
    if not args.discover_only:
        measure(api,snapshot,candidates)
