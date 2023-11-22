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
      inputs.nixpkgs.follows = "nixpkgs";
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
        buildDeps = lib.optionals stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
            Security
            pkgs.libiconv
          ]);
        crate = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);
          strictDeps = true;
          buildInputs = buildDeps;
        };
      in {
        apps.${system}.default = let
          name = crate.pname or crate.name;
          exe = crate.passthru.exePath or "/bin/${name}";
        in {
          type = "app";
          program = crate.passthru.exePath or "/bin/${name}";
        };
        packages.${system}.default = crate;
        checks.${system} = {inherit crate;};
        devShells.${system}.default = pkgs.mkShell {
          packages =
            [
              toolchain
              pkgs.rust-analyzer-unwrapped
            ]
            ++ buildDeps;
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      }
    );
}
