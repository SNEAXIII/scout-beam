.PHONY: help images deploy down
.DEFAULT_GOAL := help

API_IMAGE   := sneaxiii/scout-beam-api:latest
FRONT_IMAGE := sneaxiii/scout-beam-front:latest
STACK       := scout-beam
SCOUT_BEAM_HOST ?= scout-beam.mawster.app
# Public OAuth client ids; the secrets go through `docker secret`.
DISCORD_CLIENT_ID ?= 1571345980664709261
GOOGLE_CLIENT_ID ?=

# `stack rm` returns before its networks are gone: a deploy meanwhile loses them ("network not found").
STACK_NETWORKS = docker network ls -q --filter label=com.docker.stack.namespace=$(STACK)
WAIT_REMOVED = while [ -z "$$(docker stack services -q $(STACK) 2>/dev/null)" ] && [ -n "$$($(STACK_NETWORKS))" ]; do sleep 1; done

help: ## Show this help
	@grep -E '^[a-z-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  %-8s %s\n", $$1, $$2}'

images: ## Build both images locally
	docker build -t $(API_IMAGE) api
	docker build -t $(FRONT_IMAGE) front

# --resolve-image never: run the images built here instead of pulling them from the registry.
# The tag never changes, so Swarm sees the same spec: --force restarts the services on the new build.
deploy: images ## Build, then deploy the stack (SCOUT_BEAM_HOST overrides the host)
	@$(WAIT_REMOVED)
	SCOUT_BEAM_HOST=$(SCOUT_BEAM_HOST) DISCORD_CLIENT_ID=$(DISCORD_CLIENT_ID) \
		GOOGLE_CLIENT_ID=$(GOOGLE_CLIENT_ID) docker stack deploy --resolve-image never -c stack.yaml $(STACK)
	docker service update --force --no-resolve-image --image $(API_IMAGE) $(STACK)_api
	docker service update --force --no-resolve-image --image $(FRONT_IMAGE) $(STACK)_front

down: ## Remove the stack (the database volume stays), once its networks are gone too
	docker stack rm $(STACK)
	@$(WAIT_REMOVED)
