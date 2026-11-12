.PHONY: images deploy down

API_IMAGE   := sneaxiii/scout-beam-api:latest
FRONT_IMAGE := sneaxiii/scout-beam-front:latest

images: ## Build both images locally
	docker build -t $(API_IMAGE) api
	docker build -t $(FRONT_IMAGE) front

# --resolve-image never: run the images built here instead of pulling them from the registry.
deploy: images ## Build, then deploy the stack: make deploy SCOUT_BEAM_HOST=scout-beam.example.com
	SCOUT_BEAM_HOST=$(SCOUT_BEAM_HOST) docker stack deploy --resolve-image never -c stack.yaml scout-beam

down: ## Remove the stack (the database volume stays)
	docker stack rm scout-beam
