# maintain with
# nix flake lock --update-input cargo2nix
# nix run github:cargo2nix/cargo2nix
# patch -p1 -R -i zvariant.patch 
{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    cargo2nix.url = "github:cargo2nix/cargo2nix";
  };

  outputs = { self, nixpkgs, flake-utils, cargo2nix, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        systemTargets = {
          "x86_64-linux" = "x86_64-unknown-linux-gnu";
          "x86_64-darwin" = "x86_64-apple-darwin";
        };
      in let
        pkgs = import nixpkgs {
          inherit system;
          overlays =
            [ cargo2nix.overlays.default rust-overlay.overlays.default ];
        };

        libPath = with pkgs;
          lib.makeLibraryPath [
            libGL
            libxkbcommon
            wayland
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ];
        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.69.0";
          packageFun = import ./Cargo.nix;
          target = systemTargets.${system};

          # Use the existing all list of overrides and append your override
          packageOverrides = pkgs:
            pkgs.rustBuilder.overrides.all ++ [

              # parentheses disambiguate each makeOverride call as a single list element
              (pkgs.rustBuilder.rustLib.makeOverride {
                name = "crabcassonne";
                overrideAttrs = drv: {
                  propagatedBuildInputs = drv.propagatedBuildInputs or [ ]
                    ++ [ pkgs.glibc pkgs.makeWrapper pkgs.gcc-unwrapped ];
                };
              })
            ];
        };

        crab-wrapped = pkgs.runCommand "crabcassonne" {
          meta = rustPkgs.workspace.crabcassonne.meta or { };
          passthru = (rustPkgs.workspace.crabcassonne.passthru or { }) // {
            unwrapped = rustPkgs.workspace.crabcassonne;
          };
          nativeBuildInputs = [ pkgs.makeWrapper ];
          makeWrapperArgs = [ ];
        } ''
          cp -rs --no-preserve=mode,ownership "${
            rustPkgs.workspace.crabcassonne { }
          }" $out
          wrapProgram "$out/bin/crabcassonne" --prefix LD_LIBRARY_PATH : "${libPath}"
        '';

      in rec {
        packages = {
          crabcassonne = crab-wrapped;
          default = packages.crabcassonne;
        };
      });
}
