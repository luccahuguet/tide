#!/usr/bin/env bash
# Clean-machine check of docs/install.md (fx-qlek.1): runs the guide's ```sh blocks, in order and
# as written, as an ordinary user in a fresh pinned Ubuntu container that has no Rust, no rustup,
# no C compiler and no checkout. It prints a transcript and fails unless the example runs and its
# test passes in the sandbox.
#
# Where the run differs from a newcomer's machine:
# - the clone URL: the repository is not public, so `git clone <url> tide` clones a git bundle of
#   HEAD instead;
# - /nix exists before step 1, an empty bind mount of RUN_DIR/nix owned by the host user, so the
#   installer's sudo step that creates /nix does not run (the image has no sudo). The host user
#   must therefore be uid 1000, the image's `ubuntu`; the check refuses any other uid;
# - the prerequisites are installed as root with `apt-get update` and `apt-get install
#   --no-install-recommends ca-certificates curl xz-utils git`, not with the guide's sudo line;
# - NIX_CONFIG caps Nix at 4 cores and 2 jobs;
# - stdin is /dev/null, so step 6's `nix develop` realizes the shell and exits at once; no cargo
#   command runs in it.
# Nix's own build sandbox is off in the container (Docker's seccomp blocks user namespaces), so
# this check proves the steps work on a machine without rustup or a system C compiler. It does not
# prove that they ignore those on a machine that has them; fx-1n5n's `checks.link`, built with
# `--option sandbox true`, is that proof.
#
# Usage: scripts/clean-install-check.sh RUN_DIR
#   RUN_DIR must be new or empty, on a disk with about 10 GB free: it holds the container's /nix
#   and /tmp, so Docker's root disk gets only the image (about 78 MB, kept for reruns; remove it
#   with `docker rmi`) and a small container layer. The check deletes them at exit and keeps
#   RUN_DIR/transcript.log.
#   A SIGKILL skips cleanup. To recover, run `docker rm -f "$(cat RUN_DIR/container.id)"`, then
#   `docker run --rm -v RUN_DIR:/work IMAGE find /work/nix /work/tmp /work/check -mindepth 1 -delete`.
# It compiles Rust inside the container, so run it as a heavy run (AGENTS.md, Heavy runs) under a
# timeout; the container is capped at 4 CPUs.
set -euo pipefail

image=ubuntu:24.04@sha256:73b999393d07f2bf65a40373778a44dc9a6d54ad7718d29247d8ff9413c4d6a8
expected_stdout='[3, 7, 9, 14, 20]'
# The guide's steps; extraction fails on any other count, so a step cannot drop out unnoticed.
expected_blocks=6

