.PHONY: build test test-requirements fmt lint tox

build:
	rm -rf build
	pip install -e .[flask]

test-requirements: .make.test-requirements-install

.make.test-requirements-install: requirements-test.txt
	pip install pytest
	pip install -r requirements-test.txt
	touch $@

dev-requirements: .make.dev-requirements-install

.make.dev-requirements-install: requirements-dev.txt
	pip install -r requirements-dev.txt
	touch $@

test:
	@$(MAKE) tox

fmt: dev-requirements
	black .

lint: dev-requirements
	flake8 .

typecheck: dev-requirements
	mypy tests
	mypy sqlalchemy_oso

package:
	python setup.py sdist bdist_wheel

tox: dev-requirements
	@tox -p
