{ pkgs ? import <nixpkgs> { } }: {
  out = pkgs.rustPlatform.buildRustPackage rec {
    pname = "crabcassonne";
    version = "0.1.0";

    cargoLock.lockFile = ./Cargo.lock;
    src = pkgs.lib.cleanSource ./.;
    nativeBuildInputs = [ pkgs.makeWrapper ];

    libPath = with pkgs; lib.makeLibraryPath [
      libGL
      libxkbcommon
      wayland
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      xorg.libXrandr
    ];



    buildInputs = with pkgs; [
      glibc
      gcc-unwrapped
    ];
    postInstall = ''
            wrapProgram "$out/bin/crabcassonne" --prefix LD_LIBRARY_PATH : "${libPath}"
            '';

#    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
    meta = { description = "Crabcassonne"; };
  };
}
