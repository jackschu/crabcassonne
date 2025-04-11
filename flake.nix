# maintain with
# nix flake lock --update-input crate2nix
# nix run github:nix-community/crate2nix
{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crate2nix.url = "github:nix-community/crate2nix";
  };

  outputs = { self, nixpkgs, flake-utils, crate2nix, rust-overlay }:
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
            [ rust-overlay.overlays.default ];
        };
        crateName = "crabcassonne";

        inherit (import "${crate2nix}/tools.nix" { inherit pkgs; })
          generatedCargoNix;

        project = pkgs.callPackage (generatedCargoNix {
          name = crateName;
          src = ./.;
        }) {
          defaultCrateOverrides = pkgs.defaultCrateOverrides // {
            crabcassonne = drv: {
              propagatedBuildInputs = drv.propagatedBuildInputs or [ ]
                                      ++ [ pkgs.glibc pkgs.makeWrapper pkgs.gcc-unwrapped ];
            };
          };
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
        # rustPkgs = pkgs.rustBuilder.makePackageSet {
        #   rustVersion = "1.86.0";
        #   packageFun = import ./Cargo.nix;
        #   target = systemTargets.${system};

        #   # Use the existing all list of overrides and append your override
        #   packageOverrides = pkgs:
        #     pkgs.rustBuilder.overrides.all ++ [

        #       # parentheses disambiguate each makeOverride call as a single list element
        #       (pkgs.rustBuilder.rustLib.makeOverride {
        #         name = "crabcassonne";
        #         overrideAttrs = drv: {
        #           propagatedBuildInputs = drv.propagatedBuildInputs or [ ]
        #             ++ [ pkgs.glibc pkgs.makeWrapper pkgs.gcc-unwrapped ];
        #         };
        #       })
        #     ];
        # };

        # crab-wrapped = pkgs.runCommand "crabcassonne" {
        #   meta = rustPkgs.workspace.crabcassonne.meta or { };
        #   passthru = (rustPkgs.workspace.crabcassonne.passthru or { }) // {
        #     unwrapped = rustPkgs.workspace.crabcassonne;
        #   };
        #   nativeBuildInputs = [ pkgs.makeWrapper ];
        #   makeWrapperArgs = [ ];
        # } ''
        #   cp -rs --no-preserve=mode,ownership "${
        #     rustPkgs.workspace.crabcassonne { }
        #   }" $out
        #   wrapProgram "$out/bin/crabcassonne" --prefix LD_LIBRARY_PATH : "${libPath}"
        # '';
#        target = ${system};
        built = project.rootCrate.build;
        crab-wrapped = pkgs.runCommand "crabcassonne" {
          meta = built.meta or { };
          passthru = (built.passthru or { }) // {
            unwrapped = built;
          };
          nativeBuildInputs = [ pkgs.makeWrapper ];
          makeWrapperArgs = [ ];
        } ''
          cp -rs --no-preserve=mode,ownership "${
            built
          }" $out
          wrapProgram "$out/bin/crabcassonne" --prefix LD_LIBRARY_PATH : "${libPath}"
        '';


      in rec {

        packages = {
          crab-wrapped = crab-wrapped;
          crabcassonne = project.rootCrate.build;
          default = packages.crabcassonne;
        };
      });
}
