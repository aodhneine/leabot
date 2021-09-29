#!/usr/bin/fish
function info
  printf "%s=>%s " (set_color -o blue) (set_color normal)
  printf "%s%s%s\n" (set_color -o) "$argv" (set_color normal)
end

info "cleaning old build files"
rm -rf build libretls

info "creating build folder"
mkdir build
set out_dir (pwd)/build

info "cloning libretls"
git clone https://github.com/libressl-portable/portable.git libretls --depth 1
pushd libretls

info "configuring libretls"
./autogen.sh
./configure CC=clang CFLAGS='-O2 -march=native -pipe' --prefix=$out_dir

info "building libretls"
make -j17

info "installing libretls"
make install

popd
