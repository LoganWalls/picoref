{
  description = "A minimal reference manager";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
  outputs = {
    nixpkgs,
    rust-overlay,
    crane,
    ...
  }: let
    inherit (nixpkgs) lib;
    withSystem = f:
      lib.fold lib.recursiveUpdate {}
      (map f ["x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"]);
  in
    withSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        inherit (pkgs) stdenv lib;
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;
        jsPackage = pkgs.buildNpmPackage {
          pname = "bibtex-converter";
          version = "0.1.0";
          src = ./js;
          npmDepsHash = "sha256-PMsVA12HldZfMptP0fQe9GR+ZT2Wf30Kku/VHoCsbx4=";
          installPhase = ''
            cp -r dist/ $out/
          '';
        };
        buildDeps = with pkgs; (
          [
            esbuild
          ]
          ++ lib.optionals stdenv.isDarwin [
            libiconv
          ]
        );
        crate = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          nativeBuildInputs = buildDeps;
          preBuild = ''
            mkdir -p js
            rm -rf js/dist
            ln -s ${jsPackage} js/dist
          '';
        };
      in {
        apps.${system}.default = let
          name = crate.pname or crate.name;
          exe = crate.passthru.exePath or "/bin/${name}";
        in {
          type = "app";
          program = "${crate}${exe}";
        };
        packages.${system}.default = crate;
        checks.${system} = {inherit crate;};
        devShells.${system}.default = pkgs.mkShell {
          packages = with pkgs;
            [
              toolchain
              rust-analyzer-unwrapped
              glow
              nodePackages.npm
            ]
            ++ buildDeps;
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}
