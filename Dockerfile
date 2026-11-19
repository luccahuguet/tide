# Tide's alpha in an x86_64 Linux container, for trying it on macOS and Windows (docs/docker.md).
# Two images share one build stage, which installs Tide by docs/install.md's route and no other:
# Nix's single-user installer, then the repository's flake. The stage copies the checkout instead
# of cloning, so an image holds the revision it was built from; .dockerignore allowlists what it
# copies, which leaves out .git, the .beads ledger and every local file.
#
#   docker build --platform linux/amd64 --target try -t tide-try .   # runs the demo, nothing else
#   docker run --rm --platform linux/amd64 tide-try
#   docker build --platform linux/amd64 --target dev -t tide .       # a shell with Nix and the flake
#   docker run --rm -it --platform linux/amd64 tide
#
# NIX_CONFIG, a build argument that the images do not keep, can cap Nix's build parallelism,
# e.g. --build-arg NIX_CONFIG=$'cores = 4\nmax-jobs = 4'.
FROM --platform=linux/amd64 ubuntu:24.04@sha256:73b999393d07f2bf65a40373778a44dc9a6d54ad7718d29247d8ff9413c4d6a8 AS nix

# The guide's prerequisites. The image has no sudo, so root creates /nix for the ubuntu user, the
# one step the installer would otherwise run with sudo.
RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        ca-certificates curl xz-utils git \
    && rm -rf /var/lib/apt/lists/* \
    && install -d -m 0755 -o ubuntu -g ubuntu /nix

USER ubuntu
ENV USER=ubuntu
WORKDIR /home/ubuntu
# Step 1's commands are bash; the PATH entry is what step 1's `. ~/.nix-profile/etc/profile.d/nix.sh`
# gives a shell.
SHELL ["/bin/bash", "-o", "pipefail", "-c"]
ENV PATH=/home/ubuntu/.nix-profile/bin:$PATH
ARG NIX_CONFIG=

# Step 1: Nix's single-user install. Flakes are on for good, as the guide's step 1 describes, so
# commands in the container need no --extra-experimental-features.
RUN sh <(curl --proto '=https' --tlsv1.2 -L https://nixos.org/nix/install) --no-daemon \
    && mkdir -p ~/.config/nix \
    && echo 'experimental-features = nix-command flakes' > ~/.config/nix/nix.conf

# Step 2: the repository, from the build context. Without .git, Nix reads the flake as a plain
# directory.
COPY --chown=ubuntu:ubuntu . tide
WORKDIR /home/ubuntu/tide

# The try image's contents: the closure of the flake's `tide-demo`, which `nix run .#tide-demo`
# runs, under /nix/store as Nix built it, with /bin/tide-demo naming its script.
FROM nix AS try-closure
RUN nix build --out-link ~/demo .#tide-demo \
    && mkdir -p ~/try/nix/store ~/try/bin ~/try/tmp \
    && cp -a $(nix-store --query --requisites ~/demo) ~/try/nix/store/ \
    && ln -s "$(readlink ~/demo)/bin/tide-demo" ~/try/bin/tide-demo \
    && chmod 1777 ~/try/tmp

# The try image: the demo's closure and nothing else, no distribution and no Nix. The demo builds
# the driver and Tide's std with the closure's toolchain into /tmp/tide-demo, then runs.
FROM scratch AS try
COPY --from=try-closure /home/ubuntu/try /
ENV HOME=/tmp CARGO_TARGET_DIR=/tmp/tide-demo
USER 1000:1000
WORKDIR /tmp
CMD ["/bin/tide-demo"]

# The dev image, which .devcontainer also builds. Step 3: the driver, as result/bin/tide-admit.
# The demo and step 6's dev shell are realized here so that `nix run .#tide-demo` and
# `nix develop` in the container start at once.
FROM nix AS dev
RUN nix build .#tidelang-admit \
    && nix build --no-link .#tide-demo \
    && nix develop --command true

CMD ["bash"]
