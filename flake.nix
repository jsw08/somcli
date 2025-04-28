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
        lib = pkgs.lib;

        naersk' = pkgs.callPackage naersk {};
      in let
        deps = [pkgs.pkg-config pkgs.openssl];
      in {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          name = "somcli";
          src = ./.;
          buildInputs = deps;

          meta = {
            description = "Reads a somtoday ical url, caches it and displays your lessons.";
            mainProgram = "somcli";
            homepage = "https://github.com/jsw08/somcli";
            license = lib.licenses.mit;
            maintainers = [
              {
                email = "jurnwubben@gmail.com";
                github = "jsw08";
                githubId = "46420489";
                name = "Jurn Wubben";
              }
            ];
          };
        };

        # For `nix develop` (optional, can be skipped):
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [rustc cargo] ++ deps;
        };
      }
    );
}
