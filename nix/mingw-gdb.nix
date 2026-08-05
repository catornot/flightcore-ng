{
  lib,
  stdenv,
  fetchurl,
  rpmextract,
  mingw-libgcc,
  mingw-stdlibcpp,
  mingw-winpthreads,
  mingw-gmp,
  mingw-expat,
  mingw-mpfr,
  mingw-filesystem,
  mingw-crt,
}:

stdenv.mkDerivation rec {
  pname = "mingw64-gdb";
  version = "17.1";

  src = fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/mingw-gdb/${version}/2.fc44/noarch/mingw64-gdb-${version}-2.fc44.noarch.rpm";
    sha256 = "sha256-EkXhFdRcBVoxrsq08JRmmGf0Xf1I8BfZa1obzDeBkXM=";
  };

  dontUnpack = true;

  nativeBuildInputs = [ rpmextract ];

  phases = [ "installPhase" ];

  installPhase = ''
    mkdir $out

    rpmextract $src
    mv ./usr/share $out
    mv ./usr/x86_64-w64-mingw32/sys-root/mingw/bin $out
    mv ./usr/x86_64-w64-mingw32/sys-root/mingw/share/* $out/share

    ln -s ${mingw-libgcc}/bin/* $out/bin
    ln -s ${mingw-stdlibcpp}/bin/* $out/bin
    ln -s ${mingw-winpthreads}/bin/* $out/bin
    ln -s ${mingw-expat}/bin/* $out/bin
    ln -s ${mingw-gmp}/bin/* $out/bin
    ln -s ${mingw-mpfr}/bin/* $out/bin
    ln -s ${mingw-filesystem}/bin/* $out/bin
    # ln -s ${mingw-crt}/bin/* $out/bin
  '';

  meta = {
    homepage = "https://packages.fedoraproject.org/pkgs/mingw-gdb/mingw64-gdb/";
    description = "";
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
    license = lib.licenses.gpl3;
    maintainers = [ ];
    platforms = [ "x86_64-linux" ];
  };
}
