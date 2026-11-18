#!/usr/bin/env python3
"""Descriptive lexical admission observations, never a bug/catch classifier.

Run on pool repositories only after coordinator 'go'. --self-test uses fixtures.
See protocol.md for the alias/import and discarded-JoinHandle heuristic.
"""
import argparse
import csv
import pathlib
import re
import sys
from scan import balanced_end, non_test, source_texts, use_paths

AMBIENT = ('std::fs','std::net','std::process','std::env')
CLOCK = ('std::time::SystemTime','std::time::Instant')


def category(path):
    for prefix in AMBIENT:
        if path == prefix or path.startswith(prefix+'::'):
            return prefix
    for prefix in CLOCK:
        if path == prefix or path.startswith(prefix+'::'):
            return 'clock'
    return None


def observe(path, text):
    """Emit canonical uses without type resolution, preserving original line offsets."""
    imports = list(re.finditer(r'\buse\s+[^;]+;', text))
    aliases = dict((alias,canonical) for canonical,alias in use_paths(text) if alias != '*')
    found = []
    def emit(offset,kind,canonical,detail):
        found.append(dict(file=path,line=text.count('\n',0,offset)+1,kind=kind,canonical=canonical,detail=detail))
    for match in imports:
        for canonical,alias in use_paths(match[0]):
            if kind := category(canonical):
                emit(match.start(),'ambient-import:'+kind,canonical,'lexical import; not an operation count')
    out = list(text)
    for match in imports:
        out[match.start():match.end()] = ['\n' if c == '\n' else ' ' for c in match[0]]
    body = ''.join(out)
    # shortcut: same-file direct aliases only; require Rust name resolution before claiming admission diagnostics.
    references = re.compile(r'\b\w+(?:\s*::\s*\w+)*')
    for match in references.finditer(body):
        spelling = re.sub(r'\s','',match[0])
        head,sep,tail = spelling.partition('::')
        canonical = aliases.get(head,head)+(sep+tail if sep else '')
        if kind := category(canonical):
            emit(match.start(),'ambient-reference:'+kind,canonical,'lexical reference, not proof of a live operation')
        if canonical != 'std::thread::spawn':
            continue
        opening = re.match(r'\s*\(', body[match.end():])
        if not opening:
            continue
        call_open = match.end()+opening.end()-1
        call_end = balanced_end(body,call_open,'(',')')
        prefix_start = max(body.rfind(';',0,match.start()),body.rfind('{',0,match.start()),body.rfind('}',0,match.start()))+1
        prefix = body[prefix_start:match.start()].strip()
        suffix = body[call_end:]
        if (prefix == '' or re.fullmatch(r'let\s+_\s*=',prefix)) and re.match(r'\s*;',suffix):
            emit(match.start(),'discarded-spawn-handle',canonical,'direct expression statement or let _ = spawn(...);')
        elif re.fullmatch(r'(?:std\s*::\s*mem\s*::\s*)?drop\s*\(',prefix) and re.match(r'\s*\)\s*;',suffix):
            emit(match.start(),'discarded-spawn-handle',canonical,'direct drop(spawn(...));')
        elif re.match(r'\s*\.\s*join\s*\(',suffix):
            emit(match.start(),'joined-spawn-handle',canonical,'immediate .join()')
        else:
            emit(match.start(),'unresolved-spawn-handle',canonical,'stored/returned/transformed handle; no drop inference')
    return found


def self_test():
    text,_ = non_test('''use std::{fs as files, env::var as get, time::Instant, thread::spawn as go};
fn f() {
 files::read("x"); get("Y"); Instant::now();
 let _ = go(|| {});
 go(|| {});
 drop(go(|| {}));
 let handle = go(|| {});
 go(|| {}).join().unwrap();
 // std::net::TcpStream::connect("ignored");
}
#[cfg(test)] fn ignored() { go(|| {}); }
''')
    result = observe('fixture.rs',text)
    assert sum(r['kind']=='discarded-spawn-handle' for r in result) == 3,result
    assert sum(r['kind']=='unresolved-spawn-handle' for r in result) == 1,result
    assert sum(r['kind']=='joined-spawn-handle' for r in result) == 1,result
    assert sum(r['kind'].startswith('ambient-reference:') for r in result) == 3,result
    assert [r['line'] for r in result if r['kind']=='discarded-spawn-handle'] == [4,5,6]
    assert not any('std::net' in r['canonical'] for r in result)
    print('self-test: passed')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--self-test',action='store_true')
    p.add_argument('--repo',type=pathlib.Path)
    a = p.parse_args()
    if a.self_test:
        self_test()
    else:
        if a.repo is None:
            p.error('--repo is required (pool repositories only after go)')
        writer = csv.DictWriter(sys.stdout,['file','line','kind','canonical','detail'],delimiter='\t',lineterminator='\n')
        writer.writeheader()
        for path,text in source_texts(a.repo.resolve())[0]:
            writer.writerows(observe(path,text))
