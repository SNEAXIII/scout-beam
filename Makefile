.PHONY: images deploy down

API_IMAGE   := sneaxiii/scout-beam-api:latest
FRONT_IMAGE := sneaxiii/scout-beam-front:latest
SCOUT_BEAM_HOST ?= scout-beam.mawster.app

images: ## Build both images locally
	docker build -t $(API_IMAGE) api
	docker build -t $(FRONT_IMAGE) front

# --resolve-image never: run the images built here instead of pulling them from the registry.
# The tag never changes, so Swarm sees the same spec: --force restarts the services on the new build.
deploy: images ## Build, then deploy the stack (SCOUT_BEAM_HOST overrides the host)
	SCOUT_BEAM_HOST=$(SCOUT_BEAM_HOST) docker stack deploy --resolve-image never -c stack.yaml scout-beam
	docker service update --force --no-resolve-image --image $(API_IMAGE) scout-beam_api
	docker service update --force --no-resolve-image --image $(FRONT_IMAGE) scout-beam_front

down: ## Remove the stack (the database volume stays)
	docker stack rm scout-beam
