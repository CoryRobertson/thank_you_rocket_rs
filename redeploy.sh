git pull
docker build -t thank_you_rocket -f Dockerfile .
docker-compose down
docker-compose up -d