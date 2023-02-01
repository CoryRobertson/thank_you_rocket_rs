if [ !-d rhythm_rs ] ; then
  git clone https://github.com/CoryRobertson/rhythm_rs.git
fi
cd rhythm_rs ; trunk build --public-url .. --release
cd ..
cp -r ./rhythm_rs/dist/* ./static/