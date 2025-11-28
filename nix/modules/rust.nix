{ inputs, self, ... }:
{
  imports = [
    inputs.rust-flake.flakeModules.default
    inputs.rust-flake.flakeModules.nixpkgs
    inputs.process-compose-flake.flakeModule
    inputs.cargo-doc-live.flakeModule
  ];
  perSystem =
    { config
    , self'
    , pkgs
    , lib
    , ...
    }:
    {
      # Override the rust-project.src to include templates directory for Askama
      rust-project.src = lib.cleanSourceWith {
        src = self;
        filter =
          path: type:
          # Include templates directory for Askama
          (lib.hasInfix "/templates" path)
          || (
            config.rust-project.crateNixFile != null
            && lib.hasSuffix "/${config.rust-project.crateNixFile}" path
          )
          ||
          # Default filter from crane (allow .rs files, Cargo.toml, etc.)
          (config.rust-project.crane-lib.filterCargoSources path type);
      };

      rust-project.crates."sportsday-scoreboard-v2".crane.args = {
        buildInputs = lib.optionals pkgs.stdenv.isDarwin (
          with pkgs.darwin.apple_sdk.frameworks;
          [
            IOKit
          ]
        );
      };
      packages.default = self'.packages.sportsday-scoreboard-v2;
    };
}
