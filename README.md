# Thank You Rocket Rs
A message reception web server that allows a person to connect, write a message, and have it be stored on the server. 
The messages are viewable only to the user who sent them, and the server host.

At the moment, the actual ui displayed to the user is pretty bare bones, I don't plan on going super far with it, as it's beyond the scope of the project.

When using the docker image for the website, the hash salt can be mounted as a volume (see the docker compose file), to allow a salt to be shared among multiple builds of the docker image.

Other stored data is also mounted to the same volume as the salt allowing for persistent data between builds, as well as inspecting the data, if needed.

### This project was inspired by [saythanks.io](https://saythanks.io/)