{
  description = "Tide: the tidelang-admit admission driver and Tide's std, tidelang-std";

  # docs/nix.md records these inputs (AP-DEPENDENCY-001). nixpkgs is Eon's pin; rust-overlay is
  # newer than Eon's because rust-toolchain.toml's nightly postdates Eon's pin.
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/567a49d1913ce81ac6e9582e3553dd90a955875f";
    rust-overlay = {
      url = "github:oxalica/rust-overlay/49f68bf7cd35f3ef31fca584a22cf3d4e13f6fec";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self, nixpkgs, rust-overlay }:
    let
      # x86_64-unknown-linux-gnu is the only host the driver links on; aarch64-darwin is fx-w7up.
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      inherit (pkgs) lib;

      # rust-toolchain.toml owns the pin: the nightly, rustc-dev and rust-src.
      toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      rustPlatform = pkgs.makeRustPlatform {
        cargo = toolchain;
        rustc = toolchain;
      };

      # The driver's sysroot: rustc and its `lib`, as real files in one store path. rust-overlay's
      # toolchain joins its components with symbolic links, and the driver recognizes sysroot
      # libraries only by paths that resolve inside the sysroot (REPLAY-2's dependency identity).
      # The copy's ELF search paths are moved from the joined toolchain to the copy, because rustc
      # finds its sysroot from the librustc_driver it loads.
      sysroot =
        pkgs.runCommand "tidelang-sysroot-${toolchain.version or "nightly"}"
          { nativeBuildInputs = [ pkgs.patchelf ]; }
          ''
            mkdir -p $out/bin
            cp -L ${toolchain}/bin/rustc $out/bin/
            cp -rL ${toolchain}/lib $out/
            chmod -R u+w $out
            joined=${toolchain}
            find $out -type f | while read -r file; do
              rpath=$(patchelf --print-rpath "$file" 2>/dev/null) || continue
              case "$rpath" in
                *"$joined"*) patchelf --set-rpath "''${rpath//"$joined"/$out}" "$file" ;;
              esac
            done
            test -z "$(find $out -type l)"
            if grep -rlF ${toolchain} $out; then
              echo "the sysroot still names ${toolchain}" >&2
              exit 1
            fi
            test "$($out/bin/rustc --print sysroot)" = "$out"
          '';

      # Only what Cargo reads, so edits elsewhere rebuild nothing.
      src = lib.fileset.toSource {
        root = ./.;
        fileset = lib.fileset.unions [
          ./Cargo.toml
          ./Cargo.lock
          ./crates
        ];
      };
      cargoDeps = rustPlatform.importCargoLock { lockFile = ./Cargo.lock; };

      # The linker the driver bakes in (TIDE_LINKER_BIN): the cc wrapper's bin holds `cc` and the
      # binutils wrapper's `ld`, so linking reads nothing under /usr/bin.
      linkerBin = "${pkgs.stdenv.cc}/bin";
      # Where the executables that linker makes find glibc and libgcc (their DT_RUNPATH): RUN-3
      # grants a test's libraries from these trusted directories (TIDE_RUNTIME_LIB_DIRS).
      runtimeLibDirs = lib.concatMapStringsSep ":" (pkg: "${lib.getLib pkg}/lib") [
        pkgs.stdenv.cc.libc
        pkgs.stdenv.cc.cc
      ];

      # Cargo's own invocation, as README's demo build runs it, with the dependencies vendored
      # from Cargo.lock. Its stdout is Cargo's JSON messages, from which installs take artifact
      # paths, as README does, because the pinned Cargo's build directory layout has no
      # stable `deps` directory.
      cargoBuild =
        {
          pname,
          cargoArgs,
          installPhase,
          env ? { },
          dontStrip ? false,
        }:
        pkgs.stdenv.mkDerivation {
          inherit
            pname
            src
            cargoDeps
            installPhase
            dontStrip
            ;
          # Cargo comes from the toolchain; rustc, and so the driver's sysroot, from `sysroot`.
          env = {
            RUSTC = "${sysroot}/bin/rustc";
          }
          // env;
          version = "0.0.0";
          nativeBuildInputs = [
            toolchain
            rustPlatform.cargoSetupHook
            pkgs.jq
          ];
          buildPhase = ''
            runHook preBuild
            cargo build --offline --locked -j "$NIX_BUILD_CORES" \
              --message-format=json-render-diagnostics ${cargoArgs} > cargo-messages.json
            runHook postBuild
          '';
          doCheck = false;
        };

      # Tide's std as README's demo build makes it: the dev profile, which the driver's tests and
      # the conformance corpus also use.
      tidelang-std = cargoBuild {
        pname = "tidelang-std";
        cargoArgs = "-p tide_std";
        # Linked programs keep the std's debug information, as README says.
        dontStrip = true;
        installPhase = ''
          runHook preInstall
          mkdir -p $out/lib/deps
          # Every library the build made, the std's dependencies and its macros included, goes to
          # lib/deps (`--tide-dep-dir`), the std's own rlib included (`--tide-std`).
          jq -r 'select(.reason == "compiler-artifact") | .filenames[]
            | select(test("\\.(rlib|so)$"))' cargo-messages.json \
            | xargs -d '\n' cp -t $out/lib/deps
          runHook postInstall
        '';
      };

      tidelang-admit = cargoBuild {
        pname = "tidelang-admit";
        cargoArgs = "--release -p tide-admit";
        env = {
          TIDE_LINKER_BIN = linkerBin;
          TIDE_RUNTIME_LIB_DIRS = runtimeLibDirs;
          TIDE_DEFAULT_STD = "${tidelang-std}/lib/deps/libtide_std.rlib";
          TIDE_DEFAULT_DEP_DIR = "${tidelang-std}/lib/deps";
        };
        installPhase = ''
          runHook preInstall
          install -Dm755 target/release/tide-admit $out/bin/tide-admit
          runHook postInstall
        '';
      };

      tide-admit = lib.getExe' tidelang-admit "tide-admit";
    in
    {
      packages.${system} = {
        inherit tidelang-admit tidelang-std;
        default = tidelang-admit;
      };

      apps.${system} = rec {
        tidelang-admit = {
          type = "app";
          program = tide-admit;
          meta.description = "Tide's admission driver, with Tide's std built in";
        };
        default = tidelang-admit;
      };

      devShells.${system}.default = pkgs.mkShell {
        # The pinned toolchain, cargo included, for working on the workspace. A driver Cargo
        # builds here gets the packages' sysroot, linker and trusted runtime directories, so its
        # link builds and RUN-3 runs read nothing under /usr/bin (docs/nix.md).
        packages = [ toolchain ];
        env = {
          RUSTC = "${sysroot}/bin/rustc";
          TIDE_LINKER_BIN = linkerBin;
          TIDE_RUNTIME_LIB_DIRS = runtimeLibDirs;
        };
      };

      # The acceptance proof: in the sandbox, which has no /usr/bin, the driver alone admits,
      # links and runs a Tide program, and nothing the flake runs names /usr/bin.
      checks.${system}.link =
        pkgs.runCommand "tidelang-link-check"
          {
            example = ./examples/contracts;
            nativeBuildInputs = [ pkgs.binutils ];
          }
          ''
            set -eu
            test ! -e /usr/bin/cc
            test ! -e /usr/bin/ld
            cp -r $example pkg
            ${tide-admit} --crate-type=bin --emit=link --crate-name=sorted_insert \
              --out-dir out pkg/sorted_insert.rs
            test "$(out/sorted_insert)" = "[3, 7, 9, 14, 20]"
            readelf -l out/sorted_insert | grep -F 'interpreter: ${builtins.storeDir}/'
            # RUN-3's run mode grants the test its libraries from the driver's trusted runtime
            # directories, the store's glibc and libgcc.
            ${tide-admit} --test --emit=link --crate-name=sorted_insert --out-dir tests \
              --tide-run-test __tide_contract_tests::insert_sorted_3 pkg/sorted_insert.rs
            # What runs: the driver, the std's proc-macro and the linked program. The std's rlibs
            # are rustc's inputs; syn's carries its docs' `#!/usr/bin/env` shebang example.
            files=$(find ${tidelang-admit} ${tidelang-std} out tests/sorted_insert-test -type f \
              \( -perm -u+x -o -name '*.so' \))
            test -n "$files"
            # grep exits 1 for no match; a match (0) or an error (2) fails the check.
            status=0
            grep -lF /usr/bin $files || status=$?
            if [ "$status" -ne 1 ]; then
              echo "grep for /usr/bin exited $status: a match or a read error" >&2
              exit 1
            fi
            touch $out
          '';
    };
}
