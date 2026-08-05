{
  lib,
  stdenv,
  fetchurl,
  rpmextract,
}:

stdenv.mkDerivation rec {
  pname = "mingw64-expat";
  version = "2.8.1";

  src = fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/mingw-expat/${version}/1.fc44/noarch/mingw64-expat-${version}-1.fc44.noarch.rpm";
    sha256 = "sha256-JYmwS+D6LbvsA2EN8F8Q5+mKLjbLtHxBBowOWoXikRc=";
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
    homepage = "https://packages.fedoraproject.org/pkgs/mingw-expat/mingw64-expat/";
    description = "";
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
    license = lib.licenses.gpl3;
    maintainers = [ ];
    platforms = [ "x86_64-linux" ];
  };
}
