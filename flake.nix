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
        };
        toolchain = (pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);

      in
      {
        formatter = pkgs.nixfmt-tree;

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            toolchain
            pkgs.umu-launcher
            pkgs.zenity
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
