{
  lib,
  stdenv,
  fetchurl,
  rpmextract,
}:

stdenv.mkDerivation rec {
  pname = "mingw64-winpthreads";
  version = "14.0.0";

  src = fetchurl {
    url = "https://kojipkgs.fedoraproject.org//packages/mingw-winpthreads/${version}/1.eln156/noarch/mingw64-winpthreads-${version}-1.eln156.noarch.rpm";
    sha256 = "sha256-6AxW5xEalYtgEknGuNhfcx7K3PmSrZxeWH/VcKDLThI=";
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
    homepage = "https://packages.fedoraproject.org/pkgs/mingw-gcc/mingw64-winpthreads/";
    description = "";
    sourceProvenance = with lib.sourceTypes; [ binaryNativeCode ];
    license = lib.licenses.gpl3;
    maintainers = [ ];
    platforms = [ "x86_64-linux" ];
  };
}