die() { echo "clean-install-check: $*" >&2; exit 1; }
[ $# -eq 1 ] || die "usage: $0 RUN_DIR"
uid=$(id -u)
[ "$uid" = 1000 ] || die "run as uid 1000, the image's ubuntu user, who must own RUN_DIR/nix (you are uid $uid)"
repo=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
dirty=$(git -C "$repo" status --porcelain)
[ -z "$dirty" ] || die "$repo has uncommitted changes; the check clones HEAD"
mkdir -p "$1"
run=$(realpath "$1")
listing=$(ls -A "$run")
[ -z "$listing" ] || die "$run is not empty"
mkdir "$run/nix" "$run/tmp" "$run/check"
chmod 1777 "$run/tmp"
# tee ignores the signals that timeout sends to the whole process group, so it outlives them and
# records cleanup's output; without it, cleanup's first write would end in SIGPIPE.
exec > >(trap '' TERM INT HUP; exec tee "$run/transcript.log") 2>&1

# Docker writes the ID of the container it creates to this fresh file. Cleanup removes only that
# container, and none when the file is missing or empty.
cidfile=$run/container.id
rev=
passed=
# Runs at every exit. It keeps the exit status, removes this run's container, then empties the
# bind-mounted directories (only once no container of this run can still use them). A cleanup
# failure fails the check; PASS is printed only after cleanup succeeds.
finish() {
  local rc=$? clean=1 cid=
  set +e
  if [ -s "$cidfile" ]; then
    cid=$(cat "$cidfile")
    if docker rm -f "$cid" >/dev/null; then rm -f "$cidfile"; else clean=; fi
  else
    rm -f "$cidfile"
  fi
  if [ -n "$clean" ] && [ -n "$(ls -A "$run/nix" "$run/tmp" "$run/check")" ]; then
    # The store's files are read-only and some of /tmp is root's, so root in the image deletes them.
    docker run --rm -v "$run:/work" "$image" find /work/nix /work/tmp /work/check -mindepth 1 -delete \
      || clean=
  fi
  if [ -n "$clean" ]; then rmdir "$run/nix" "$run/tmp" "$run/check" || clean=; fi
  echo "### disk after cleanup"; df -h / "$run"
  if [ -z "$clean" ]; then
    echo "clean-install-check: cleanup failed; $run or container ${cid:-(none)} is left" >&2
    if [ "$rc" -eq 0 ]; then rc=1; fi
  fi
  if [ "$rc" -eq 0 ] && [ -n "$passed" ]; then
    echo "### $(date -u +%FT%TZ) clean-install-check: PASS ($rev)"
  else
    if [ "$rc" -eq 0 ]; then rc=1; fi
    echo "### $(date -u +%FT%TZ) clean-install-check: FAIL (exit $rc)"
  fi
  exit "$rc"
}
trap finish EXIT
trap 'exit 143' TERM INT HUP

rev=$(git -C "$repo" rev-parse HEAD)
docker_version=$(docker version --format '{{.Server.Version}}')
echo "### clean-install-check $(date -u +%FT%TZ)"
echo "revision: $rev"
echo "image: $image"
echo "host: $(uname -srm); docker $docker_version"
echo "### disk before"; df -h / "$run"

git -C "$repo" bundle create "$run/check/tide.bundle" HEAD
# The guide's ```sh blocks become the user's script, each echoed before it runs. Only ```sh (a
# step) and ```text (output) fences are allowed, unindented, so a step fenced any other way fails
# here instead of dropping out. The clone line, `git clone <url> tide`, must occur exactly once;
# only its URL changes, to the bundle.
{
  echo 'set -eo pipefail'
  echo 'cd ~'
  git -C "$repo" show HEAD:docs/install.md | awk -v expected="$expected_blocks" '
    function bad(msg) { print "docs/install.md:" NR ": " msg > "/dev/stderr"; failed = 1; exit 1 }
    /^[ \t]*(```|~~~)/ {
      if (kind == "" && $0 == "```sh") { kind = "sh"; n++; lines = ""; next }
      if (kind == "" && $0 == "```text") { kind = "text"; next }
      if (kind != "" && $0 == "```") {
        if (kind == "sh") {
          printf "printf \"\\n### docs/install.md block %d (%%s)\\n\" \"$(date -u +%%T)\"\ncat <<'\''TIDE_BLOCK'\''\n%sTIDE_BLOCK\n%s", n, lines, lines
        }
        kind = ""
        next
      }
      bad("fence \"" $0 "\": steps are ```sh and output is ```text, unindented")
    }
    kind == "sh" {
      if ($0 ~ /^git clone [^ ]+ tide$/) { $0 = "git clone /check/tide.bundle tide"; clones++ }
      lines = lines $0 "\n"
    }
    END {
      if (failed) exit 1
      if (kind != "") bad("unclosed fence")
      if (n != expected) bad(n " sh blocks, expected " expected)
      if (clones != 1) bad(clones + 0 " lines \"git clone <url> tide\", expected 1")
    }
  '
} > "$run/check/steps.sh" || die "docs/install.md: cannot extract the guide's steps"

# Root prepares the machine: proves what is absent, installs the guide's prerequisites, then runs
# the guide as the image's ordinary user, ubuntu (uid 1000, the owner of RUN_DIR's /nix).
cat > "$run/check/machine.sh" <<'EOF'
set -euo pipefail
fail() { echo "clean-install-check: $*" >&2; exit 1; }
echo "### machine"
grep PRETTY_NAME /etc/os-release
absent() {
  for p in /usr/bin/cc /usr/bin/gcc /usr/bin/ld /root/.rustup /root/.cargo /home/ubuntu/.rustup \
    /home/ubuntu/.cargo; do
    if [ -e "$p" ]; then fail "$p exists ($1)"; fi
  done
  for c in rustc cargo rustup cc gcc ld nix; do
    if command -v "$c" >/dev/null; then fail "$c is on PATH ($1)"; fi
  done
  echo "absent ($1): /usr/bin/{cc,gcc,ld}; ~/.rustup and ~/.cargo of root and ubuntu;" \
    "rustc, cargo, rustup, cc, gcc, ld and nix on root's PATH"
}
if [ -e /home/ubuntu/tide ]; then fail "the checkout ~ubuntu/tide exists"; fi
nix_listing=$(ls -A /nix)
if [ -n "$nix_listing" ]; then fail "/nix is not empty"; fi
echo "absent: the checkout ~ubuntu/tide; /nix is empty"
echo "### prerequisites (docs/install.md): curl, xz-utils, git"
apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
  ca-certificates curl xz-utils git >/dev/null
absent "after the prerequisites"
runuser -l ubuntu -w NIX_CONFIG -c 'bash /check/steps.sh'
echo "### after the guide"
absent "after the guide"
driver=$(readlink /home/ubuntu/tide/result)
echo "driver: $driver"
EOF

# NIX_CONFIG caps Nix's build parallelism to the container's CPUs. docker run pulls the pinned image
# when it is missing. The pipeline ends only after tee has written the whole guide log, which the
# checks below read.
docker run --cidfile "$cidfile" --init --platform linux/amd64 --cpus 4 \
  -e NIX_CONFIG=$'cores = 4\nmax-jobs = 2' \
  -v "$run/nix:/nix" -v "$run/tmp:/tmp" -v "$run/check:/check:ro" \
  "$image" bash /check/machine.sh 2>&1 | tee "$run/check/guide.log"

echo "### disk before cleanup"; df -h / "$run"
log=$run/check/guide.log
grep -qxF "$expected_stdout" "$log" || die "the example did not print $expected_stdout"
grep -q '^sandbox: ' "$log" || die "no sandbox: line"
grep -q '^test result: ok\. 1 passed' "$log" || die "the example's test did not pass"
passed=1
