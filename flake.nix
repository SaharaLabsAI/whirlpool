{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/25.11";
    utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      utils,
      crane,
      rust-overlay,
      ...
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        # A useful helper for folding a list of `prevSet -> newSet` functions
        # into an attribute set.
        composeAttrOverrides =
          defaultAttrs: overrides: builtins.foldl' (acc: f: acc // (f acc)) defaultAttrs overrides;

        cargoTarget = pkgs.stdenv.hostPlatform.rust.rustcTargetSpec;
        cargoTargetEnvVar = builtins.replaceStrings [ "-" ] [ "_" ] (pkgs.lib.toUpper cargoTarget);

        cargoTOML = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        packageVersion = cargoTOML.workspace.package.version;

        # Latest stable rust toolchain (rustup "default" profile) plus IDE/lint/format extensions.
        rustStable = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "clippy"
            "rustfmt"
          ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;

        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.libgit2
          pkgs.perl
        ];

        pythonEnv = pkgs.python3.withPackages (
          ps: with ps; [
            pyyaml
          ]
        );

        withClang = prev: {
          buildInputs = prev.buildInputs or [ ] ++ [
            pkgs.clang
          ];
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
        };

        withMaxPerf = prev: {
          cargoBuildCommand = "cargo build --profile=maxperf";
          cargoExtraArgs = prev.cargoExtraArgs or "" + " --features=jemalloc,asm-keccak";
          RUSTFLAGS = prev.RUSTFLAGS or [ ] ++ [
            "-Ctarget-cpu=native"
          ];
        };

        withMold = prev: {
          buildInputs = prev.buildInputs or [ ] ++ [
            pkgs.mold
          ];
          "CARGO_TARGET_${cargoTargetEnvVar}_LINKER" = "${pkgs.llvmPackages.clangUseLLVM}/bin/clang";
          RUSTFLAGS = prev.RUSTFLAGS or [ ] ++ [
            "-Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold"
          ];
        };

        mkReth =
          overrides:
          craneLib.buildPackage (
            composeAttrOverrides {
              pname = "reth";
              version = packageVersion;
              src = ./.;
              inherit nativeBuildInputs;
              doCheck = false;
            } overrides
          );

      in
      {
        devShell =
          let
            overrides = [
              withClang
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
              withMold
            ];
          in
          craneLib.devShell (
            composeAttrOverrides {
              packages = nativeBuildInputs ++ [
                pkgs.rust-analyzer
                pkgs.cargo-nextest
                pythonEnv
              ];

              shellHook = ''
                repo_root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
                demo_path="$repo_root/devtools/demo/personality"
                if [ -d "$demo_path" ]; then
                  export PATH="$demo_path:$PATH"
                fi
              '';

              # Remove the hardening added by nix to fix jmalloc compilation error.
              # More info: https://github.com/tikv/jemallocator/issues/108
              hardeningDisable = [ "fortify" ];

            } overrides
          );
      }
    );
}
