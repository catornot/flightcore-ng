{
  lib,
  stdenv,
  fetchurl,
  rpmextract,
}:

stdenv.mkDerivation rec {
  pname = "mingw64-libstdcpp";
  version = "16.1.1";

  src = fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/mingw-gcc/${version}/2.eln156/x86_64/mingw64-libstdc++-${version}-2.eln156.x86_64.rpm";
    sha256 = "sha256-Ufht4z7xB+KsmJOgVPW4eYlwkyupfHwQxtmWIzEZOp4=";
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
    homepage = "https://packages.fedoraproject.org/pkgs/mingw-gcc/mingw64-stdlibc++/";
    description = "";
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
    license = lib.licenses.gpl3;
    maintainers = [ ];
    platforms = [ "x86_64-linux" ];
  };
}
