{
  lib,
  stdenv,
  fetchurl,
  rpmextract,
}:

stdenv.mkDerivation rec {
  pname = "mingw64-filesystem";
  version = "151";

  src = fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/mingw-filesystem/${version}/1.eln154/noarch/mingw64-filesystem-${version}-1.eln154.noarch.rpm";
    sha256 = "sha256-/20D3UGgpIIkY6xh57UQNkUzwYsbizCKzGeKYNwCM9U=";
  };

  dontUnpack = true;

  nativeBuildInputs = [ rpmextract ];

  phases = [ "installPhase" ];

  installPhase = ''
    mkdir $out

    rpmextract $src
    mv ./usr/share $out
    mv ./usr/x86_64-w64-mingw32/sys-root/mingw/bin $out
  '';

  meta = {
    homepage = "https://packages.fedoraproject.org/pkgs/mingw-filesystem";
    description = "";
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
    license = lib.licenses.gpl3;
    maintainers = [ ];
    platforms = [ "x86_64-linux" ];
  };
}
