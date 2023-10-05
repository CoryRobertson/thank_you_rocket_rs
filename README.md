# Thank You Rocket Rs
A personal web page that has notable features:
- Permanent messaging system
- Pastebin like functionality
- File uploading for whitelisted users
- Links to other projects, hosted on the same website when possible

A message reception web server that allows a person to connect, write a message, and have it be stored on the server. 
The messages are viewable only to the user who sent them, and the server host.

When using the docker image for the website, the hash salt can be mounted as a part of a volume (see the docker compose file), to allow a salt to be shared among multiple builds of the docker image.

Other stored data is also mounted to the same volume as the salt allowing for persistent data between builds, as well as inspecting the data, if needed.

### This project was inspired by [saythanks.io](https://saythanks.io/)