{
  lib,
  rustPlatform,
  makeWrapper,
  pkgs,
  pkg-config,
  umu-launcher,
  zenity,

  libx11,
  libxcursor,
  libxrandr,
  wayland,
  libxkbcommon,
  vulkan-loader,
}:
let
  cargoLock = (import ./cargo_lock.nix { });
in
rustPlatform.buildRustPackage (finalAttrs: {
  name = "flightcore-ng";

  src = ../.;

  nativeBuildInputs = [
    pkg-config
    makeWrapper
  ];

  cargoBuildFlags = [
    "--package"
    finalAttrs.name
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

  LD_LIBRARY_PATH = builtins.foldl' (
    a: b: "${a}:${b}/lib"
  ) "${vulkan-loader}/lib" finalAttrs.runtimeDependencies;

  postInstall = ''
    wrapProgram $out/bin/${finalAttrs.name}  --prefix LD_LIBRARY_PATH : ${finalAttrs.LD_LIBRARY_PATH} --prefix PATH : ${
      lib.makeBinPath [
        umu-launcher
        zenity
      ]
    }
  '';

  meta = {
    description = "Next Generation Installer/Updater/Launcher for Northstar";
    homepage = "https://github.com/catornot/flightcore-ng";
    license = lib.licenses.mit;
    maintainers = [ "cat_or_not" ];
    mainProgram = finalAttrs.name;
  };

  inherit cargoLock;
})
