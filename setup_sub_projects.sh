if [ ! -d rhythm_rs ] ; then
  git clone https://github.com/CoryRobertson/rhythm_rs.git
fi
cd rhythm_rs ; trunk build --public-url rhythm_rs/ --release
cd ..
if [ ! -d rhythm_rs_dist ] ; then
  mkdir rhythm_rs_dist
fi
cp -r ./rhythm_rs/dist/* ./rhythm_rs_dist/
if [ ! -d discreet_math_fib ] ; then
  git clone --branch web-version https://github.com/CoryRobertson/discreet_math_fib.git
fi
cd discreet_math_fib ; trunk build --public-url discreet_math_fib/ --release
cd ..
if [ ! -d discreet_math_fib_dist ] ; then
  mkdir discreet_math_fib_dist
fi
cp -r ./discreet_math_fib/dist/* ./discreet_math_fib_dist/
rm -rf discreet_math_fib rhythm_rs
