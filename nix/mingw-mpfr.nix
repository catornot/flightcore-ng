{
  lib,
  stdenv,
  fetchurl,
  rpmextract,
}:

stdenv.mkDerivation rec {
  pname = "mingw64-mpfr";
  version = "4.0.2";

  src = fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/mingw-mpfr/${version}/16.fc44/noarch/mingw64-mpfr-${version}-16.fc44.noarch.rpm";
    sha256 = "sha256-1BVfvPn0P+T25m1Qm9IZJ9NnJ01Y+sRpDutKCTklYjM=";
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
    homepage = "https://packages.fedoraproject.org/pkgs/mingw-mpfr";
    description = "";
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
    license = lib.licenses.gpl3;
    maintainers = [ ];
    platforms = [ "x86_64-linux" ];
  };
}
