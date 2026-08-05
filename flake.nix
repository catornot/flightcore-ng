{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      utils,
      rust-overlay,
    }:
    utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
          config.allowUnfree = true;
        };
        toolchain = (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);

      in
      {
        formatter = pkgs.nixfmt-tree;

        packages = {
          cli = pkgs.callPackage ./nix/cli.nix { };

          gui = pkgs.callPackage ./nix/gui.nix { };

          mingw-gdb = pkgs.callPackage ./nix/mingw-gdb.nix {
            inherit (self.packages.${system})
              mingw-stdlibcpp
              mingw-libgcc
              mingw-winpthreads
              mingw-expat
              mingw-gmp
              mingw-mpfr
mingw-filesystem       mingw-crt       ;
          };
          mingw-libgcc = pkgs.callPackage ./nix/mingw-libgcc.nix { };
          mingw-stdlibcpp = pkgs.callPackage ./nix/mingw-libstdcpp.nix {
          };
          mingw-winpthreads = pkgs.callPackage ./nix/mingw-winpthreads.nix { };
          mingw-expat = pkgs.callPackage ./nix/mingw-expat.nix { };
          mingw-gmp = pkgs.callPackage ./nix/mingw-gmp.nix { };
          mingw-mpfr = pkgs.callPackage ./nix/mingw-mpfr.nix { };
          mingw-filesystem= pkgs.callPackage ./nix/mingw-filesystem.nix { };
          mingw-crt= pkgs.callPackage ./nix/mingw-crt.nix { };

          default = self.packages.${system}.gui;
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            toolchain
            pkgs.umu-launcher
            pkgs.zenity
            (self.packages.${system}.mingw-gdb)
          ];

          runtimeDependencies = with pkgs; [
            libgcc
            stdenv.cc
            zstd
            libxkbcommon
            vulkan-loader
            libx11
            libxcursor
            libxi
            libxrandr
            alsa-lib-with-plugins
            wayland
            glfw
            udev
          ];
          LD_LIBRARY_PATH = nixpkgs.lib.makeLibraryPath self.devShells.${system}.default.runtimeDependencies;
        };
      }
    );
}
