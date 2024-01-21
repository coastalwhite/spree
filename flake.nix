{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-23.11";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in with pkgs; rec {
	    packages = {
			app = pkgs.rustPlatform.buildRustPackage rec {
			  pname = "spree";
			  version = "0.0.1";
			  src = ./.;
			  cargoBuildFlags = "-p spree";

			  cargoLock = {
				lockFile = ./Cargo.lock;
			  };

			  nativeBuildInputs = [
				pkg-config
			  ];

			  buildInputs = [
				  libxkbcommon libGL
			      
				  # X11
    		      xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr

				  # Wayland
				  wayland

				  fontconfig
			  ];

			  WINIT_UNIX_BACKEND = "wayland";
			  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

			  postFixup = ''
			    patchelf --add-rpath ${ pkgs.libGL }/lib $out/bin/*
			    patchelf --add-rpath ${ pkgs.libxkbcommon }/lib $out/bin/*
			    patchelf --add-rpath ${ pkgs.wayland }/lib $out/bin/*
			    patchelf --add-rpath ${ pkgs.fontconfig }/lib $out/bin/*
			  '';
			};
		  default = packages.app;
		};

        devShell = mkShell rec {
          buildInputs = [
            libxkbcommon
            libGL
            wayland
          ];
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
        };
      });
}