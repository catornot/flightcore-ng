{
  lib,
  rustPlatform,
  pkgs,
  makeWrapper,
  pkg-config,
  umu-launcher,
  zenity,
}:
let
  cargoLock = (import ./cargo_lock.nix { });
in
rustPlatform.buildRustPackage (finalAttrs: {
  name = "flightcore-ng-cli";

  src = ../.;

  rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;

  nativeBuildInputs = [
    finalAttrs.rustToolchain
    pkg-config
    makeWrapper
  ];

  cargoBuildFlags = [
    "--package"
    finalAttrs.name
  ];

  postInstall = ''
    wrapProgram $out/bin/${finalAttrs.name} --prefix PATH : ${
      lib.makeBinPath [
        umu-launcher
        zenity
      ]
    }
  '';

  meta = {
    description = "Next Generation Installer/Updater/Launcher cli for Northstar";
    homepage = "https://github.com/catornot/flightcore-ng";
    license = lib.licenses.mit;
    maintainers = [ "cat_or_not" ];
    mainProgram = finalAttrs.name;
  };

  inherit cargoLock;
})
