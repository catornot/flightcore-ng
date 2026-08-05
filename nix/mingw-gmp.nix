{
  lib,
  stdenv,
  fetchurl,
  rpmextract,
}:

stdenv.mkDerivation rec {
  pname = "mingw64-gmp";
  version = "6.3.0";

  src = fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/mingw-gmp/${version}/5.fc44/noarch/mingw64-gmp-${version}-5.fc44.noarch.rpm";
    sha256 = "sha256-q3MQBAXBDpFrMUx06EiHd5hUnbWuY0WGJ8G13EeZyfQ=";
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
    homepage = "https://packages.fedoraproject.org/pkgs/mingw-expat/mingw64-gmp/";
    description = "";
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
    license = lib.licenses.gpl3;
    maintainers = [ ];
    platforms = [ "x86_64-linux" ];
  };
}
