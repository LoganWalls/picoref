{
  description = "A minimal reference manager";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        flake-utils.follows = "flake-utils";
        nixpkgs.follows = "nixpkgs";
      };
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , crane
    , fenix
    , flake-utils
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        inherit (pkgs) lib stdenv;
        fenixPkgs = fenix.packages.${system};
        rustToolchain = fenixPkgs.stable.toolchain;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        buildDeps = lib.optionals stdenv.isDarwin (with pkgs.darwin.apple_sdk.frameworks; [
          Security
          pkgs.libiconv
        ]);
        crate = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          buildInputs = buildDeps;
        };
      in
      {
        checks = {
          inherit crate;
        };
        packages.default = crate;
        apps.default = flake-utils.lib.mkApp {
          drv = crate;
        };
        devShell = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;
          buildInputs = [ fenixPkgs.rust-analyzer buildDeps ];
          nativeBuildInputs = [ rustToolchain ];
        };
      }
    );
}
