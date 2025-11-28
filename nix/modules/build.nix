{ self, inputs, ... }:
{
  perSystem =
    { config
    , self'
    , pkgs
    , lib
    , system
    , ...
    }:
    let
      # Get the Rust package from the rust-flake configuration
      rustPackage = self'.packages.sportsday-scoreboard-v2;

      # Fetch node_modules as a fixed-output derivation
      nodeDependencies = pkgs.stdenv.mkDerivation {
        pname = "sportsday-scoreboard-node-modules";
        version = "0.1.0";

        src = lib.cleanSourceWith {
          src = self;
          filter =
            path: type:
            let
              baseName = baseNameOf path;
            in
            baseName == "package.json" || baseName == "bun.lock" || baseName == "bun.lockb";
        };

        nativeBuildInputs = [ pkgs.bun ];

        buildPhase = ''
          export HOME=$TMPDIR
          export BUN_INSTALL_CACHE_DIR=$TMPDIR/.bun-cache

          echo "Installing dependencies with bun..."
          bun install --frozen-lockfile --no-progress
        '';

        installPhase = ''
          mkdir -p $out
          cp -r node_modules $out/
        '';

        outputHashMode = "recursive";
        outputHashAlgo = "sha256";
        outputHash = "sha256-AtMImkNZ0uRH+2T0CnIPZX7N2zXZ3CcF51NBYumjnzM=";
      };

      # Build the frontend assets using the fetched dependencies
      frontendAssets = pkgs.stdenv.mkDerivation {
        pname = "sportsday-scoreboard-frontend";
        version = "0.1.0";
        src = self;

        nativeBuildInputs = [ pkgs.bun ];

        buildPhase = ''
          export HOME=$TMPDIR

          # Link the pre-fetched node_modules
          ln -s ${nodeDependencies}/node_modules node_modules

          echo "Building frontend assets..."
          bun run scripts/build.ts
        '';

        installPhase = ''
          mkdir -p $out
          cp -r assets $out/
        '';
      };

    in
    {
      packages = {
        # Streamlined Docker image using layered approach for better caching
        dockerImage = pkgs.dockerTools.streamLayeredImage {
          name = "sportsday-scoreboard-v2";
          tag = "latest";

          contents = [
            pkgs.sqlite
            pkgs.coreutils
            pkgs.bash
            pkgs.cacert
            rustPackage
            frontendAssets
          ];

          extraCommands = ''
            mkdir -p app
            ${lib.optionalString (builtins.pathExists "${self}/config.yaml") ''
              cp ${self}/config.yaml app/config.yaml
            ''}
            # Copy templates directory for runtime access (if needed by Askama)
            ${lib.optionalString (builtins.pathExists "${self}/templates") ''
              mkdir -p app/templates
              cp -r ${self}/templates/* app/templates/
            ''}
            # Copy frontend assets to /app/assets
            mkdir -p app/assets
            cp -r ${frontendAssets}/assets/* app/assets/
          '';

          config = {
            Cmd = [ "${rustPackage}/bin/sportsday-scoreboard-v2" ];
            WorkingDir = "/app";
            ExposedPorts = {
              "3000/tcp" = { };
            };
            Env = [
              "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            ];
          };
        };
      };
    };
}
