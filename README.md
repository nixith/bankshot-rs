# Bankshot

this is my challenge I attempted to build for the HackPackCTF. While the initial implementation wasn't able to be integrated properly, I wanted to get a playable form up for a friend who wanted to try it. I also wanted to rewrite this in rust, just for fun.

For context: this is an LLM based challenge which is intended to be accessed only through the exposed endpoint in the docker container. If you can access the flag through that, you've won! congrats. If you obtain this flag by, say, looking at the env file, you've technically won but you're lame.

## Instructions
### Setup
1. Run `./getLLm.sh`. This will download the AI model to your machine locally in this git repo. This uses podman, but it shouldn't be hard to adapt to docker if that's your preference. You only have to run this once!
### Run the Challenge
2. Run your favorite docker compose program, i.e. `podman compose up` or `docker compose up`
3. checkout the challenge in your browser at `localhost:3000`


