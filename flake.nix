{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk"; #https://github.com/nix-community/naersk
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};
      in let
        deps = [pkgs.pkg-config pkgs.openssl];
      in {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          name = "somcli";
          src = ./.;
          buildInputs = deps;
        };

        # For `nix develop` (optional, can be skipped):
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo] ++ deps;
        };
      }
    );
}
