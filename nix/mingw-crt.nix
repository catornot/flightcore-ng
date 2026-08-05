{
  lib,
  stdenv,
  fetchurl,
  rpmextract,
}:

stdenv.mkDerivation rec {
  pname = "mingw64-crt";
  version = "14.0.0";

  src = fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/mingw-crt/${version}/2.eln156/noarch/mingw64-crt-${version}-2.eln156.noarch.rpm";
    sha256 = "sha256-O3wDl3zz+XifdmPoEtleVxOJEJbBSaKMi0FQmM6ibro=";
  };

  dontUnpack = true;

  nativeBuildInputs = [ rpmextract ];

  phases = [ "installPhase" ];

  installPhase = ''
    mkdir $out

    rpmextract $src
    mv ./usr/share $out
    ls ./usr/x86_64-w64-mingw32/sys-root/mingw/lib/
    mv ./usr/x86_64-w64-mingw32/sys-root/mingw/lib $out
  '';

  meta = {
    homepage = "https://packages.fedoraproject.org/pkgs/mingw-crt";
    description = "";
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
    license = lib.licenses.gpl3;
    maintainers = [ ];
    platforms = [ "x86_64-linux" ];
  };
}
