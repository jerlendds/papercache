.PHONY: watch build

watch:
	(cd ui && npm run dev) & \
	UI_PID=$$!; \
	cargo watch -x run & \
	API_PID=$$!; \
	trap 'kill $$UI_PID $$API_PID' INT TERM EXIT; \
	wait

build:
	cd ui && npm run build && cd .. && cargo build --release
