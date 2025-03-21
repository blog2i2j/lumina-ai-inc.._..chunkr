# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.4.2](https://github.com/lumina-ai-inc/chunkr/compare/v1.4.1...v1.4.2) (2025-03-12)


### Bug Fixes

* Github action now removes v from version before tagging ([6c77a1f](https://github.com/lumina-ai-inc/chunkr/commit/6c77a1f5f435c362ec62aabb8bd29a78cc7eba1e))
* Moved infrastructure from values.yaml to infrastructure.yaml ([e4ba284](https://github.com/lumina-ai-inc/chunkr/commit/e4ba284b85c3290f585abce36d97c8c9860bdb9a))

## [1.4.1](https://github.com/lumina-ai-inc/chunkr/compare/v1.4.0...v1.4.1) (2025-03-12)


### Bug Fixes

* Continue on error on docker build ([aca0b44](https://github.com/lumina-ai-inc/chunkr/commit/aca0b4444875a1b053924a60380e6ee44a4dc005))

## [1.4.0](https://github.com/lumina-ai-inc/chunkr/compare/v1.3.5...v1.4.0) (2025-03-12)


### Features

* /health return current version ([627e8c9](https://github.com/lumina-ai-inc/chunkr/commit/627e8c9a1160bf4a360f6d0ea0f1376f64344642))


### Bug Fixes

* Updated changelog paths ([d20b811](https://github.com/lumina-ai-inc/chunkr/commit/d20b8112fc5043f5eecabf1e72e89412b1b5e7b1))

## [1.3.5](https://github.com/lumina-ai-inc/chunkr/compare/v1.3.4...v1.3.5) (2025-03-12)


### Bug Fixes

* Added back segmentation docker with self hosted runner ([0984ba2](https://github.com/lumina-ai-inc/chunkr/commit/0984ba2710fca19a807985e5a92fbf1e185bbb03))

## [1.3.4](https://github.com/lumina-ai-inc/chunkr/compare/v1.3.3...v1.3.4) (2025-03-11)


### Bug Fixes

* Removed segmenetation from docker build ([5dc9e6e](https://github.com/lumina-ai-inc/chunkr/commit/5dc9e6e5d1687bbe6ab3555f7df5656856a43f34))

## [1.3.3](https://github.com/lumina-ai-inc/chunkr/compare/v1.3.2...v1.3.3) (2025-03-11)


### Bug Fixes

* Updated rust version for docker builds ([e5a3633](https://github.com/lumina-ai-inc/chunkr/commit/e5a3633e970dacae3ce08e42f5d7249aed592fa6))

## [1.3.2](https://github.com/lumina-ai-inc/chunkr/compare/v1.3.1...v1.3.2) (2025-03-11)


### Bug Fixes

* Release-please docker build ([6e1ff43](https://github.com/lumina-ai-inc/chunkr/commit/6e1ff43ad0d5780d2f4a6e67b0b2bcc47d8964f6))

## [1.3.1](https://github.com/lumina-ai-inc/chunkr/compare/v1.3.0...v1.3.1) (2025-03-11)


### Bug Fixes

* Docker compose updated uses pr ([f45abd1](https://github.com/lumina-ai-inc/chunkr/commit/f45abd130d4c643c288c3492bb27f6736059dfbf))

## [1.3.0](https://github.com/lumina-ai-inc/chunkr/compare/v1.2.0...v1.3.0) (2025-03-11)


### Features

* Added automation of docker build ([#365](https://github.com/lumina-ai-inc/chunkr/issues/365)) ([f01cb2f](https://github.com/lumina-ai-inc/chunkr/commit/f01cb2fc66c104066f1188149cdbbb8390337169))


### Bug Fixes

* Debugging please release ([e574177](https://github.com/lumina-ai-inc/chunkr/commit/e574177cc28c68e86ab08ac5b83328b393b02bf4))
* Debugging please release with core changes ([558a6f9](https://github.com/lumina-ai-inc/chunkr/commit/558a6f9fd86c5d6e53b770dd48909a3a60e7f110))
* Docker builds use root version ([82e1768](https://github.com/lumina-ai-inc/chunkr/commit/82e176868e215f550377d9aed91e5b37fd57faba))
* Docker compose files update separately ([15328a2](https://github.com/lumina-ai-inc/chunkr/commit/15328a23dfd4399b6a56babb18becd04bf7bdf72))
* Image tag updates not full image ([7b8791f](https://github.com/lumina-ai-inc/chunkr/commit/7b8791f6bdee1e2b5f47496936700de4ddaee537))
* Only trigger docker build after releases created ([676c280](https://github.com/lumina-ai-inc/chunkr/commit/676c280e975ea37a8a737876854b0e3aa7006fc2))

## [1.2.0](https://github.com/lumina-ai-inc/chunkr/compare/v1.1.0...v1.2.0) (2025-03-11)


### Features

* Added release please for automated releases ([#363](https://github.com/lumina-ai-inc/chunkr/issues/363)) ([d808d4e](https://github.com/lumina-ai-inc/chunkr/commit/d808d4e72464b83590dfab73fe973e2f98b4f7e7))


### Bug Fixes

* Await was missing in response ([1ad37d8](https://github.com/lumina-ai-inc/chunkr/commit/1ad37d851ee0379c13ba663fc8bafb3541e409a2))
* Await was missing in response ([632adce](https://github.com/lumina-ai-inc/chunkr/commit/632adce42c7850a788e0e46817e2498724c76890))

### Added
- Added route `POST /task/parse` and `PATCH /task/{task_id}/parse` to parse a task. These routes are exactly the same as the `POST /task` and `PATCH /task/{task_id}` routes, but don't use a multipart request.
>The old routes are deprecated but will continue to work for the foreseeable future.
- Batch parallelization, so individual tasks can take full advantage of unused GPU resources.

### Changed
- OCR `All` is now the default strategy
- Significant improvements to OCR quality

### Removed
- Removed terraform directory

### Fixed
- Fixed bug in saving output from the python client


## [1.1.0] - 2025-01-29

### Added
- Added `chunk_processing` config to control chunking
- Added `high_resolution` config to control image density
- Added `segmentation_processing` config to control LLM processing on the segments
- Added `segmentation_strategy` to control segmentation
- Added `expires_in` to API and self deployment config, it is the number of seconds before the task expires and is deleted
- Concurrent OCR and segmentation
- Concurrent page processing
- CPU support - run with `docker compose up -f compose-cpu.yaml -d`
- Python client - `pip install chunkr-ai`
- PATCH `/task/{task_id}` - allows you to update the configuration for a task. Only the steps that are updated will be re-run.
- DELETE `/task/{task_id}` - allows you to delete a task as long as it Status is not `Processing`
- GET `/task/{task_id}/cancel` - allows you to cancel a task before Status is `Processing`
- Helm chart
- Cloudflared tunnel support for https
- Azure support for self deployment
- Minio support for storage
- Python client
- Optionally get base64 encoded files from the API rather than a presigned URL
- Upload base64 encoded files and presigned URLs, when using the Python client

### Changed
- Combined all workers into a `task` worker. See [279](https://github.com/lumina-ai-inc/chunkr/issues/279)
- Redis is now part of the kubernetes deployment
- Documentation
- Improved segmentation quality and speed
- Dashboard has table view - search, deletion, cancellation
- Viewer - better ux
- Better usage tracking - includes graph
- Landing page

### Fixed
- List items incorrect heuristics. See [276](https://github.com/lumina-ai-inc/chunkr/issues/276)
- Reading order

### Removed
(All changes maintain compatibility with old configs)
- Deprecated `model` config
- Deprecated `target_chunk_length`, you can now use `chunk_processing.target_length` instead 
- Deprecated `structured_extraction.json_schema.type`
- Deprecated `ocr_strategy.Off`
- Deprecated `expires_at` in the Python client
