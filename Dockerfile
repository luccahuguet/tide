# Tide's alpha in an x86_64 Linux container, for trying it on macOS and Windows (docs/docker.md).
# It installs Tide by docs/install.md's route and no other: Nix's single-user installer, then the
# repository's flake. The image copies the checkout instead of cloning, so it holds the revision
# it was built from; .dockerignore allowlists what it copies, which leaves out .git, the .beads
# ledger and every local file.
#
#   docker build --platform linux/amd64 -t tide .
#   docker run --rm -it --platform linux/amd64 tide
#
# NIX_CONFIG, a build argument that the image does not keep, can cap Nix's build parallelism,
# e.g. --build-arg NIX_CONFIG=$'cores = 4\nmax-jobs = 4'.
FROM --platform=linux/amd64 ubuntu:24.04@sha256:73b999393d07f2bf65a40373778a44dc9a6d54ad7718d29247d8ff9413c4d6a8

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

# Step 3: the driver, as result/bin/tide-admit. Step 6: the dev shell, realized here so that
# `nix develop` in the container starts at once.
RUN nix build .#tidelang-admit \
    && nix develop --command true

CMD ["bash"]
