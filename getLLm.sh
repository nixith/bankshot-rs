#!/usr/bin/env sh

# this is a shell script because podman/docker doesn't like it
# when you do this in compose. https://github.com/ollama/ollama/issues/3578
#
# As a benefit, you don't have to wait for the model to pull every time you run.
#
# Repalce with docker as desired.

# run ollama in background - need serve to pull https://github.com/ollama/ollama/issues/3369
podman run -d --name ollama --replace -v ./.ollama/ollama:/root/.ollama -p 11434:11434 ollama/ollama:latest

# pull mistral into the same directory docker-compose ollama mounts too
podman exec -it ollama ollama pull mistral:instruct

# kill  the old ollama
podman kill ollama

#remove the old ollama
podman rm ollama
