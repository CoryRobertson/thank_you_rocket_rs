cargo build
docker build -t fast_thank_you_rocket -f fastbuild.Dockerfile .
docker run -it --rm -p 8080:8080 -v ./output:/output fast_thank_you_rocket